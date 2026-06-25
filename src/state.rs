#![allow(dead_code)]
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use eframe::{
    egui::{Align2, vec2},
    egui_wgpu,
};
use egui_file_dialog::{self as fd, FileDialog};

use crate::{
    GeneratorKind,
    generators::{
        rendering::{
            BloomRenderResources, FluidRenderResources, OutputResources, StereometerRenderResources,
        },
        stereometer::Stereometer,
    },
    labeled_enum,
};

#[derive(Default)]
pub enum PlaybackMode {
    #[default]
    Loop,
    Once,
}

pub trait Labeled: Copy + PartialEq {
    fn text(self) -> &'static str;
}

labeled_enum!(
    Resolution {
        P480 =>"480p",
        P720 =>"720p",
        P1080=>"1080p",
        P1200=>"1200p",
        P1440=>"1440p",
        P1600=>"1600p",
        P2160=>"2160p",
    },
    P1080
);

impl Resolution {
    pub fn value(self) -> (u32, u32) {
        match self {
            Resolution::P480 => (480, 480),
            Resolution::P720 => (720, 720),
            Resolution::P1080 => (1080, 1080),
            Resolution::P1200 => (1200, 1200),
            Resolution::P1440 => (1440, 1440),
            Resolution::P1600 => (1600, 1600),
            Resolution::P2160 => (2160, 2160),
        }
    }
}

impl Labeled for Resolution {
    fn text(self) -> &'static str {
        self.label()
    }
}

labeled_enum!(Fps {
    FPS24 => "24",
    FPS30 => "30",
    FPS45 => "45",
    FPS60 => "60",
}, FPS45);

impl Fps {
    pub fn value(self) -> usize {
        match self {
            Fps::FPS24 => 24,
            Fps::FPS30 => 30,
            Fps::FPS45 => 45,
            Fps::FPS60 => 60,
        }
    }
}

impl Labeled for Fps {
    fn text(self) -> &'static str {
        self.label()
    }
}

labeled_enum!(ExportQuality {
    Worst => "Worst (Fastest)",
    Good => "Good",
    Best => "Best (Very Slow)",
}, Good);

impl ExportQuality {
    pub fn value(self) -> usize {
        match self {
            ExportQuality::Worst => 20,
            ExportQuality::Good => 18,
            ExportQuality::Best => 14,
        }
    }
}

impl Labeled for ExportQuality {
    fn text(self) -> &'static str {
        self.label()
    }
}

#[derive(Default)]
pub struct ExportConfig {
    pub resolution: Resolution,
    pub frame_rate: Fps,
    pub quality: ExportQuality,
    pub total_frames: usize,
}

pub struct Fluidwave {
    pub last_frame: Instant,
    pub gravity: f32,
    pub pressure_multiplier: f32,
    pub target_density: f32,
    pub smoothing_radius: f32,
    pub near_pressure_multiplier: f32,
    pub viscosity_strength: f32,
}

pub struct AppState {
    pub file_dialog: FileDialog,
    pub dir_dialog: FileDialog,
    pub playback_mode: PlaybackMode,
    pub gen_kind: GeneratorKind,
    pub stereo: Stereometer,
    pub fwave: Fluidwave,
    pub pos: [f32; 2],

    pub stereometer_render_resources: Option<StereometerRenderResources>,
    pub fluid_render_resources: Option<FluidRenderResources>,
    pub bloom_render_resources: Option<BloomRenderResources>,
    pub output_render_resources: Option<OutputResources>,
    pub resources: egui_wgpu::CallbackResources,

    // Export states
    pub export_config: ExportConfig,
    pub show_export_resolution: bool,
    pub show_export_fps: bool,
    pub show_export_quality: bool,
    pub cur_frame_idx: usize,
    pub export_sample_idx: usize,
    pub show_export_modal: bool,
    pub start_render: bool,
    pub rendering: bool,
    pub export_canceled: bool,
    pub writer_handle: Option<std::thread::JoinHandle<()>>,
    pub logger_handle: Option<std::thread::JoinHandle<()>>,
    pub export_tx: Option<flume::Sender<Vec<u8>>>,
    pub prev_export_timestamp: Option<Instant>,
    pub export_elapsed_time: Option<Duration>,

    pub dark_mode: bool,
    pub fullscreen: bool,
    pub import_open: bool,

    pub save_preset: bool,
    pub load_preset: bool,
    pub show_preset_options: bool,
    pub show_preset_save_modal: bool,
    pub show_preset_load_modal: bool,
    pub open_preset_save_file_picker: bool,
    pub open_preset_load_file_picker: bool,
    pub picked_preset_save_dir: bool,
    pub picked_preset_load_file: bool,
    pub file_picked: bool,
    pub preset_save_path: Option<PathBuf>,
    pub preset_load_path: Option<PathBuf>,
    pub preset_name: String,

    pub show_file_options: bool,
    pub window_drag_tooltip_modal_deadline: Option<Instant>,
    pub window_drag_tooltip_modal_open: bool,
    pub show_fullscreen_button: bool,
    pub show_settings: bool,

    pub gen_kind_options_open: bool,
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
    pub bloom_open: bool,

    pub env_follower_last_sample: f32,
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

            dir_dialog: FileDialog::new()
                .opening_mode(egui_file_dialog::OpeningMode::LastPickedDir)
                .show_left_panel(true)
                .show_pinned_folders(true)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0)),

            gen_kind: GeneratorKind::Fluidwave,
            show_fullscreen_button: true,
            stereometer_render_resources: None,
            fluid_render_resources: None,
            bloom_render_resources: None,
            output_render_resources: None,
            resources: egui_wgpu::CallbackResources::new(),

            export_config: ExportConfig::default(),
            writer_handle: None,
            logger_handle: None,
            export_tx: None,
            prev_export_timestamp: None,
            export_elapsed_time: None,
            show_export_resolution: false,
            show_export_fps: false,
            show_export_quality: false,
            cur_frame_idx: 0,
            export_sample_idx: 0,
            show_export_modal: false,
            start_render: false,
            rendering: false,
            export_canceled: false,

            playback_mode: PlaybackMode::default(),
            stereo: Stereometer::default(),
            fwave: Fluidwave {
                last_frame: Instant::now(),
                // gravity: 0.0,
                // pressure_multiplier: 290.0,
                // target_density: 2300.0,
                // smoothing_radius: 0.1,
                // near_pressure_multiplier: 10.0,
                // viscosity_strength: 0.001,
                // gravity: -20.0,
                // pressure_multiplier: 210.0,
                // target_density: 5550.0,
                // smoothing_radius: 0.08,
                // near_pressure_multiplier: 10.0,
                // viscosity_strength: 0.02,
                gravity: 0.0,
                pressure_multiplier: 152.0,
                target_density: 0.0,
                smoothing_radius: 0.1,
                near_pressure_multiplier: 7.1,
                viscosity_strength: 0.01,
            },
            pos: [0.0; 2],

            dark_mode: true,
            fullscreen: false,
            import_open: false,
            show_file_options: false,
            show_preset_options: false,
            window_drag_tooltip_modal_deadline: None,
            window_drag_tooltip_modal_open: false,

            save_preset: false,
            load_preset: false,
            show_preset_save_modal: false,
            show_preset_load_modal: false,
            picked_preset_save_dir: false,
            picked_preset_load_file: false,
            open_preset_save_file_picker: false,
            file_picked: false,
            open_preset_load_file_picker: false,
            preset_save_path: None,
            preset_load_path: None,
            preset_name: String::default(),

            show_settings: false,

            gen_kind_options_open: false,
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
            bloom_open: false,

            env_follower_last_sample: 0.0,
        }
    }
}
