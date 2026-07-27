#![allow(dead_code)]
use std::{
    collections::VecDeque,
    io::Cursor,
    num::NonZero,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use eframe::egui::emath::Numeric;
use rodio::{
    DeviceSinkError, DeviceTrait, Source,
    cpal::{
        self, BufferSize,
        traits::{HostTrait, StreamTrait},
    },
    source::SeekError,
};
use smart_default::SmartDefault;

use crate::{audio::file_as_raw_bytes, traits::AudioProperties};

pub struct ArcAudioSource {
    samples: Arc<[f32]>,
    num_channels: NonZero<u16>,
    sample_rate: NonZero<u32>,
    total_duration: Option<Duration>,
    pos: usize,
}

impl ArcAudioSource {
    pub fn new(
        samples: Arc<[f32]>,
        num_channels: NonZero<u16>,
        sample_rate: NonZero<u32>,
        total_duration: Option<Duration>,
    ) -> Self {
        Self {
            samples,
            num_channels,
            sample_rate,
            total_duration,
            pos: 0,
        }
    }
}

impl Iterator for ArcAudioSource {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.samples.get(self.pos).copied();
        self.pos += 1;
        v
    }
}

impl Source for ArcAudioSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> rodio::ChannelCount {
        self.num_channels
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let cur_channel = self.pos % self.num_channels.get() as usize;
        let new_pos =
            (pos.as_secs_f64() * self.sample_rate.to_f64() * self.num_channels.to_f64()) as usize;
        let new_pos = new_pos
            .min(self.samples.len())
            .next_multiple_of(self.num_channels.get() as usize);
        self.pos = new_pos.saturating_sub(cur_channel);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct AudioFileContents {
    pub path: PathBuf,
    pub duration: std::time::Duration,
    pub sample_rate: u32,
    pub num_channels: u16,
    pub samples: Arc<[f32]>,
}

#[derive(Default)]
pub struct AudioPlayer {
    pub contents: AudioFileContents,
    sink: Option<rodio::Player>,

    // internal to keep the sink alive
    _stream: Option<rodio::MixerDeviceSink>,
}

impl AudioProperties for AudioPlayer {
    fn is_live(&self) -> bool {
        false
    }

    fn sample_rate(&self) -> u32 {
        self.contents.sample_rate
    }

    fn num_channels(&self) -> u16 {
        self.contents.num_channels
    }

    fn audio_buffer(&self) -> &[f32] {
        &self.contents.samples
    }

    fn position(&self) -> Duration {
        self.position()
    }

    fn popped_sample_count(&self) -> usize {
        0
    }
}

#[derive(SmartDefault)]
pub struct LiveInput {
    _stream: Option<cpal::Stream>,

    #[default(48000)]
    sample_rate: u32,
    #[default(2)]
    num_channels: u16,

    /// Receives the audio buffer
    instream_rx: Option<flume::Receiver<Vec<f32>>>,

    #[default(Instant::now())]
    /// Time when stream started
    init_timestamp: Instant,

    buffer: VecDeque<f32>,
    popped_samples: usize,
}

impl AudioProperties for LiveInput {
    fn is_live(&self) -> bool {
        true
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn num_channels(&self) -> u16 {
        self.num_channels
    }

    fn audio_buffer(&self) -> &[f32] {
        self.get_audio_slice()
    }

    fn position(&self) -> Duration {
        Instant::now().saturating_duration_since(self.init_timestamp)
    }

    fn popped_sample_count(&self) -> usize {
        self.popped_samples
    }
}

#[allow(dead_code)]
impl AudioPlayer {
    pub fn new(path: PathBuf, paused: bool) -> Result<Self, DeviceSinkError> {
        let mut stream = rodio::DeviceSinkBuilder::open_default_sink()?;
        stream.log_on_drop(false);

        let sink = rodio::Player::connect_new(stream.mixer());
        let bytes = file_as_raw_bytes(path.clone());
        let decoder = Self::create_decoder(bytes);

        let duration = decoder.total_duration();
        let fs = decoder.sample_rate();
        let ch = decoder.channels();
        let alloc_size =
            (duration.unwrap().as_secs_f64() * fs.to_f64() * ch.to_f64()).round() as usize;
        let mut arc = Arc::<[f32]>::new_uninit_slice(alloc_size);
        let data = Arc::get_mut(&mut arc).unwrap();

        let mut written = 0;
        for (i, s) in decoder.enumerate() {
            if i >= alloc_size {
                break;
            }
            data[i].write(s);
            written += 1;
        }
        (written..alloc_size).for_each(|i| {
            data[i].write(0.0);
        });

        // SAFETY: Arc always init with samples from decoder or zero padded
        let s = unsafe { arc.assume_init() };
        let contents = AudioFileContents {
            path,
            duration: duration.unwrap(),
            sample_rate: fs.into(),
            num_channels: ch.into(),
            samples: Arc::clone(&s),
        };

        sink.append(ArcAudioSource::new(s, ch, fs, duration));
        if paused {
            sink.pause();
        }

        Ok(Self {
            contents,
            sink: Some(sink),
            _stream: Some(stream),
        })
    }
    pub fn play(&self) {
        self.sink.as_ref().expect("duh").play();
    }
    pub fn pause(&self) {
        self.sink.as_ref().expect("duh").pause();
    }
    pub fn toggle_playback(&self) {
        if self.sink.as_ref().expect("duh").is_paused() {
            self.sink.as_ref().expect("duh").play();
        } else {
            self.sink.as_ref().expect("duh").pause();
        }
    }
    pub fn is_paused(&self) -> bool {
        self.sink.as_ref().expect("duh").is_paused()
    }
    pub fn position(&self) -> Duration {
        self.sink.as_ref().expect("duh").get_pos()
    }
    pub fn try_seek(&self, pos: Duration) -> Result<(), SeekError> {
        self.sink.as_ref().expect("duh").try_seek(pos)
    }
    /// Signals that there's no more audio to play
    pub fn ended(&self) -> bool {
        self.sink.as_ref().expect("duh").empty()
    }

    fn create_decoder(bytes: Vec<u8>) -> rodio::Decoder<Cursor<Vec<u8>>> {
        let len = bytes.len() as u64;
        rodio::Decoder::builder()
            .with_data(Cursor::new(bytes))
            .with_byte_len(len)
            .build()
            .unwrap()
    }
}

impl LiveInput {
    pub fn start_stream(
        &mut self,
        buffer_size: u32,
        device: Option<cpal::Device>,
    ) -> Result<cpal::Device, Box<dyn std::error::Error>> {
        let dev = device.unwrap_or(
            cpal::default_host()
                .default_output_device()
                .ok_or("No default output device available")?,
        );

        println!("Selected device: {}", dev.description().unwrap());

        let default_config = dev.default_output_config()?;
        let sample_format = default_config.sample_format();
        let mut config = default_config.config();
        self.sample_rate = config.sample_rate;
        self.num_channels = config.channels;

        let buf_size = match config.buffer_size {
            BufferSize::Fixed(v) => v,
            BufferSize::Default => {
                config.buffer_size = BufferSize::Fixed(buffer_size);
                buffer_size
            }
        };

        let (audio_tx, audio_rx) = flume::bounded((self.sample_rate / buf_size) as usize);
        let stream = dev.build_input_stream_raw(
            &config,
            sample_format,
            move |data: &cpal::Data, _| {
                let data = data
                    .as_slice()
                    .iter()
                    .flat_map(|d: &&[f32]| d.iter().copied())
                    .collect::<Vec<f32>>();
                audio_tx.try_send(data.to_vec()).unwrap_or_default();
            },
            move |e| println!("{e}"),
            None,
        )?;
        stream.play()?;
        self.instream_rx = Some(audio_rx);
        self._stream = Some(stream);

        Ok(dev)
    }

    pub fn get_audio_slice(&self) -> &[f32] {
        self.buffer.as_slices().0
    }

    pub fn update_buffer(&mut self) {
        self.popped_samples = 0;

        if let Some(rx) = self.instream_rx.as_ref() {
            while let Ok(input) = rx.try_recv() {
                self.buffer.extend(input);
            }
        }

        while self.buffer.len() > self.sample_rate as usize * self.num_channels as usize {
            self.buffer.pop_front();
            self.popped_samples += 1;
        }

        self.buffer.make_contiguous();
    }
}
