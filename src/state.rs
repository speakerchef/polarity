#![allow(dead_code)]
use crate::{
    generators::{Envelope, fluidwave::ModSrc, rendering::EffectsCallback},
    traits::{ActiveGenerator, Labeled},
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use eframe::{
    egui::{Align2, Pos2, vec2},
    egui_wgpu,
};
use egui_file_dialog::{self as fd, FileDialog};

use crate::{
    GenKindLabel, Preset,
    generators::{
        fluidwave::Fluidwave,
        rendering::{
            EffectsRenderResources, FluidCbParams, FluidRenderResources, GenCbParams,
            OutputResources, StereoCbParams, StereometerRenderResources,
        },
        stereometer::Stereometer,
    },
    labeled_enum,
    ui::canvas::{MIN_SUBSTEP_DIV, SUBSTEP_DIV, TARGET_DT},
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
    pub env_a: Option<Envelope>,
    pub env_b: Option<Envelope>,
    pub env_c: Option<Envelope>,
    pub env_d: Option<Envelope>,
    pub stereometer_render_resources: Option<StereometerRenderResources>,
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
        match self.gen_kind {
            GenKindLabel::Stereometer => {
                const MAX_POINT_SIZE: f32 = 0.01;
                let s = &self.stereo;
                let env = |src: ModSrc, range: f32| -> f32 {
                    self.envelope_value_from_mod_src(src, range)
                };
                let point_size =
                    s.point_size + env(s.point_size_mod_src, s.point_size_rng) * MAX_POINT_SIZE;

                let s = &mut self.stereo;
                GenCbParams::Stereo(StereoCbParams {
                    render_mode: s.render_mode,
                    point_size,
                    live_pos: std::mem::take(&mut s.live_buffer),
                    trace_pos: s.trace_buffer.clone().into(),

                    live_low_pos: std::mem::take(&mut s.live_low_buffer),
                    live_mid_pos: std::mem::take(&mut s.live_mid_buffer),
                    live_high_pos: std::mem::take(&mut s.live_high_buffer),
                    trace_low_pos: s.trace_low_buffer.clone().into(),
                    trace_mid_pos: s.trace_mid_buffer.clone().into(),
                    trace_high_pos: s.trace_high_buffer.clone().into(),

                    fs_color: s.fs_color.into(),
                    lb_color: s.mb_color[0].into(),
                    mb_color: s.mb_color[1].into(),
                    hb_color: s.mb_color[2].into(),
                })
            }
            GenKindLabel::Fluidwave => {
                const MAX_FRAME_TIME: f32 = 1. / 12. / SUBSTEP_DIV;
                const MAX_LUMINANCE_FLOOR: f32 = 100.0;

                let env = |src: ModSrc, range: f32| -> f32 {
                    self.envelope_value_from_mod_src(src, range)
                };
                let f = &self.fwave;
                let luminance_floor = f.luminance_floor
                    + env(f.luminance_floor_mod_src, f.luminance_floor_rng) * MAX_LUMINANCE_FLOOR;

                let f = &mut self.fwave;
                let pressure_multiplier = f.pressure_multiplier
                    - if f.envelope_pressure_link {
                        400.0
                            * self
                                .env_a
                                .as_ref()
                                .expect("unreachable without envelope")
                                .envelope(f.env_range)
                                .powf(DAMP_FACTOR)
                    } else {
                        0.0
                    };

                let sim_speed_scale = 100.0 / f.sim_speed.max(1.0);
                let sim_speed = (sim_speed_scale * SUBSTEP_DIV) /* higher == slower */
                    .clamp(MIN_SUBSTEP_DIV, 100.0)
                    .round();

                let now = Instant::now();
                let frame_time = if live {
                    now.duration_since(f.last_frame).as_secs_f32() / sim_speed
                } else {
                    1. / fps as f32 / sim_speed
                };
                f.frame_time_accumulator += frame_time.min(MAX_FRAME_TIME);

                let active_envelope = self.env_a.as_ref().expect("unreachable without envelope");
                let params = GenCbParams::Fwave(FluidCbParams {
                    color_mode: f.color_mode,
                    uniform_color: f.uniform_color,
                    //fwave will use env A as its driver
                    particle_pos: active_envelope.envelope(f.env_range),
                    frame_time_accumulator: f.frame_time_accumulator,
                    gravity: f.gravity,
                    pressure_multiplier,
                    target_density: f.target_density,
                    smoothing_radius: f.smoothing_radius,
                    edge_damping_factor: f.edge_damping_factor,
                    near_pressure_multiplier: f.near_pressure_multiplier,
                    viscosity_amount: f.viscosity_amount,
                    point_size: f.point_size,
                    energy_transfer_mode: f.energy_transfer_mode,
                    force_direction: f.force_direction,
                    color_arrangement: f.color_arrangement,
                    color_invert: f.color_invert,
                    luminance_mode: f.luminance_mode,
                    luminance_floor,
                    substeps: sim_speed,
                });
                f.last_frame = now;
                f.frame_time_accumulator %= TARGET_DT; // leftover frametime
                params
            }
        }
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
}

impl Default for AppState {
    fn default() -> Self {
        let fstr = std::fs::read_to_string("presets/default.json").unwrap_or_default();
        let preset: Preset = serde_json::from_str(&fstr).unwrap_or_default();
        let (stereo, fwave) = (preset.stereometer, preset.fluidwave);
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
            gen_kind: GenKindLabel::Fluidwave,
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
                ..BoolStates::default()
            },
        }
    }
}
