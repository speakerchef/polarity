use std::{path::PathBuf, time::Duration};

use crate::{
    audio::audio_player::*,
    ui::{canvas, control_panel, timeline},
};
use egui::{CornerRadius, Key, Stroke, StrokeKind, emath::GuiRounding};

use crate::{state::AppState, ui::palette as plt, ui::theme};

#[derive(Default)]
pub struct PolarityApp {
    st: AppState,
    player: Option<AudioPlayer>,
}

impl PolarityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::apply_theme(&cc.egui_ctx);
        Self::default()
    }

    pub fn load_file(&mut self, path: PathBuf) {
        if let Some(old_player) = &self.player {
            if *old_player.contents.path != path {
                println!("Diff");
                self.st.file_loaded = false;
                self.spawn_audio_player(path);
            }
        } else {
            self.spawn_audio_player(path);
        }
    }

    pub fn spawn_audio_player(&mut self, path: PathBuf) {
        // Clear old player
        if let Some(old) = self.player.take() {
            old.clear();
        }
        self.player = Some(AudioPlayer::new(path));
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
            .frame(egui::Frame::NONE.inner_margin(12.0).fill(plt::VOID))
            .show_inside(ui, |ui| {
                timeline::draw(ui, &mut self.st, &mut self.player);
                control_panel::draw(ui, &mut self.st);
                canvas::draw(ui);
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
        if let Some(p) = &self.player
            && !p.is_paused()
        {
            ctx.request_repaint_after_secs(Duration::from_millis(16).as_secs_f32());
        }

        // Open file dialog
        if self.st.import_open {
            self.st.file_dialog.pick_file();
            self.st.import_open = false;
        }

        // Check if user picked a file
        if let Some(path) = self
            .st
            .file_dialog
            .update(ctx)
            .picked()
            .map(|p| p.to_path_buf())
            && !self.st.file_loaded
        {
            self.load_file(path);
            self.st.file_loaded = true;
        };

        self.handle_playback(ctx);
        if let Some(p) = &self.player {
            self.st.elapsed_time = p.position();
        }

        // Check if playback has ended
        if let Some(player) = &self.player
            && player.ended()
        {
            println!("Respawning player");
            self.spawn_audio_player(player.contents.path.to_path_buf());
        }
    }
}
