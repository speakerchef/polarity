use core::panic::PanicInfo;
use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
    path::{Path, PathBuf},
    sync::{
        Arc,
        mpsc::{Receiver, Sender, channel},
    },
};

use crate::{
    state::{AudioFileContents, AudioPlayer, PlaybackState},
    ui::control_panel,
};
use egui::{CornerRadius, Key, Stroke, StrokeKind, emath::GuiRounding};
use rodio::{self, Decoder, Source};

use crate::{state::PanelState, ui::palette as plt, ui::theme};

#[derive(Default)]
pub struct PolarityApp {
    state: PanelState,
    player: Option<AudioPlayer>,
    audio: AudioFileContents,
}

impl PolarityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::apply_theme(&cc.egui_ctx);
        Self::default()
    }

    pub fn load_file(&mut self, path: PathBuf) {
        if let Some(old_path) = &self.audio.path {
            if *old_path != path {
                println!("Diff");
                self.audio.path.take();
                self.state.file_loaded = false;
                self.spawn_audio_player(path);
            }
        } else {
            self.spawn_audio_player(path);
        }
    }

    pub fn spawn_audio_player(&mut self, path: PathBuf) {
        // Clear old player
        if let Some(mut old) = self.player.take() {
            println!("OLD");
            old.stop();
            old.handle.join().ok();
        }

        let (tx, rx) = channel::<PlaybackState>();
        let (end_tx, end_rx) = channel::<bool>();
        let (md_tx, md_rx) = channel::<AudioFileContents>();

        let handle = std::thread::spawn(move || {
            let mut sink =
                rodio::DeviceSinkBuilder::open_default_sink().expect("Open default sink");
            sink.log_on_drop(false);
            let sink = rodio::Player::connect_new(sink.mixer());

            let mut bytes = Vec::new();
            File::open(path.clone())
                .expect("error opening file")
                .read_to_end(&mut bytes)
                .expect("error reading file");
            let decoder = Decoder::new(Cursor::new(bytes.clone())).unwrap();
            md_tx
                .send(AudioFileContents {
                    path: Some(path),
                    duration: decoder.total_duration().unwrap(),
                    sample_rate: decoder.sample_rate().into(),
                    num_channels: decoder.channels().into(),
                    samples: Decoder::new(Cursor::new(bytes)).unwrap().collect(),
                })
                .unwrap();
            sink.append(decoder);
            sink.pause();

            loop {
                if sink.empty() {
                    end_tx.send(true).expect("unable to send closing message");
                    break;
                }
                if let Ok(cmd) = rx.recv() {
                    println!("Received: {:?}", cmd);
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

        self.player = Some(AudioPlayer::new(tx, end_rx, md_rx, handle));
        println!("Spawned audio player");
    }

    pub fn handle_playback(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::Space)) {
            if let Some(player) = &mut self.player {
                if player.is_paused() {
                    player.play();
                } else {
                    player.pause();
                }
            } else {
                println!("No audio player loaded");
            }
        }
    }
}

impl eframe::App for PolarityApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let resp = egui::CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(12.0).fill(plt::BG))
            .show_inside(ui, |ui| {
                control_panel::draw(ui, &mut self.state);
            });

        let border_rect = resp.response.rect.shrink(12.0).round_ui();
        ui.painter().rect_stroke(
            border_rect,
            CornerRadius::ZERO,
            Stroke {
                width: 1.0,
                color: plt::BORDER,
            },
            StrokeKind::Inside,
        );
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Open file dialog
        if self.state.import_open {
            self.state.file_dialog.pick_file();
            self.state.import_open = false;
        }

        // Check if user picked a file
        if let Some(path) = self
            .state
            .file_dialog
            .update(ctx)
            .picked()
            .map(|p| p.to_path_buf())
            && !self.state.file_loaded
        {
            self.load_file(path);
            self.state.file_loaded = true;
        };

        // load audio file data
        if let Some(p) = &self.player
            && let Ok(md) = p.metadata_rx.try_recv()
        {
            println!(
                "Metadata: {:?}",
                AudioFileContents {
                    samples: Vec::new(),
                    path: self.audio.path.clone(),
                    ..md
                }
            );
            self.audio = md;
        }

        self.handle_playback(ctx);

        // Check if playback has ended
        if let Some(player) = &self.player
            && player.ended()
        {
            println!("Respawning player");
            self.spawn_audio_player(self.audio.path.clone().unwrap().to_path_buf());
        }
    }
}
