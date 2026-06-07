use biquad::*;
use std::{path::PathBuf, time::Duration};

use crate::{
    audio::{StereoFilter, audio_player::*},
    state::{FilterMode, PlaybackMode},
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
                self.spawn_audio_player(path, true);
            }
        } else {
            self.spawn_audio_player(path, true);
        }
    }

    pub fn spawn_audio_player(&mut self, path: PathBuf, paused: bool) {
        // Clear old player
        self.player.take();

        self.player = AudioPlayer::new(path, paused)
            .inspect_err(|err| println!("{}", err))
            .ok();
    }

    pub fn handle_playback(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::Space)) {
            if let Some(player) = &mut self.player {
                player.toggle_playback();
            } else {
                println!("No audio player loaded");
            }
        }
    }

    pub fn update_filters(&mut self) {
        if self.st.stereo.live_fs_filters.is_none()
            && let Some(p) = &self.player
        {
            self.st.set_default_freqs = true;
            let st = &mut self.st.stereo;
            let filters = Some((
                StereoFilter::from_coeffs_butterworth(Type::LowPass, 200., p.contents.sample_rate),
                StereoFilter::from_coeffs_butterworth(
                    Type::BandPass,
                    1000.,
                    p.contents.sample_rate,
                ),
                StereoFilter::from_coeffs_butterworth(
                    Type::HighPass,
                    5000.,
                    p.contents.sample_rate,
                ),
            ));
            st.live_fs_filters = filters.clone();
            st.trace_fs_filters = filters.clone();
            st.live_mb_filters = filters.clone();
            st.trace_mb_filters = filters;
        }

        if self.st.stereo.live_fs_filters.is_some()
            && let Some(p) = &self.player
            && self.st.stereo.last_freq != self.st.stereo.filter_freq
        {
            self.st.set_default_freqs = false;
            let st = &mut self.st.stereo;
            st.last_freq = st.filter_freq;
            let livefs = st.live_fs_filters.as_mut().unwrap();
            let tracefs = st.trace_fs_filters.as_mut().unwrap();
            match st.filter_mode {
                FilterMode::Off => (),
                FilterMode::Lpf => {
                    livefs.0 = StereoFilter::from_coeffs_butterworth(
                        Type::LowPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                    tracefs.0 = StereoFilter::from_coeffs_butterworth(
                        Type::LowPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                }
                FilterMode::Bpf => {
                    livefs.1 = StereoFilter::from_coeffs_butterworth(
                        Type::BandPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                    tracefs.1 = StereoFilter::from_coeffs_butterworth(
                        Type::BandPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                }
                FilterMode::Hpf => {
                    livefs.2 = StereoFilter::from_coeffs_butterworth(
                        Type::HighPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                    tracefs.2 = StereoFilter::from_coeffs_butterworth(
                        Type::HighPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                }
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
                canvas::draw(ui, &mut self.st, &self.player);
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
        {
            self.load_file(path);
        };
        self.update_filters();
        self.handle_playback(ctx);

        // Check if playback has ended
        if let Some(player) = &self.player
            && player.ended()
        {
            println!("Respawning player");
            let paused = matches!(self.st.playback_mode, PlaybackMode::Once);
            self.spawn_audio_player(player.contents.path.to_path_buf(), paused);
        }
    }
}
