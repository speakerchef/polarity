use std::{
    io::Cursor,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    time::Duration,
};

use rodio::{Source, source::SeekError};

use crate::audio::file_as_raw_bytes;

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AudioFileContents {
    pub path: PathBuf,
    pub duration: std::time::Duration,
    pub sample_rate: u32,
    pub num_channels: u16,
    pub samples: Vec<f32>,
}
#[derive(Clone, Copy, Debug, Default)]
pub enum PlaybackState {
    #[default]
    Pause,
    Play,
    Stop,
}

pub struct AudioPlayer {
    pub contents: AudioFileContents,
    pos: Arc<Mutex<Duration>>,
    handle: std::thread::JoinHandle<()>,
    playback_tx: Sender<PlaybackState>,
    end_rx: Receiver<bool>,
    seekreq_tx: Sender<Duration>,
    seekresp_rx: Receiver<Result<(), SeekError>>,
    playback_state: PlaybackState,
}

impl AudioPlayer {
    pub fn new(path: PathBuf) -> Self {
        // channels for state mgmt
        let (pb_tx, pb_rx) = channel::<PlaybackState>();
        let (end_tx, end_rx) = channel::<bool>();
        let (seekreq_tx, seekreq_rx) = channel::<Duration>();
        let (seekresp_tx, seekresp_rx) = channel::<Result<(), SeekError>>();

        let bytes = file_as_raw_bytes(path.clone());
        let decoder = AudioPlayer::create_decoder(bytes.clone());
        let contents = AudioFileContents {
            path: path.clone(),
            duration: decoder.total_duration().unwrap(),
            sample_rate: decoder.sample_rate().into(),
            num_channels: decoder.channels().into(),
            samples: AudioPlayer::create_decoder(bytes).collect(),
        };

        let pos = Arc::new(Mutex::new(Duration::default()));
        let cur_pos = Arc::clone(&pos);
        let handle = std::thread::spawn(move || {
            let mut sink =
                rodio::DeviceSinkBuilder::open_default_sink().expect("Open default sink");
            sink.log_on_drop(false);
            let sink = rodio::Player::connect_new(sink.mixer());
            sink.append(decoder);
            sink.pause();
            loop {
                if sink.empty() {
                    end_tx.send(true).unwrap();
                    break;
                }
                if let Ok(cmd) = pb_rx.try_recv() {
                    match cmd {
                        PlaybackState::Play => {
                            sink.play();
                        }
                        PlaybackState::Pause => {
                            sink.pause();
                        }
                        PlaybackState::Stop => break,
                    }
                }
                if let Ok(seekamt) = seekreq_rx.try_recv() {
                    let res = sink.try_seek(seekamt);
                    seekresp_tx.send(res).unwrap();
                }
                *cur_pos.lock().unwrap() = sink.get_pos();
            }
        });

        println!(
            "Audio Data: {:?}",
            AudioFileContents {
                samples: Vec::new(),
                path: contents.path.clone(),
                ..contents
            }
        );

        Self {
            pos,
            handle,
            playback_tx: pb_tx,
            end_rx,
            seekreq_tx,
            seekresp_rx,
            playback_state: PlaybackState::default(),
            contents,
        }
    }

    /// Deletes the player & closes audio thread
    pub fn clear(mut self) {
        self.stop();
        self.handle.join().ok();
    }

    pub fn play(&mut self) {
        self.playback_state = PlaybackState::Play;
        self.playback_tx.send(PlaybackState::Play).unwrap();
    }
    pub fn pause(&mut self) {
        self.playback_state = PlaybackState::Pause;
        self.playback_tx.send(PlaybackState::Pause).unwrap();
    }
    pub fn stop(&mut self) {
        self.playback_state = PlaybackState::Stop;
        self.playback_tx.send(PlaybackState::Stop).unwrap();
    }
    pub fn is_paused(&self) -> bool {
        matches!(self.playback_state, PlaybackState::Pause)
    }

    pub fn position(&self) -> Duration {
        *self.pos.lock().unwrap()
    }

    pub fn try_seek(&mut self, d: Duration) -> Result<(), SeekError> {
        self.seekreq_tx.send(d).unwrap();
        self.seekresp_rx.recv().unwrap_or(Ok(()))
    }

    /// Signals that there's no more audio to play
    pub fn ended(&self) -> bool {
        let Ok(ended) = self.end_rx.try_recv() else {
            return false;
        };
        ended || matches!(self.playback_state, PlaybackState::Stop)
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
