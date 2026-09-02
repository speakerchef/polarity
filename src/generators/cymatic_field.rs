use std::f32::consts::PI;

use eframe::egui::{Pos2, pos2};

use crate::{
    Rgba,
    generators::{
        EnvelopeBank, FftPass, FftWindow, FilterBank, FilterParams, MAX_POINT_SIZE, MIN_POINT_SIZE,
        PostFx, fft_max_frequency_bin,
        fluidwave::ModSrc,
        rendering::{GenCbParams, Particle2DCbParams},
        stereometer::{FilterMode, ParticleRenderMode},
    },
    traits::{ActiveGenerator, AudioSrc, Generator, ParamAccess},
    ui::control_panel_widgets::{
        mod_slider_row, section_header_submenu, slider_row, toggle_button_row,
    },
};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CymaticField {
    pub filter_params: FilterParams,

    fs_color: Rgba,
    efx: PostFx,

    line_thickness: f32,
    line_thick_mod_src: ModSrc,
    line_thick_range: f32,
    line_thick_mod_open: bool,
    boundary: f32,
    boundary_mod_src: ModSrc,
    boundary_range: f32,
    boundary_mod_open: bool,

    invert: bool,

    max_mode: f32,

    point_size: f32,
    point_size_mod_src: ModSrc,
    point_size_rng: f32,
    point_size_mod_open: bool,

    fft_window: FftWindow,

    #[serde(skip)]
    live_buffer: Vec<Pos2>,
    #[serde(skip)]
    fft_pass: FftPass,
}

impl Default for CymaticField {
    fn default() -> Self {
        Self {
            // fs_color: Rgba::new(255, 65, 110, 255),
            fs_color: Rgba::new(255, 135, 0, 255),
            efx: PostFx {
                use_bloom: true,
                bloom: 0.,
                bloom_mod_src: ModSrc::EnvB,
                bloom_range: 20.0,
                use_vignette: true,
                use_chroma: true,
                chroma_shift_mod_src: ModSrc::EnvA,
                chroma_shift_range: 65.0,
                chroma_blur: 4.0,
                chroma_type: super::ChromaType::Radial,
                ..Default::default()
            },
            boundary: 0.25,
            boundary_mod_src: ModSrc::EnvA,
            boundary_range: 100.0,
            boundary_mod_open: false,

            invert: false,
            max_mode: 10.0,

            line_thickness: 0.250,
            line_thick_mod_src: ModSrc::EnvA,
            line_thick_range: 15.0,
            line_thick_mod_open: false,

            point_size: 0.0032,
            fft_window: FftWindow::default(),
            point_size_mod_src: ModSrc::EnvA,
            point_size_rng: -35.0,
            point_size_mod_open: false,
            live_buffer: Vec::new(),
            fft_pass: FftPass::default(),

            filter_params: FilterParams {
                filter_mode: FilterMode::Hpf,
                last_freq: 0.0,
                filter_freq: 100.0,
            },
        }
    }
}

impl CymaticField {
    pub fn draw(
        &mut self,
        f: &mut FilterBank,
        env: &EnvelopeBank,
        input: &dyn AudioSrc,
        export_sample_idx: Option<usize>,
    ) {
        let num_ch = input.num_channels() as usize;
        let sr = input.sample_rate() as f32;
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
                (input.position().as_secs_f32() * sr) as usize
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

        // fft needs at least 8192 else panics
        if window.len() / num_ch < FftWindow::default().value() {
            return;
        }

        let buf = window
            .chunks_exact(num_ch)
            .flat_map(|s| {
                let (l, r) = (s[0], *s.last().unwrap_or(&0_f32));
                if let Some(fil) = f.live_fs_filters.as_mut() {
                    let s = fil.2.run(l, r);
                    [s.0, s.1]
                } else {
                    [l, r]
                }
            })
            .collect::<Vec<f32>>();

        self.live_buffer = self.chladni_pattern(env, &buf, num_ch, sr as usize);
    }

    fn chladni_pattern(
        &self,
        env: &EnvelopeBank,
        buf: &[f32],
        num_ch: usize,
        sample_rate: usize,
    ) -> Vec<Pos2> {
        let f = fft_max_frequency_bin(self.fft_pass.fft.clone(), buf, num_ch, sample_rate);
        let (m, n) = self.frequency_to_chladni_mode(f.frequency);
        let resolution: usize = 300;

        let mut out = Vec::<Pos2>::new();
        for y in 0..=resolution {
            for x in 0..=resolution {
                let x = x as f32 / resolution as f32;
                let y = y as f32 / resolution as f32;
                let v = self.chladni(x, y, m, n);

                let b = 1.0
                    - (self.boundary
                        + env.envelope_value_from_mod_src(
                            self.boundary_mod_src,
                            self.boundary_range,
                        ))
                    .min(1.0);
                let mut check = v < b
                    && v >= b
                        - (self.line_thickness
                            + env.envelope_value_from_mod_src(
                                self.line_thick_mod_src,
                                self.line_thick_range,
                            ));
                if self.invert {
                    check = !check;
                }

                if check {
                    out.extend([pos2(x, y), pos2(-x, y), pos2(-x, -y), pos2(x, -y)]);
                }
            }
        }
        out
    }

    fn frequency_to_chladni_mode(&self, f: f32) -> (f32, f32) {
        let modes = self.generate_chladni_modes();
        let index = self.frequency_to_mode_index(f, modes.len() as f32 - 1.0);

        modes[index]
    }
    fn generate_chladni_modes(&self) -> Vec<(f32, f32)> {
        let max = self.max_mode;
        let mut n = 1.0;
        let mut m = 2.0;
        let mut out = Vec::new();
        while m <= max {
            while n < m {
                out.push((n, m));
                n += 1.0;
            }
            n = 1.0;
            m += 1.0;
        }
        out
    }

    fn chladni(&self, x: f32, y: f32, m: f32, n: f32) -> f32 {
        (m * PI * x).cos() * (n * PI * y).cos() - (n * PI * x).cos() * (m * PI * y).cos()
    }

    fn frequency_to_mode_index(&self, f: f32, max_idx: f32) -> usize {
        if f <= 0.0 {
            return 0;
        }
        const MIN_F: f32 = 50.0;
        const MAX_F: f32 = 20_000.0;

        let norm = (f / MIN_F).ln() / (MAX_F / MIN_F).ln();
        (norm * max_idx).floor() as usize
    }
}

impl ActiveGenerator for CymaticField {}
impl Generator for CymaticField {
    fn prepare(
        &mut self,
        f: &mut super::FilterBank,
        env: &EnvelopeBank,
        input: &dyn AudioSrc,
        export_sample_idx: Option<usize>,
    ) {
        self.draw(f, env, input, export_sample_idx);
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
            fs_color: self.fs_color.into(),
            ..Default::default()
        })
    }

    fn draw_filtering_menu(
        &mut self,
        ui: &mut eframe::egui::Ui,
        open: &mut crate::state::BoolStates,
    ) {
        section_header_submenu(ui, "FILTERING", &mut open.filtering_open);
        let fp = &mut self.filter_params;
        if open.filtering_open {
            slider_row(ui, "FREQ", &mut fp.filter_freq, 1.0, 1000.0, 0, false);
            fp.filter_freq = fp.filter_freq.round();
        }
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
            toggle_button_row(ui, "INVERT", &mut self.invert, false);
            slider_row(ui, "MAX MODE", &mut self.max_mode, 2.0, 10.0, 0, false);
            self.max_mode = self.max_mode.round();
            mod_slider_row(
                ui,
                "LINE WIDTH",
                &mut self.line_thickness,
                0.001,
                1.0,
                3,
                &mut self.line_thick_mod_src,
                &mut self.line_thick_mod_open,
                &mut open.mod_src_open,
                &mut self.line_thick_range,
                false,
            );

            mod_slider_row(
                ui,
                "BOUNDARY",
                &mut self.boundary,
                0.01,
                1.0,
                2,
                &mut self.boundary_mod_src,
                &mut self.boundary_mod_open,
                &mut open.mod_src_open,
                &mut self.boundary_range,
                false,
            );

            mod_slider_row(
                ui,
                "POINT SIZE",
                &mut self.point_size,
                0.0005,
                0.01,
                4,
                &mut self.point_size_mod_src,
                &mut self.point_size_mod_open,
                &mut open.mod_src_open,
                &mut self.point_size_rng,
                false,
            );
        }
    }
}
impl ParamAccess for CymaticField {
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
