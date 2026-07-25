#![allow(dead_code)]
use crate::{
    audio::{StereoFilter, audio_player::AudioPlayer},
    generators::{
        ChromaType, EnvelopeBank, FilterBank, GenKind, cymatic_field::CymaticField,
        fluidwave::ModSrc, oscilloscope::Oscilloscope, polar_patterns::PolarPatterns,
        rendering::EffectsCallback, stereometer::FilterMode,
    },
    traits::{ActiveGenerator, Generator, Labeled},
    ui::control_panel_widgets::{
        dropdown_row, mod_slider_row, section_header_submenu, slider_row, subheader_toggle_button,
    },
};
use biquad::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use eframe::{
    egui::{self, Pos2},
    egui_wgpu,
};

use crate::{
    Preset,
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
    pub use_hw_encoder: bool,
    pub total_frames: usize,
}
#[derive(Default)]
pub struct BoolStates {
    pub debug_file_loaded: bool,

    pub gen_kind_options_open: bool,
    pub gen_open: bool,
    pub render_open: bool,
    pub render_mode_options_open: bool,
    pub export_enabled: bool,

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
    pub show_file_options: bool,
    pub window_drag_tooltip_modal_open: bool,
    pub show_fullscreen_button: bool,
    pub show_settings: bool,
    pub show_export_resolution: bool,
    pub show_export_fps: bool,
    pub show_export_quality: bool,
    pub open_export_path_picker: bool,
    pub export_sample_idx: usize,
    pub show_export_modal: bool,
    pub start_render: bool,
    pub rendering: bool,
    pub export_canceled: bool,
}

pub struct AppState {
    pub playback_mode: PlaybackMode,
    pub gen_kind: GenKind,
    pub stereo: Stereometer,
    pub fwave: Fluidwave,
    pub osci: Oscilloscope,
    pub polar_pat: PolarPatterns,
    pub cymatics: CymaticField,

    pub stereometer_render_resources: Option<P2DRenderResources>,
    pub fluid_render_resources: Option<FluidRenderResources>,
    pub bloom_render_resources: Option<EffectsRenderResources>,
    pub output_render_resources: Option<OutputResources>,
    pub resources: egui_wgpu::CallbackResources,

    pub env_bank: EnvelopeBank,
    pub filterbank: FilterBank,

    pub preset_save_path: Option<PathBuf>,
    pub preset_load_path: Option<PathBuf>,

    pub window_drag_tooltip_modal_deadline: Option<Instant>,

    pub bool: BoolStates,

    // Export states
    pub export_path: Option<PathBuf>,
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
            GenKind::Stereometer => &mut self.stereo,
            GenKind::Fluidwave => &mut self.fwave,
            GenKind::Oscilloscope => &mut self.osci,
            GenKind::PolarPatterns => &mut self.polar_pat,
            GenKind::CymaticField => &mut self.cymatics,
        }
    }

    pub fn update_filters(&mut self, pl: Option<&AudioPlayer>) {
        let Some(p) = pl else {
            return;
        };
        let fb = &self.filterbank;
        let has_filters = fb.live_fs_filters.is_some()
            && fb.trace_fs_filters.is_some()
            && fb.live_mb_filters.is_some()
            && fb.trace_mb_filters.is_some();

        let Some(fp) = self.active_gen().filter_params() else {
            return;
        };
        if !has_filters {
            self.bool.set_default_freqs = true;
            let filters = Some((
                StereoFilter::from_coeffs_butterworth(Type::LowPass, 300., p.contents.sample_rate),
                StereoFilter::from_coeffs_butterworth(
                    Type::BandPass,
                    1000.,
                    p.contents.sample_rate,
                ),
                StereoFilter::from_coeffs_butterworth(
                    Type::HighPass,
                    3000.,
                    p.contents.sample_rate,
                ),
            ));
            self.filterbank.live_fs_filters = filters.clone();
            self.filterbank.trace_fs_filters = filters.clone();
            self.filterbank.live_mb_filters = filters.clone();
            self.filterbank.trace_mb_filters = filters;
        } else if fp.last_freq != fp.filter_freq {
            fp.last_freq = fp.filter_freq;
            let filter_freq = fp.filter_freq;
            self.bool.set_default_freqs = false;

            let mut livefs = std::mem::take(&mut self.filterbank.live_fs_filters).expect("safe");
            let mut tracefs = std::mem::take(&mut self.filterbank.trace_fs_filters).expect("safe");
            let fmode = self.active_gen().filter_params().expect("safe").filter_mode;
            match fmode {
                FilterMode::Off => (),
                FilterMode::Lpf => {
                    livefs.0 = StereoFilter::from_coeffs_butterworth(
                        Type::LowPass,
                        filter_freq,
                        p.contents.sample_rate,
                    );
                    tracefs.0 = StereoFilter::from_coeffs_butterworth(
                        Type::LowPass,
                        filter_freq,
                        p.contents.sample_rate,
                    );
                }
                FilterMode::Bpf => {
                    livefs.1 = StereoFilter::from_coeffs_butterworth(
                        Type::BandPass,
                        filter_freq,
                        p.contents.sample_rate,
                    );
                    tracefs.1 = StereoFilter::from_coeffs_butterworth(
                        Type::BandPass,
                        filter_freq,
                        p.contents.sample_rate,
                    );
                }
                FilterMode::Hpf => {
                    livefs.2 = StereoFilter::from_coeffs_butterworth(
                        Type::HighPass,
                        filter_freq,
                        p.contents.sample_rate,
                    );
                    tracefs.2 = StereoFilter::from_coeffs_butterworth(
                        Type::HighPass,
                        filter_freq,
                        p.contents.sample_rate,
                    );
                }
            }
            self.filterbank.live_fs_filters = Some(livefs);
            self.filterbank.trace_fs_filters = Some(tracefs);
        }
    }

    pub fn build_renderer_callback_params(&mut self, live: bool, fps: usize) -> GenCbParams {
        let mut stereo = std::mem::take(&mut self.stereo);
        let mut fwave = std::mem::take(&mut self.fwave);
        let mut osci = std::mem::take(&mut self.osci);
        let mut polar_pat = std::mem::take(&mut self.polar_pat);
        let mut cymatics = std::mem::take(&mut self.cymatics);
        let ret = match self.gen_kind {
            GenKind::Stereometer => stereo.get_gen_callback_params(self, live, fps),
            GenKind::Fluidwave => fwave.get_gen_callback_params(self, live, fps),
            GenKind::Oscilloscope => osci.get_gen_callback_params(self, live, fps),
            GenKind::PolarPatterns => polar_pat.get_gen_callback_params(self, live, fps),
            GenKind::CymaticField => cymatics.get_gen_callback_params(self, live, fps),
        };
        self.stereo = stereo;
        self.fwave = fwave;
        self.osci = osci;
        self.polar_pat = polar_pat;
        self.cymatics = cymatics;

        ret
    }

    pub fn build_effects_callback_params(&mut self) -> EffectsCallback {
        let fx = self.active_gen().post_fx();
        let env = |src: ModSrc, range: f32| -> f32 {
            self.env_bank.envelope_value_from_mod_src(src, range)
        };
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
        let (_gen_kind, stereo, fwave, osci, polar_pat, cymatics) = (
            preset.gen_kind,
            preset.stereometer,
            preset.fluidwave,
            preset.oscilloscope,
            preset.polar_patterns,
            preset.cymatics,
        );
        Self {
            gen_kind: GenKind::default(),
            stereometer_render_resources: None,
            fluid_render_resources: None,
            bloom_render_resources: None,
            output_render_resources: None,
            resources: egui_wgpu::CallbackResources::new(),

            filterbank: FilterBank::default(),
            env_bank: EnvelopeBank::default(),

            export_path: None,
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
            polar_pat,
            cymatics,

            window_drag_tooltip_modal_deadline: None,
            preset_save_path: None,
            preset_load_path: None,

            cur_frame_idx: 0,
            bool: BoolStates {
                show_fullscreen_button: true,
                dark_mode: true,
                export_enabled: true,
                ..BoolStates::default()
            },
        }
    }
}
