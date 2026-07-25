use core::f32;
use std::{f32::consts::TAU, time::Duration};

use biquad::Type;
use eframe::egui::{Pos2, Vec2, lerp, pos2};

use crate::{
    Rgba,
    audio::{StereoFilter, audio_player::AudioPlayer},
    generators::{
        EnvelopeBank, FilterBank, FilterParams, PostFx,
        fluidwave::ModSrc,
        positional_interp_upsampling,
        rendering::{GenCbParams, Particle2DCbParams},
        stereometer::{FilterMode, ParticleRenderMode},
    },
    labeled_enum,
    traits::{ActiveGenerator, Generator, Labeled, ParamAccess},
    ui::control_panel_widgets::{
        dropdown_row, mod_slider_row, section_header_submenu, slider_row, static_label,
        toggle_button_row,
    },
};

labeled_enum!(OscilloscopeKind {
    Waveform => "Waveform",
    CircularWaveform => "Circular Waveform" ,
    DelayPlot => "Galactic Orbit",
}, Waveform);
impl Labeled for OscilloscopeKind {
    fn text(self) -> &'static str {
        self.label()
    }
}
labeled_enum!(WaveformDir {
    In => "Inward",
    Out => "Outward",
    Bipolar => "Bipolar"
}, In);

impl Labeled for WaveformDir {
    fn text(self) -> &'static str {
        self.label()
    }
}

labeled_enum!(DelayDivision {
    Div64 => "1 / 64",
    Div32 => "1 / 32",
    Div16 => "1 / 16",
    Div8 => "1 / 8",
}, Div64);

impl DelayDivision {
    fn value(&self) -> f32 {
        match self {
            Self::Div64 => 1.0 / 64.0,
            Self::Div32 => 1.0 / 32.0,
            Self::Div16 => 1.0 / 16.0,
            Self::Div8 => 1.0 / 8.0,
        }
    }
}

impl Labeled for DelayDivision {
    fn text(self) -> &'static str {
        self.label()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Oscilloscope {
    pub filter_params: Option<FilterParams>,

    kind: OscilloscopeKind,
    wave_dir: WaveformDir,
    waveform_window_sz: f32,
    delay_plot_density: f32,
    sample_rate: Option<usize>,
    last_freq: f32,

    #[serde(skip)]
    lpf: Option<StereoFilter>,

    fs_color: Rgba,
    efx: PostFx,
    point_size: f32,
    point_size_mod_src: ModSrc,
    point_size_rng: f32,
    point_size_mod_open: bool,
    max_height: f32,
    continuous: bool,
    phase_aligned: bool,
    circular_wave_radius: f32,

    default_delay_set: bool,
    delay_samples: f32,
    delay_division: DelayDivision,
    use_delay_division: bool,
    delay_div_open: bool,

    use_rotation: bool,
    angle: f32,
    upsample_factor: f32,
    camera_z: f32,
    total_scale: f32,
    rot_freq: f32,
    pitch: f32,
    low_end_focus: bool,
    filter_bypass_threshold: f32,

    #[serde(skip)]
    live_buffer: Vec<Pos2>,
}

impl Default for Oscilloscope {
    fn default() -> Self {
        Self {
            waveform_window_sz: 25.0,
            delay_plot_density: 250.0,
            kind: OscilloscopeKind::DelayPlot,
            wave_dir: WaveformDir::In,
            sample_rate: None,
            lpf: None,
            last_freq: 500.0,

            continuous: false,
            phase_aligned: true,
            circular_wave_radius: 0.7,

            default_delay_set: false,
            delay_samples: 0.0,
            delay_division: DelayDivision::default(),
            use_delay_division: true,
            delay_div_open: false,

            use_rotation: true,
            angle: 100.0,
            upsample_factor: 2.0,
            camera_z: 10.0,
            total_scale: 0.6,
            rot_freq: 0.1,
            pitch: 0.3,
            //purple aura
            fs_color: Rgba {
                r: 80.,
                g: 60.,
                b: 255.,
                a: 255.0,
            },
            filter_params: {
                Some(FilterParams {
                    filter_mode: super::stereometer::FilterMode::Lpf,
                    last_freq: 0.0,
                    filter_freq: 1000.0,
                })
            },
            low_end_focus: false,
            filter_bypass_threshold: 0.5,

            efx: PostFx {
                use_bloom: true,
                bloom: 2.2,
                bloom_mod_src: ModSrc::EnvA,
                bloom_range: 40.0,
                use_vignette: true,
                use_chroma: true,
                chroma_shift_mod_src: ModSrc::EnvB,
                chroma_shift_range: 100.0,
                chroma_blur: 4.0,
                chroma_type: super::ChromaType::Radial,
                ..Default::default()
            },
            point_size: 0.0015,
            // point_size: 0.0023,
            point_size_mod_src: ModSrc::None,
            point_size_rng: 0.0,
            point_size_mod_open: false,
            max_height: 0.7,

            live_buffer: Default::default(),
        }
    }
}

fn rising_zero_crossing(frame: (f32, f32)) -> bool {
    frame.0 <= 0.0 && frame.1 > 0.0
}
fn falling_zero_crossing(frame: (f32, f32)) -> bool {
    frame.0 >= 0.0 && frame.1 < 0.0
}
fn get_audio_frame(pl: &AudioPlayer, idx: usize, num_ch: usize) -> (f32, f32) {
    let s = &pl.contents.samples;
    (
        *s.get(idx * num_ch).unwrap_or(&0_f32),
        *s.get(idx * num_ch + 1).unwrap_or(&0_f32),
    )
}

impl Oscilloscope {
    fn align_phase(
        &self,
        start_idx: &mut usize,
        end_idx: &mut usize,
        pl: &AudioPlayer,
        num_ch: usize,
    ) {
        const MAX_TIMEOUT_SAMPLES: usize = 4096;
        let original_start = *start_idx;
        let original_end = *end_idx;
        while !rising_zero_crossing(get_audio_frame(pl, *start_idx, num_ch)) {
            if (*start_idx - original_start) > MAX_TIMEOUT_SAMPLES {
                *start_idx = original_start;
                break;
            }
            *start_idx += 1;
        }

        if matches!(self.kind, OscilloscopeKind::CircularWaveform) {
            while !falling_zero_crossing(get_audio_frame(pl, *end_idx, num_ch)) {
                if (*end_idx - original_end) > MAX_TIMEOUT_SAMPLES {
                    *end_idx = original_end;
                    break;
                }
                *end_idx += 1;
            }
        }
    }

    fn get_position_from_kind(
        &self,
        x: f32,
        cur_sample: f32,
        delay: (f32, f32),
        angle: &mut f32,
        angular_inc: f32,
        pl_pos: f32,
    ) -> Pos2 {
        match self.kind {
            OscilloscopeKind::Waveform => pos2(x, cur_sample),
            OscilloscopeKind::CircularWaveform => {
                let r = self.circular_wave_radius;
                let theta = *angle + pl_pos;
                let mut circle_pos = pos2(theta.sin(), theta.cos()) * r;
                let dir = circle_pos / r;
                circle_pos += match self.wave_dir {
                    WaveformDir::In => (dir * -cur_sample.abs() * r).to_vec2(),
                    WaveformDir::Out => (dir * cur_sample.abs() * (1.0 - r)).to_vec2(),
                    WaveformDir::Bipolar => (dir * cur_sample * (1.0 - r)).to_vec2(),
                };
                *angle += angular_inc * 2.0;
                circle_pos
            }
            OscilloscopeKind::DelayPlot => {
                let (x, y, z) = (cur_sample, delay.0, delay.1);
                let angle = if self.use_rotation {
                    (pl_pos * self.rot_freq * TAU) % TAU
                } else {
                    self.angle / 360.0 * TAU
                };

                let x1 = x * angle.cos() + z * angle.sin();
                let z1 = -x * angle.sin() + z * angle.cos();
                let y2 = y * self.pitch.cos() - z1 * self.pitch.sin();

                let depth = (self.camera_z - z1).max(0.1);
                let perspective = self.camera_z / depth;

                const MIN_OFFSET: f32 = 0.1;
                const MAX_OFFSET: f32 = 2.85;
                let mut offset = (10.0 / self.camera_z).min(MAX_OFFSET) * MIN_OFFSET;

                const PITCH_THRESH: f32 = 0.3;
                if self.pitch.abs() < PITCH_THRESH {
                    offset -= lerp(
                        0.0..=offset,
                        (PITCH_THRESH - self.pitch.abs()) / PITCH_THRESH,
                    );
                }
                offset *= self.pitch.signum();
                pos2(x1, y2 + offset) * perspective * self.total_scale
            }
        }
    }

    fn filter_samples(&mut self, f: &mut FilterBank, l: f32, r: f32) -> (f32, f32) {
        if l.abs() >= self.filter_bypass_threshold || r.abs() >= self.filter_bypass_threshold {
            return (l, r);
        }
        let Some(fil) = f.live_fs_filters.as_mut() else {
            return (l, r);
        };
        let fp = self.filter_params.expect("safe");
        match fp.filter_mode {
            FilterMode::Off => (l, r),
            FilterMode::Lpf => fil.0.run(l, r),
            FilterMode::Bpf => fil.1.run(l, r),
            FilterMode::Hpf => fil.2.run(l, r),
        }
    }

    pub fn draw(&mut self, f: &mut FilterBank, pl: &AudioPlayer, export_sample_idx: Option<usize>) {
        let sr = pl.contents.sample_rate as usize;
        self.sample_rate = Some(sr);
        if !self.default_delay_set {
            self.delay_samples = sr as f32 * DelayDivision::default().value();
            self.default_delay_set = true;
        }
        if self.use_delay_division {
            self.delay_samples = sr as f32 * self.delay_division.value();
        }

        let num_ch = pl.contents.num_channels as usize;
        let s = &pl.contents.samples;
        let mut start_idx =
            export_sample_idx.unwrap_or_else(|| (pl.position().as_secs_f64() * sr as f64) as usize);
        let gap = (Duration::from_millis(if matches!(self.kind, OscilloscopeKind::DelayPlot) {
            self.delay_plot_density
        } else {
            self.waveform_window_sz
        } as u64)
        .as_secs_f32()
            * sr as f32) as usize;
        let mut end_idx = start_idx + gap + 1;

        if self.phase_aligned {
            self.align_phase(&mut start_idx, &mut end_idx, pl, num_ch);
        }

        let live_window = s
            .get(start_idx * num_ch..end_idx * num_ch)
            .unwrap_or_default();

        let dist = |pos: Vec2| {
            let dx = pos.x;
            let dy = pos.y;
            (dx * dx + dy * dy).sqrt()
        };

        let mut x: f32 = -1.0;
        let mut last_pos = Pos2::ZERO;

        let mut angle: f32 = 0.0;
        let angular_increment = TAU / live_window.len() as f32;
        let pos_buf: Vec<Pos2> = live_window
            .chunks_exact(num_ch)
            .enumerate()
            .flat_map(|(i, s)| {
                let l = s[0];
                let cur_idx = start_idx + i;
                let delay_l = get_audio_frame(pl, cur_idx + self.delay_samples as usize, num_ch).0;
                let delay_r =
                    get_audio_frame(pl, cur_idx + 2 * self.delay_samples as usize, num_ch).1;

                if self.lpf.is_none()
                    || self.last_freq != self.filter_params.expect("safe").filter_freq
                {
                    let fp = self.filter_params.as_mut().expect("safe");
                    self.lpf = Some(StereoFilter::from_coeffs_butterworth(
                        Type::LowPass,
                        fp.filter_freq,
                        sr as u32,
                    ));
                    self.last_freq = fp.filter_freq;
                }
                let fil2 = self.lpf.as_mut().expect("safe");

                let ((l, _), (delay_l, delay_r)) = if self.low_end_focus {
                    (
                        fil2.run(l, delay_r),
                        self.filter_samples(f, delay_l, delay_r),
                    )
                } else {
                    ((l, 0_f32), (delay_l, delay_r))
                };

                let cur_sample = l * if !matches!(self.kind, OscilloscopeKind::DelayPlot) {
                    self.max_height
                } else {
                    1.0
                };

                let mut pos: Vec<Pos2> = Vec::new();
                let cur_pos = self.get_position_from_kind(
                    x,
                    cur_sample,
                    (delay_l, delay_r),
                    &mut angle,
                    angular_increment,
                    (start_idx as f32 / pl.contents.samples.len() as f32)
                        * pl.contents.duration.as_secs_f32(),
                );
                pos.push(cur_pos);

                let diff = cur_pos - last_pos;
                let dist = dist(diff);
                let dir = diff / dist;
                let mut intervals = (dist / self.point_size).floor() as usize;

                if self.continuous && last_pos != pos2(0.0, 0.0) {
                    while intervals > 0 {
                        pos.push(last_pos + dir * self.point_size);
                        last_pos += dir * self.point_size;
                        intervals -= 1;
                    }
                }

                x += 2.0 / live_window.len() as f32 * 2.0;
                last_pos = *pos.last().unwrap_or(&cur_pos);

                pos
            })
            .collect();

        self.live_buffer = positional_interp_upsampling(self.upsample_factor, &pos_buf);
    }
}

impl ActiveGenerator for Oscilloscope {}
impl Generator for Oscilloscope {
    fn prepare(
        &mut self,
        f: &mut FilterBank,
        _env: &EnvelopeBank,
        pl: &crate::audio::audio_player::AudioPlayer,
        export_sample_idx: Option<usize>,
    ) {
        self.draw(f, pl, export_sample_idx);
    }

    fn get_gen_callback_params(
        &mut self,
        st: &crate::state::AppState,
        _live: bool,
        _fps: usize,
    ) -> super::rendering::GenCbParams {
        let s = self;
        let env = |src: ModSrc, range: f32| st.env_bank.envelope_value_from_mod_src(src, range);
        GenCbParams::Particle2D(Particle2DCbParams {
            render_mode: ParticleRenderMode::FullSpectrum,
            point_size: s.point_size + env(s.point_size_mod_src, s.point_size_rng),
            add_point_border: false,

            live_pos: std::mem::take(&mut s.live_buffer),

            fs_color: s.fs_color.into(),
            ..Default::default()
        })
    }

    fn draw_render_menu(&mut self, ui: &mut eframe::egui::Ui, open: &mut crate::state::BoolStates) {
        section_header_submenu(ui, "RENDER", &mut open.render_open);
        if open.render_open {
            dropdown_row(
                ui,
                "MODE",
                &mut self.kind,
                OscilloscopeKind::ALL,
                &mut open.render_mode_options_open,
                false,
            );
            if matches!(self.kind, OscilloscopeKind::CircularWaveform) {
                dropdown_row(
                    ui,
                    "DIRECTION",
                    &mut self.wave_dir,
                    WaveformDir::ALL,
                    &mut open.force_direction_options_open,
                    false,
                );
            }
        }
    }

    fn draw_filtering_menu(
        &mut self,
        _ui: &mut eframe::egui::Ui,
        _open: &mut crate::state::BoolStates,
    ) {
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
            toggle_button_row(ui, "CONTINUOUS", &mut self.continuous, false);

            if matches!(self.kind, OscilloscopeKind::CircularWaveform) {
                slider_row(
                    ui,
                    "RADIUS",
                    &mut self.circular_wave_radius,
                    0.01,
                    1.,
                    2,
                    false,
                );
            }
            if matches!(self.kind, OscilloscopeKind::DelayPlot) {
                toggle_button_row(ui, "ROTATION", &mut self.use_rotation, false);
                toggle_button_row(ui, "EXTRA DEFINITION (LPF)", &mut self.low_end_focus, false);
                if !self.use_rotation {
                    slider_row(ui, "ANGLE", &mut self.angle, 0.0, 360.0, 0, false);
                }
                slider_row(ui, "DISTANCE", &mut self.camera_z, 1.0, 10.0, 1, false);
                slider_row(ui, "SCALE", &mut self.total_scale, 0.1, 1.0, 2, false);
            } else {
                self.low_end_focus = false;
                toggle_button_row(ui, "PHASE ALIGNED", &mut self.phase_aligned, false);
                slider_row(ui, "MAX HEIGHT", &mut self.max_height, 0.0, 1.0, 2, false);
            }

            if matches!(self.kind, OscilloscopeKind::DelayPlot) {
                slider_row(
                    ui,
                    "DENSITY",
                    &mut self.delay_plot_density,
                    10.0,
                    1000.0,
                    0,
                    false,
                );
            } else {
                slider_row(
                    ui,
                    "WINDOW(ms)",
                    &mut self.waveform_window_sz,
                    1.0,
                    750.0,
                    2,
                    false,
                );
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
            self.upsample_factor = self.upsample_factor.floor();

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

            if open.advanced_mode {
                static_label(ui, "ADVANCED");
                if matches!(self.kind, OscilloscopeKind::DelayPlot) {
                    if self.use_rotation {
                        slider_row(ui, "SPEED", &mut self.rot_freq, 0.01, 2.0, 2, false);
                    }
                    slider_row(ui, "PITCH", &mut self.pitch, -1.0, 1.0, 2, false);
                }

                if let Some(sr) = self.sample_rate {
                    toggle_button_row(ui, "DELAY DIVISIONS", &mut self.use_delay_division, false);
                    if self.use_delay_division {
                        dropdown_row(
                            ui,
                            "DELAY",
                            &mut self.delay_division,
                            DelayDivision::ALL,
                            &mut self.delay_div_open,
                            false,
                        );
                    } else {
                        slider_row(
                            ui,
                            "DELAY",
                            &mut self.delay_samples,
                            1.,
                            sr as f32 / 8.0,
                            0,
                            false,
                        );
                    }
                }

                slider_row(
                    ui,
                    "LPF BYPASS AMT",
                    &mut self.filter_bypass_threshold,
                    0.0,
                    1.0,
                    2,
                    false,
                );
                let fp = self.filter_params.as_mut().expect("safe");
                slider_row(ui, "LPF FREQ", &mut fp.filter_freq, 1.0, 1000.0, 0, false);
                fp.filter_freq = fp.filter_freq.round();
            }
        }
    }
}

impl ParamAccess for Oscilloscope {
    fn post_fx(&self) -> PostFx {
        self.efx
    }
    fn post_fx_mut(&mut self) -> &mut PostFx {
        &mut self.efx
    }
    fn filter_params(&mut self) -> Option<&mut FilterParams> {
        self.filter_params.as_mut()
    }
}
