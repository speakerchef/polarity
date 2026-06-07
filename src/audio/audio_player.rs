#![allow(dead_code)]
use std::{io::Cursor, num::NonZero, path::PathBuf, sync::Arc, time::Duration};

use egui::emath::Numeric;
use rodio::{DeviceSinkError, Source, source::SeekError};

use crate::audio::file_as_raw_bytes;

#[derive(Debug, Clone, Default)]
pub struct AudioFileContents {
    pub path: PathBuf,
    pub duration: std::time::Duration,
    pub sample_rate: u32,
    pub num_channels: u16,
    pub samples: Arc<[f32]>,
}
pub struct AudioPlayer {
    pub contents: AudioFileContents,
    sink: rodio::Player,

    // internal to keep the sink alive
    _stream: rodio::MixerDeviceSink,
}

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
            sink,
            _stream: stream,
        })
    }
    pub fn play(&self) {
        self.sink.play();
    }
    pub fn pause(&self) {
        self.sink.pause();
    }
    pub fn toggle_playback(&self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
    pub fn position(&self) -> Duration {
        self.sink.get_pos()
    }
    pub fn try_seek(&self, pos: Duration) -> Result<(), SeekError> {
        self.sink.try_seek(pos)
    }
    /// Signals that there's no more audio to play
    pub fn ended(&self) -> bool {
        self.sink.empty()
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
