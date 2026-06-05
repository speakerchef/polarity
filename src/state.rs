#![allow(unused_variables, dead_code)]
use std::{
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::mpsc::{Receiver, Sender, channel},
    time::Duration,
};

use egui::{Align2, vec2};
use egui_file_dialog::{self as fd, FileDialog, FileDialogConfig};
use rodio::Source;

fn file_as_raw_bytes(path: PathBuf) -> Vec<u8> {
    let mut bytes = Vec::new();
    fs::File::open(path.clone())
        .expect("error opening file")
        .read_to_end(&mut bytes)
        .expect("error reading file");
    bytes
}

macro_rules! labeled_enum {
    ($name:ident { $($variant:ident => $label:literal),+ $(,)?}, $def:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];
            pub fn label(self) -> &'static str {
                match self { $($name::$variant => $label),+ }
            }
        }
        impl Default for $name {
            fn default() -> Self { $name::$def }
        }
    };
}

labeled_enum!(StereometerKind {
    LinearBipolar  => "Linear Bipolar",
    ScaledBipolar  => "Scaled Bipolar",
    LinearLissajous => "Linear Lissajous",
    ScaledLissajous => "Scaled Lissajous",
}, ScaledLissajous);

labeled_enum!(RenderMode {
    FullSpectrum => "Full Spectrum",
    MultiBand    => "Multi-Band",
}, MultiBand);

labeled_enum!(FilterMode {
    Off => "Off",
    Lpf => "Lpf",
    Bpf => "Bpf", 
    Hpf => "Hpf",
}, Off);

pub trait Labeled: Copy + PartialEq {
    fn text(self) -> &'static str;
}
impl Labeled for crate::state::RenderMode {
    fn text(self) -> &'static str {
        self.label()
    }
}
impl Labeled for crate::state::FilterMode {
    fn text(self) -> &'static str {
        self.label()
    }
}
impl Labeled for crate::state::StereometerKind {
    fn text(self) -> &'static str {
        self.label()
    }
}
pub type Hsl = (f32, f32, f32);

#[derive(Clone, Copy, Debug, Default)]
pub enum PlaybackState {
    #[default]
    Pause,
    Play,
    Stop,
}

pub struct AudioPlayer {
    pub contents: AudioFileContents,
    handle: std::thread::JoinHandle<()>,
    playback_tx: Sender<PlaybackState>,
    end_rx: Receiver<bool>,
    playback_state: PlaybackState,
}

impl AudioPlayer {
    pub fn new(path: PathBuf) -> Self {
        // channels for state mgmt
        let (pb_tx, pb_rx) = channel::<PlaybackState>();
        let (end_tx, end_rx) = channel::<bool>();

        let bytes = file_as_raw_bytes(path.clone());
        let decoder = AudioPlayer::create_decoder(bytes.clone());
        let contents = AudioFileContents {
            path: path.clone(),
            duration: decoder.total_duration().unwrap(),
            sample_rate: decoder.sample_rate().into(),
            num_channels: decoder.channels().into(),
            samples: AudioPlayer::create_decoder(bytes).collect(),
        };

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
                if let Ok(cmd) = pb_rx.recv() {
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
            handle,
            playback_tx: pb_tx,
            end_rx,
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

    /// Signals that there's no more audio to play
    pub fn ended(&self) -> bool {
        let Ok(ended) = self.end_rx.try_recv() else {
            return false;
        };
        ended || matches!(self.playback_state, PlaybackState::Stop)
    }

    fn create_decoder(bytes: Vec<u8>) -> rodio::Decoder<Cursor<Vec<u8>>> {
        rodio::Decoder::new(Cursor::new(bytes)).unwrap()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AudioFileContents {
    pub path: PathBuf,
    pub duration: std::time::Duration,
    pub sample_rate: u32,
    pub num_channels: u16,
    pub samples: Vec<f32>,
}

pub struct AppState {
    pub file_dialog: FileDialog,
    pub render_mode: RenderMode,
    pub filter_mode: FilterMode,
    pub stereo_kind: StereometerKind,

    pub playback_state: PlaybackState,

    pub import_open: bool,
    pub file_loaded: bool,

    pub gen_open: bool,
    pub render_open: bool,
    pub render_mode_options_open: bool,
    pub stereo_kind_options_open: bool,
    pub filtering_open: bool,
    pub filter_mode_options_open: bool,
    pub mode_open: bool,
    pub color_open: bool,
    pub visual_open: bool,

    pub postfx_open: bool,
    pub sparkle_open: bool,

    pub filter_freq: f32,
    pub hsl_color_bands: [Hsl; 3],
    pub bloom: f32,

    //Timeline
    pub elapsed_time: Duration,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<bool>();
        let config = FileDialogConfig {
            file_filters: vec![],
            ..Default::default()
        };
        Self {
            file_dialog: FileDialog::new()
                .opening_mode(egui_file_dialog::OpeningMode::LastVisitedDir)
                .show_left_panel(true)
                .show_pinned_folders(true)
                .add_file_filter(
                    "Audio",
                    fd::Filter::new(|path: &Path| {
                        path.extension().unwrap_or_default() == "wav"
                            || path.extension().unwrap_or_default() == "mp3"
                    }),
                )
                .default_file_filter("Audio")
                .allow_file_overwrite(true)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0)),
            render_mode: RenderMode::default(),
            filter_mode: FilterMode::default(),
            stereo_kind: StereometerKind::default(),
            playback_state: PlaybackState::default(),

            import_open: false,
            file_loaded: false,

            gen_open: false,
            render_open: false,
            render_mode_options_open: false,
            stereo_kind_options_open: false,
            filtering_open: false,
            filter_mode_options_open: false,
            mode_open: false,
            color_open: false,
            visual_open: false,

            postfx_open: false,
            sparkle_open: false,
            filter_freq: 1.0,
            hsl_color_bands: [(0.0, 1.0, 0.50), (120.0, 1.0, 0.50), (240.0, 1.0, 0.50)],
            bloom: 0.4,

            elapsed_time: Duration::default(),
        }
    }
}
