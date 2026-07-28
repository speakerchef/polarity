use std::{collections::VecDeque, f32::consts::TAU};

use eframe::egui::{Pos2, lerp, pos2};

use crate::{
    Rgba,
    generators::{
        EnvelopeBank, FftPass, FftWindow, FilterBank, FilterParams, MAX_POINT_SIZE, MIN_POINT_SIZE,
        PostFx, fft_max_frequency_bin,
        fluidwave::ModSrc,
        positional_interp_upsampling,
        rendering::{GenCbParams, Particle2DCbParams},
        stereometer::{FilterMode, ParticleRenderMode},
    },
    labeled_enum,
    traits::{ActiveGenerator, AudioSrc, Generator, Labeled, ParamAccess},
    ui::control_panel_widgets::{
        dropdown_row, section_header_submenu, slider_row, static_label, toggle_button_row,
    },
};

labeled_enum!(PatternKind {
    Unipolar => "Unipolar",
    Bipolar => "Bipolar",
}, Unipolar);

impl Labeled for PatternKind {
    fn text(&self) -> &'static str {
        self.label()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PolarPatterns {
    pub filter_params: FilterParams,

    kind: PatternKind,
    fs_color: Rgba,
    efx: PostFx,

    use_3d: bool,
    use_rotation: bool,
    angle: f32,
    camera_z: f32,
    pitch: f32,
    rot_speed: f32,
    scale: f32,
    upsample_factor: f32,

    fft_window: FftWindow,

    point_size: f32,
    point_size_mod_src: ModSrc,
    point_size_rng: f32,
    point_size_mod_open: bool,

    #[serde(skip)]
    live_buffer: Vec<Pos2>,
    #[serde(skip)]
    trace_buffer: VecDeque<Pos2>,
    #[serde(skip)]
    fft_pass: FftPass,
}

impl Default for PolarPatterns {
    fn default() -> Self {
        Self {
            kind: PatternKind::default(),
            // fs_color: Rgba {
            //     r: 64.0,
            //     g: 93.0,
            //     b: 65.0,
            //     a: 255.0,
            // },
            fs_color: Rgba {
                r: 76.0,
                g: 113.0,
                b: 88.0,
                a: 255.0,
            },
            live_buffer: Default::default(),
            trace_buffer: Default::default(),
            fft_pass: FftPass::default(),
            use_3d: false,
            use_rotation: true,
            angle: 0.0,
            camera_z: 3.0,
            pitch: 0.3,
            rot_speed: 0.1,
            scale: 0.8,
            upsample_factor: 4.0,

            fft_window: FftWindow::W8192,

            efx: PostFx {
                use_bloom: true,
                bloom: 1.5,
                bloom_mod_src: ModSrc::EnvB,
                bloom_range: 80.0,
                use_vignette: true,
                use_chroma: true,
                chroma_shift_mod_src: ModSrc::EnvB,
                chroma_shift_range: 100.0,
                chroma_blur: 4.0,
                chroma_type: super::ChromaType::Radial,
                ..Default::default()
            },
            filter_params: FilterParams {
                filter_mode: FilterMode::Lpf,
                last_freq: 0.0,
                filter_freq: 450.,
            },
            point_size: 0.0023,
            point_size_mod_src: ModSrc::None,
            point_size_rng: 0.0,
            point_size_mod_open: false,
        }
    }
}

impl PolarPatterns {
    fn draw(&mut self, input: &dyn AudioSrc, export_sample_idx: Option<usize>) {
        let num_ch = input.num_channels() as usize;
        let sr = input.sample_rate() as usize;
        let s = input.audio_buffer();
        let gap = if input.is_live() {
            self.fft_window.value()
        } else {
            self.fft_window.value() - 1
        };

        let start_idx = export_sample_idx.unwrap_or_else(|| {
            if input.is_live() {
                (s.len() / num_ch).saturating_sub(gap)
            } else {
                (input.position().as_secs_f32() * sr as f32) as usize
            }
        });

        let end_idx = if input.is_live() {
            s.len() / num_ch
        } else {
            start_idx + gap + 1
        };

        let window = s
            .get(start_idx * num_ch..end_idx * num_ch)
            .unwrap_or_default();

        let n = window.len() / num_ch;
        let polar_speed = self.get_polar_speed(window, num_ch, sr);
        let gap = sr / 32;
        let mut pos_buf: Vec<Pos2> = Vec::with_capacity(n);
        for i in 0..n {
            let l = window.get(i * num_ch).unwrap_or(&0_f32);
            let r = window.get(i * num_ch + 1).unwrap_or(&0_f32);

            let mut delay_idx = (i + gap) * num_ch + 1;
            if delay_idx > window.len() {
                delay_idx = (i - gap) * num_ch + 1;
            }
            let delay = window.get(delay_idx).unwrap_or(&0_f32);
            let time = (start_idx + i) as f32 / sr as f32;
            let theta = (TAU * polar_speed * time) % TAU;

            let y_sample = if matches!(self.kind, PatternKind::Unipolar) {
                l
            } else {
                r
            };
            let limit = (1.0 - y_sample.abs()).max(0.0).powf(2.0);
            let (x, y, z) = (
                l * theta.sin(),
                y_sample * theta.cos(),
                // ((l - r) * theta.tan()).clamp(-limit, limit),
                (delay * theta.tan()).clamp(-limit, limit),
            );

            if self.use_3d {
                // let pos = start_idx as f32 / s.len() as f32 * input.duration.as_secs_f32();
                let pos = input.position().as_secs_f32();
                let angle = if self.use_rotation {
                    (pos * self.rot_speed * TAU) % TAU
                } else {
                    self.angle / 360.0 * TAU
                };

                let x1 = x * angle.cos() + z * angle.sin();
                let z1 = -x * angle.sin() + z * angle.cos();
                let y1 = y * self.pitch.cos() - z1 * self.pitch.sin();

                let depth = (self.camera_z - z1).max(0.1);
                let perspective = self.camera_z / depth;

                const MIN_OFFSET: f32 = 0.1;
                const MAX_OFFSET: f32 = 5.0;
                let mut offset = ((10.0 / self.camera_z).min(MAX_OFFSET) * MIN_OFFSET).powf(2.0);

                const PITCH_THRESH: f32 = 0.2;
                if self.pitch.abs() < PITCH_THRESH {
                    offset -= lerp(
                        0.0..=offset,
                        (PITCH_THRESH - self.pitch.abs()) / PITCH_THRESH,
                    );
                }
                offset *= self.pitch.signum();

                pos_buf.push(pos2(x1, y1 + offset) * self.scale * perspective);
            } else {
                pos_buf.push(pos2(x, y) * 0.95);
            }
        }

        let us_factor = self.upsample_factor.max(1.0) as usize;
        positional_interp_upsampling(&pos_buf, &mut self.live_buffer, us_factor);
    }

    fn get_polar_speed(&self, window: &[f32], num_ch: usize, sr: usize) -> f32 {
        let freq = fft_max_frequency_bin(self.fft_pass.fft.clone(), window, num_ch, sr).frequency;
        (freq * 2.0).round()
    }
}

impl ActiveGenerator for PolarPatterns {}
impl Generator for PolarPatterns {
    fn prepare(
        &mut self,
        _f: &mut FilterBank,
        _env: &EnvelopeBank,
        input: &dyn AudioSrc,
        export_sample_idx: Option<usize>,
    ) {
        self.draw(input, export_sample_idx);
    }

    fn get_gen_callback_params(
        &mut self,
        st: &crate::state::AppState,
        _live: bool,
        _fps: usize,
    ) -> super::rendering::GenCbParams {
        let env = |src: ModSrc, range: f32| st.env_bank.envelope_value_from_mod_src(src, range);
        GenCbParams::Particle2D(Particle2DCbParams {
            render_mode: ParticleRenderMode::FullSpectrum,
            point_size: (self.point_size
                + env(self.point_size_mod_src, self.point_size_rng) * MAX_POINT_SIZE)
                .clamp(MIN_POINT_SIZE, MAX_POINT_SIZE),
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
                slider_row(ui, "SCALE", &mut self.scale, 0.1, 0.8, 2, false);
                slider_row(ui, "DISTANCE", &mut self.camera_z, 1.0, 10.0, 1, false);
            }

            slider_row(
                ui,
                "UPSAMPLE",
                &mut self.upsample_factor,
                1.0,
                10.0,
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
                slider_row(ui, "PITCH", &mut self.pitch, -1.0, 1.0, 2, false);
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
        Some(&mut self.filter_params)
    }
}
