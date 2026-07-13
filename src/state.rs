#![allow(dead_code)]
use crate::{
    generators::{
        ChromaType, Envelope, fluidwave::ModSrc, oscilloscope::Oscilloscope,
        rendering::EffectsCallback,
    },
    traits::{ActiveGenerator, Generator, Labeled},
    ui::control_panel_widgets::{
        dropdown_row, mod_slider_row, section_header_submenu, slider_row, subheader_toggle_button,
    },
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use eframe::{
    egui::{self, Align2, Pos2, vec2},
    egui_wgpu,
};
use egui_file_dialog::{self as fd, FileDialog};

use crate::{
    GenKindLabel, Preset,
    generators::{
        fluidwave::Fluidwave,
        rendering::{
            EffectsRenderResources, FluidRenderResources, GenCbParams, OutputResources,
            P2DRenderResources,
        },
        stereometer::Stereometer,
    },
    labeled_enum,
};

pub const DAMP_FACTOR: f32 = 1.25;
pub const MAX_CHROMA_SHIFT: f32 = 0.2;
pub const MAX_VIGNETTE: f32 = 1.0;
pub const MAX_BLOOM: f32 = 10.0;

#[derive(Default)]
pub enum PlaybackMode {
    #[default]
    Loop,
    Once,
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
#[derive(Default)]
pub struct BoolStates {
    pub gen_kind_options_open: bool,
    pub gen_open: bool,
    pub render_open: bool,
    pub render_mode_options_open: bool,

    pub energy_transfer_mode_options_open: bool,
    pub force_direction_options_open: bool,
    pub color_mode_options_open: bool,
    pub gradient_formula_options_open: bool,
    pub color_arrangement_options_open: bool,
    pub envelope_follower_open: bool,
    pub env_a_open: bool,
    pub env_b_open: bool,
    pub env_c_open: bool,
    pub env_d_open: bool,

    pub bloom_open: bool,
    pub vignette_open: bool,
    pub chroma_open: bool,
    pub chroma_type_open: bool,

    pub stereo_kind_options_open: bool,
    pub filtering_open: bool,
    pub set_default_freqs: bool,
    pub filter_mode_options_open: bool,
    pub mode_open: bool,
    pub color_open: bool,
    pub visual_open: bool,
    pub density_open: bool,
    pub trace_open: bool,
    pub postfx_open: bool,

    pub bloom_mod_open: bool,
    pub vignette_mod_open: bool,
    pub chroma_mod_open: bool,
    pub mod_src_open: bool,
    pub import_open: bool,

    pub dark_mode: bool,
    pub advanced_mode: bool,
    pub fullscreen: bool,

    pub save_preset: bool,
    pub load_preset: bool,
    pub show_preset_options: bool,
    pub show_preset_save_modal: bool,
    pub show_preset_load_modal: bool,
    pub open_preset_save_file_picker: bool,
    pub open_preset_load_file_picker: bool,
    pub picked_preset_save_dir: bool,
    pub picked_preset_load_file: bool,
    pub show_file_options: bool,
    pub window_drag_tooltip_modal_open: bool,
    pub show_fullscreen_button: bool,
    pub show_settings: bool,
    pub show_export_resolution: bool,
    pub show_export_fps: bool,
    pub show_export_quality: bool,
    pub export_sample_idx: usize,
    pub show_export_modal: bool,
    pub start_render: bool,
    pub rendering: bool,
    pub export_canceled: bool,
}

pub struct AppState {
    pub audio_file_dialog: FileDialog,
    pub preset_file_dialog: FileDialog,
    pub playback_mode: PlaybackMode,
    pub gen_kind: GenKindLabel,
    pub stereo: Stereometer,
    pub fwave: Fluidwave,
    pub osci: Oscilloscope,
    pub env_a: Option<Envelope>,
    pub env_b: Option<Envelope>,
    pub env_c: Option<Envelope>,
    pub env_d: Option<Envelope>,
    pub stereometer_render_resources: Option<P2DRenderResources>,
    pub fluid_render_resources: Option<FluidRenderResources>,
    pub bloom_render_resources: Option<EffectsRenderResources>,
    pub output_render_resources: Option<OutputResources>,
    pub resources: egui_wgpu::CallbackResources,

    pub preset_save_path: Option<PathBuf>,
    pub preset_load_path: Option<PathBuf>,
    pub preset_name: String,

    pub window_drag_tooltip_modal_deadline: Option<Instant>,

    pub bool: BoolStates,

    // Export states
    pub export_config: ExportConfig,
    pub writer_handle: Option<std::thread::JoinHandle<()>>,
    pub logger_handle: Option<std::thread::JoinHandle<()>>,
    pub export_tx: Option<flume::Sender<Vec<u8>>>,
    pub prev_export_timestamp: Option<Instant>,
    pub export_elapsed_time: Option<Duration>,
    pub cur_frame_idx: usize,
}

impl AppState {
    pub fn active_gen(&mut self) -> &mut dyn ActiveGenerator {
        match self.gen_kind {
            GenKindLabel::Stereometer => &mut self.stereo,
            GenKindLabel::Fluidwave => &mut self.fwave,
            GenKindLabel::Oscilloscope => &mut self.osci,
        }
    }
    pub fn envelope_value_from_mod_src(&self, src: ModSrc, range: f32) -> f32 {
        let (Some(a), Some(b), Some(c), Some(d)) =
            (&self.env_a, &self.env_b, &self.env_c, &self.env_d)
        else {
            return 0.0;
        };
        match src {
            ModSrc::None => 0.0,
            ModSrc::EnvA => a.envelope(range),
            ModSrc::EnvB => b.envelope(range),
            ModSrc::EnvC => c.envelope(range),
            ModSrc::EnvD => d.envelope(range),
        }
    }
    pub fn get_envelope(&self, src: ModSrc) -> Option<&Envelope> {
        match src {
            ModSrc::None => None,
            ModSrc::EnvA => self.env_a.as_ref(),
            ModSrc::EnvB => self.env_b.as_ref(),
            ModSrc::EnvC => self.env_c.as_ref(),
            ModSrc::EnvD => self.env_d.as_ref(),
        }
    }

    pub fn build_renderer_callback_params(&mut self, live: bool, fps: usize) -> GenCbParams {
        let mut stereo = std::mem::take(&mut self.stereo);
        let mut fwave = std::mem::take(&mut self.fwave);
        let mut osci = std::mem::take(&mut self.osci);
        let ret = match self.gen_kind {
            GenKindLabel::Stereometer => stereo.into_gen_callback_params(self, live, fps),
            GenKindLabel::Fluidwave => fwave.into_gen_callback_params(self, live, fps),
            GenKindLabel::Oscilloscope => osci.into_gen_callback_params(self, live, fps),
        };
        self.stereo = stereo;
        self.fwave = fwave;
        self.osci = osci;

        ret
    }

    pub fn build_effects_callback_params(&mut self) -> EffectsCallback {
        let fx = self.active_gen().post_fx();
        let env = |src: ModSrc, range: f32| -> f32 { self.envelope_value_from_mod_src(src, range) };
        let (brng, vrng, csh_rng) = (fx.bloom_range, fx.vignette_range, fx.chroma_shift_range);
        let (bsrc, vsrc, csh_src) = (
            fx.bloom_mod_src,
            fx.vignette_mod_src,
            fx.chroma_shift_mod_src,
        );
        let (bloom_amt, vignette, chroma_shift, chroma_blur, chroma_type) = (
            (fx.bloom + env(bsrc, brng) * MAX_BLOOM).clamp(0.0, MAX_BLOOM),
            (fx.vignette + env(vsrc, vrng) * MAX_VIGNETTE).clamp(0.0, MAX_VIGNETTE),
            (fx.chroma_shift + env(csh_src, csh_rng) * MAX_CHROMA_SHIFT)
                .clamp(0.0, MAX_CHROMA_SHIFT),
            fx.chroma_blur,
            fx.chroma_type,
        );
        let (use_bloom, use_vignette, use_chroma) = (fx.use_bloom, fx.use_vignette, fx.use_chroma);

        EffectsCallback {
            top_left: Pos2::ZERO,
            use_bloom,
            bloom_amt,
            use_vignette,
            vignette,
            use_chroma,
            chroma_shift,
            chroma_blur,
            chroma_type,
        }
    }

    pub fn draw_post_fx(&mut self, ui: &mut egui::Ui) {
        let mut open = std::mem::take(&mut self.bool);
        let fx = self.active_gen().post_fx_mut();

        let rect = section_header_submenu(ui, "BLOOM", &mut open.bloom_open).rect;
        subheader_toggle_button(ui, &rect, &mut fx.use_bloom);
        if open.bloom_open {
            mod_slider_row(
                ui,
                "BLOOM",
                &mut fx.bloom,
                0.0,
                MAX_BLOOM,
                1,
                &mut fx.bloom_mod_src,
                &mut open.bloom_mod_open,
                &mut open.mod_src_open,
                &mut fx.bloom_range,
                false,
            );
        }
        let rect = section_header_submenu(ui, "VIGNETTE", &mut open.vignette_open).rect;
        subheader_toggle_button(ui, &rect, &mut fx.use_vignette);
        if open.vignette_open {
            mod_slider_row(
                ui,
                "VIGNETTE",
                &mut fx.vignette,
                0.0,
                MAX_VIGNETTE,
                2,
                &mut fx.vignette_mod_src,
                &mut open.vignette_mod_open,
                &mut open.mod_src_open,
                &mut fx.vignette_range,
                false,
            );
        }
        let rect = section_header_submenu(ui, "CHROMA", &mut open.chroma_open).rect;
        subheader_toggle_button(ui, &rect, &mut fx.use_chroma);
        if open.chroma_open {
            mod_slider_row(
                ui,
                "CHROMA",
                &mut fx.chroma_shift,
                0.0,
                MAX_CHROMA_SHIFT,
                3,
                &mut fx.chroma_shift_mod_src,
                &mut open.chroma_mod_open,
                &mut open.mod_src_open,
                &mut fx.chroma_shift_range,
                false,
            );
            slider_row(ui, "BLUR", &mut fx.chroma_blur, 0.0, 20.0, 0, false);
            dropdown_row(
                ui,
                "TYPE",
                &mut fx.chroma_type,
                ChromaType::ALL,
                &mut open.chroma_type_open,
                true,
            );
        }

        self.bool = open;
    }
}

impl Default for AppState {
    fn default() -> Self {
        let fstr = std::fs::read_to_string("presets/default.json").unwrap_or_default();
        let preset: Preset = serde_json::from_str(&fstr).unwrap_or_default();
        let (stereo, fwave, osci) = (preset.stereometer, preset.fluidwave, preset.oscilloscope);
        Self {
            audio_file_dialog: FileDialog::new()
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

            preset_file_dialog: FileDialog::new()
                .opening_mode(egui_file_dialog::OpeningMode::LastPickedDir)
                .show_left_panel(true)
                .show_pinned_folders(true)
                .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0)),
            gen_kind: GenKindLabel::Oscilloscope,
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

            playback_mode: PlaybackMode::default(),
            stereo,
            fwave,
            osci,
            env_a: None,
            env_b: None,
            env_c: None,
            env_d: None,

            window_drag_tooltip_modal_deadline: None,
            preset_save_path: None,
            preset_load_path: None,
            preset_name: String::default(),

            cur_frame_idx: 0,
            bool: BoolStates {
                show_fullscreen_button: true,
                dark_mode: true,
                ..BoolStates::default()
            },
        }
    }
}
