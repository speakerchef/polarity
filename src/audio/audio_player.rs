#![allow(dead_code)]
use std::{io::Cursor, path::PathBuf, time::Duration};

use rodio::{Source, source::SeekError};

use crate::audio::file_as_raw_bytes;

#[derive(Debug, Clone, Default)]
pub struct AudioFileContents {
    pub path: PathBuf,
    pub duration: std::time::Duration,
    pub sample_rate: u32,
    pub num_channels: u16,
    pub samples: Vec<f32>,
}
pub struct AudioPlayer {
    pub contents: AudioFileContents,
    sink: rodio::Player,

    // internal to keep the sink alive
    _stream: rodio::MixerDeviceSink,
}

#[allow(dead_code)]
impl AudioPlayer {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        let Ok(mut stream) = rodio::DeviceSinkBuilder::open_default_sink() else {
            return Err("No output device found!".to_string());
        };
        stream.log_on_drop(false);

        let sink = rodio::Player::connect_new(stream.mixer());
        let bytes = file_as_raw_bytes(path.clone());
        let decoder = Self::create_decoder(bytes.clone());
        let contents = AudioFileContents {
            path: path.clone(),
            duration: decoder.total_duration().unwrap(),
            sample_rate: decoder.sample_rate().into(),
            num_channels: decoder.channels().into(),
            samples: Self::create_decoder(bytes).collect(),
        };

        sink.append(decoder);
        sink.pause();

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
    pub fn stop(&self) {
        self.sink.stop();
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
