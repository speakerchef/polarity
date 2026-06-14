#![allow(dead_code)]
use std::path::Path;

use egui::{Align2, vec2};
use egui_file_dialog::{self as fd, FileDialog};

use crate::{Rgba, generators::stereometer::Stereometer};

#[derive(Default)]
pub enum PlaybackMode {
    #[default]
    Loop,
    Once,
}

pub trait Labeled: Copy + PartialEq {
    fn text(self) -> &'static str;
}

pub struct AppState {
    pub file_dialog: FileDialog,
    pub playback_mode: PlaybackMode,
    pub stereo: Stereometer,
    pub pos: [f32; 2],

    pub import_open: bool,

    pub gen_open: bool,
    pub render_open: bool,
    pub render_mode_options_open: bool,
    pub stereo_kind_options_open: bool,
    pub filtering_open: bool,
    pub filter_mode_options_open: bool,
    pub mode_open: bool,
    pub color_open: bool,
    pub visual_open: bool,
    pub density_open: bool,
    pub trace_open: bool,

    pub set_default_freqs: bool,

    pub postfx_open: bool,
    pub sparkle_open: bool,

    pub bloom: f32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            file_dialog: FileDialog::new()
                .opening_mode(egui_file_dialog::OpeningMode::LastPickedDir)
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

            playback_mode: PlaybackMode::default(),
            stereo: Stereometer {
                filter_freq: 1.0,
                last_freq: 1.0,
                fs_color: Rgba::new(0, 255, 0, 255),
                mb_color: [
                    Rgba::new(255, 0, 0, 255),
                    Rgba::new(0, 255, 0, 255),
                    Rgba::new(0, 0, 255, 255),
                ],
                scale_factor: 290.0,
                point_size: 1.5,
                ..Default::default()
            },
            pos: [0.0; 2],

            import_open: false,

            gen_open: false,
            render_open: false,
            render_mode_options_open: false,
            stereo_kind_options_open: false,
            filtering_open: false,
            filter_mode_options_open: false,
            mode_open: false,
            color_open: false,
            visual_open: false,
            density_open: false,
            trace_open: false,

            set_default_freqs: true,

            postfx_open: false,
            sparkle_open: false,
            bloom: 0.4,
        }
    }
}
