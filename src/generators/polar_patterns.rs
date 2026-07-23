use std::{collections::VecDeque, f32::consts::TAU, sync::Arc};

use eframe::egui::{Pos2, pos2};
use rustfft::{Fft, FftPlanner};

use crate::{
    Rgba,
    audio::audio_player::AudioPlayer,
    generators::{
        PostFx, fft_max_frequency_bin,
        fluidwave::ModSrc,
        rendering::{GenCbParams, Particle2DCbParams},
        stereometer::ParticleRenderMode,
    },
    labeled_enum,
    traits::{ActiveGenerator, Generator, Labeled, ParamAccess},
    ui::control_panel_widgets::{
        dropdown_row, section_header_submenu, slider_row, static_label, toggle_button_row,
    },
};

labeled_enum!(PatternKind {
    Unipolar => "Unipolar",
    Bipolar => "Bipolar",
}, Bipolar);

impl Labeled for PatternKind {
    fn text(self) -> &'static str {
        self.label()
    }
}

pub struct FftPass {
    pub fft: Arc<dyn Fft<f32>>,
}
impl Clone for FftPass {
    fn clone(&self) -> Self {
        fft_pass_default()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PolarPatterns {
    kind: PatternKind,
    fs_color: Rgba,
    efx: PostFx,

    use_3d: bool,
    use_rotation: bool,
    angle: f32,
    pitch: f32,
    rot_speed: f32,
    scale: f32,
    camera_z: f32,
    upsample_factor: f32,

    point_size: f32,
    point_size_mod_src: ModSrc,
    point_size_rng: f32,
    point_size_mod_open: bool,

    #[serde(skip)]
    live_buffer: Vec<Pos2>,
    #[serde(skip)]
    trace_buffer: VecDeque<Pos2>,
    #[serde(skip, default = "fft_pass_default")]
    fft_pass: FftPass,
}

const FFT_WINDOW: usize = 8192;

fn fft_pass_default() -> FftPass {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FFT_WINDOW);
    FftPass { fft }
}

impl Default for PolarPatterns {
    fn default() -> Self {
        Self {
            kind: PatternKind::default(),
            fs_color: Rgba {
                r: 255.0,
                g: 93.0,
                b: 0.0,
                a: 255.0,
            },
            live_buffer: Default::default(),
            trace_buffer: Default::default(),
            fft_pass: fft_pass_default(),
            use_3d: true,
            use_rotation: true,
            angle: 0.0,
            pitch: 0.0,
            rot_speed: 0.4,
            scale: 0.8,
            camera_z: 10.0,
            upsample_factor: 8.0,

            efx: PostFx {
                use_bloom: true,
                bloom: 1.5,
                bloom_mod_src: ModSrc::EnvB,
                bloom_range: 20.0,
                use_vignette: true,
                use_chroma: true,
                chroma_shift_mod_src: ModSrc::EnvB,
                chroma_shift_range: 100.0,
                chroma_blur: 4.0,
                chroma_type: super::ChromaType::Radial,
                ..Default::default()
            },
            point_size: 0.0015,
            point_size_mod_src: ModSrc::None,
            point_size_rng: 0.0,
            point_size_mod_open: false,
        }
    }
}

impl PolarPatterns {
    fn draw(&mut self, pl: &AudioPlayer, export_sample_idx: Option<usize>) {
        let num_ch = pl.contents.num_channels as usize;
        let sr = pl.contents.sample_rate as usize;
        let s = &pl.contents.samples;
        let start_idx =
            export_sample_idx.unwrap_or_else(|| (pl.position().as_secs_f64() * sr as f64) as usize);
        let gap = FFT_WINDOW - 1;
        let end_idx = start_idx + gap + 1;
        let window = s
            .get(start_idx * num_ch..end_idx * num_ch)
            .unwrap_or_default();

        let n = window.len();
        let polar_speed = self.get_polar_speed(window, num_ch, sr);
        let buf = (0..n)
            .map(|i| {
                let l = window.get(i * num_ch).unwrap_or(&0_f32);
                let r = window.get(i * num_ch + 1).unwrap_or(&0_f32);

                let time = (start_idx + i) as f32 / sr as f32;
                let theta = (TAU * polar_speed * time) % TAU;

                let (x, y, z) = (
                    l * theta.sin(),
                    if matches!(self.kind, PatternKind::Unipolar) {
                        l
                    } else {
                        r
                    } * theta.cos(),
                    (l - r),
                );

                if self.use_3d {
                    let pos =
                        start_idx as f32 / s.len() as f32 * pl.contents.duration.as_secs_f32();
                    let angle = if self.use_rotation {
                        pos * self.rot_speed * TAU
                    } else {
                        self.angle / 360.0 * TAU
                    };

                    let x1 = x * angle.cos() - z * angle.sin();
                    let z1 = -x * angle.sin() + z * angle.cos();
                    let y1 = y * self.pitch.cos() - z1 * self.pitch.sin();

                    let depth = (self.camera_z - z1).max(0.1);
                    let perspective = self.camera_z / depth;

                    pos2(x1, y1) * self.scale * perspective
                } else {
                    pos2(x, y) * 0.99
                }
            })
            .collect::<Vec<Pos2>>();

        self.live_buffer = self.upsample(&buf);
    }

    fn get_polar_speed(&self, window: &[f32], num_ch: usize, sr: usize) -> f32 {
        let freq = fft_max_frequency_bin(self.fft_pass.fft.clone(), window, num_ch, sr);
        (freq * 2.0).round()
    }

    fn upsample(&self, buf: &[Pos2]) -> Vec<Pos2> {
        let us_factor = self.upsample_factor;
        let mut prev = buf.first().unwrap_or(&Pos2::ZERO);
        buf.iter()
            .flat_map(|pos| {
                let dx = pos.x - prev.x;
                let dy = pos.y - prev.y;
                let ix = dx / us_factor;
                let iy = dy / us_factor;

                let (mut cx, mut cy) = (prev.x, prev.y);
                let v = (0..us_factor as usize)
                    .map(|_| {
                        cx += ix;
                        cy += iy;
                        pos2(cx, cy)
                    })
                    .collect::<Vec<Pos2>>();
                prev = pos;
                v
            })
            .collect()
    }
}

impl ActiveGenerator for PolarPatterns {}
impl Generator for PolarPatterns {
    fn prepare(
        &mut self,
        _f: &mut super::FilterBank,
        pl: &crate::audio::audio_player::AudioPlayer,
        export_sample_idx: Option<usize>,
    ) {
        self.draw(pl, export_sample_idx);
    }

    fn into_gen_callback_params(
        &mut self,
        _st: &crate::state::AppState,
        _live: bool,
        _fps: usize,
    ) -> super::rendering::GenCbParams {
        GenCbParams::Particle2D(Particle2DCbParams {
            render_mode: ParticleRenderMode::FullSpectrum,
            point_size: self.point_size,
            live_pos: std::mem::take(&mut self.live_buffer),
            trace_pos: self.trace_buffer.clone(),
            fs_color: self.fs_color.into(),
            ..Default::default()
        })
    }

    fn draw_render_menu(&mut self, ui: &mut eframe::egui::Ui, open: &mut crate::state::BoolStates) {
        dropdown_row(
            ui,
            "MODE",
            &mut self.kind,
            PatternKind::ALL,
            &mut open.mode_open,
            false,
        );
    }

    fn draw_color_menu(&mut self, ui: &mut eframe::egui::Ui, open: &mut crate::state::BoolStates) {
        section_header_submenu(ui, "COLOR", &mut open.color_open);
        if open.color_open {
            slider_row(ui, "RED", &mut self.fs_color.r, 0.0, 255.0, 0, false);
            slider_row(ui, "GREEN", &mut self.fs_color.g, 0.0, 255.0, 0, false);
            slider_row(ui, "BLUE", &mut self.fs_color.b, 0.0, 255.0, 0, false);
        }
    }

    fn draw_visual_menu(&mut self, ui: &mut eframe::egui::Ui, open: &mut crate::state::BoolStates) {
        section_header_submenu(ui, "VISUAL", &mut open.visual_open);
        if open.visual_open {
            toggle_button_row(ui, "3D", &mut self.use_3d, false);
            if self.use_3d {
                toggle_button_row(ui, "ROTATION", &mut self.use_rotation, false);
            }

            if self.use_3d {
                if !self.use_rotation {
                    slider_row(ui, "ANGLE", &mut self.angle, 0.0, 360.0, 0, false);
                }
                slider_row(ui, "DISTANCE", &mut self.camera_z, 1.0, 10.0, 1, false);
                slider_row(ui, "SCALE", &mut self.scale, 0.1, 1.0, 2, false);
            }

            slider_row(
                ui,
                "UPSAMPLE",
                &mut self.upsample_factor,
                1.0,
                32.0,
                0,
                false,
            );
            self.upsample_factor = self.upsample_factor.round();

            slider_row(
                ui,
                "POINT SIZE",
                &mut self.point_size,
                0.0005,
                0.01,
                4,
                false,
            );

            if open.advanced_mode && self.use_3d {
                static_label(ui, "ADVANCED");
                slider_row(ui, "SPEED", &mut self.rot_speed, 0.01, 1.0, 2, false);
                slider_row(ui, "PITCH", &mut self.pitch, 0.0, 1.0, 2, false);
            }
        }
    }
}

impl ParamAccess for PolarPatterns {
    fn post_fx(&self) -> PostFx {
        self.efx
    }
    fn post_fx_mut(&mut self) -> &mut PostFx {
        &mut self.efx
    }
    fn filter_params(&mut self) -> Option<&mut super::FilterParams> {
        None
    }
}
