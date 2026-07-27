use core::f32;
use std::{f32::consts::TAU, time::Duration};

use biquad::Type;
use eframe::egui::{Pos2, Vec2, lerp, pos2};

use crate::{
    Rgba,
    audio::StereoFilter,
    generators::{
        EnvelopeBank, FilterBank, FilterParams, MAX_POINT_SIZE, MIN_POINT_SIZE, PostFx,
        fluidwave::ModSrc,
        positional_interp_upsampling,
        rendering::{GenCbParams, Particle2DCbParams},
        stereometer::{FilterMode, ParticleRenderMode},
    },
    labeled_enum,
    traits::{ActiveGenerator, AudioProperties, Generator, Labeled, ParamAccess},
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
    fn text(&self) -> &'static str {
        self.label()
    }
}
labeled_enum!(WaveformDir {
    In => "Inward",
    Out => "Outward",
    Bipolar => "Bipolar"
}, In);

impl Labeled for WaveformDir {
    fn text(&self) -> &'static str {
        self.label()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Oscilloscope {
    pub filter_params: FilterParams,

    kind: OscilloscopeKind,
    wave_dir: WaveformDir,
    waveform_window_sz: f32,
    delay_plot_density: f32,
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

    delay_samples: f32,

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
            lpf: None,
            last_freq: 500.0,

            continuous: false,
            phase_aligned: true,
            circular_wave_radius: 0.7,

            delay_samples: 100.0,

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
                FilterParams {
                    filter_mode: super::stereometer::FilterMode::Lpf,
                    last_freq: 0.0,
                    filter_freq: 1000.0,
                }
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
fn get_audio_frame(buf: &[f32], idx: usize, num_ch: usize) -> (f32, f32) {
    (
        *buf.get(idx * num_ch).unwrap_or(&0_f32),
        *buf.get(idx * num_ch + 1).unwrap_or(&0_f32),
    )
}

impl Oscilloscope {
    fn align_phase(
        &self,
        start_idx: &mut usize,
        end_idx: &mut usize,
        input: &dyn AudioProperties,
        num_ch: usize,
    ) {
        const MAX_TIMEOUT_SAMPLES: usize = 4096;
        let original_start = *start_idx;
        let original_end = *end_idx;
        while !rising_zero_crossing(get_audio_frame(input.audio_buffer(), *start_idx, num_ch)) {
            if (*start_idx - original_start) > MAX_TIMEOUT_SAMPLES {
                *start_idx = original_start;
                break;
            }
            *start_idx += 1;
        }

        if matches!(self.kind, OscilloscopeKind::CircularWaveform) {
            while !falling_zero_crossing(get_audio_frame(input.audio_buffer(), *end_idx, num_ch)) {
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
        let fp = self.filter_params;
        match fp.filter_mode {
            FilterMode::Off => (l, r),
            FilterMode::Lpf => fil.0.run(l, r),
            FilterMode::Bpf => fil.1.run(l, r),
            FilterMode::Hpf => fil.2.run(l, r),
        }
    }

    pub fn draw(
        &mut self,
        f: &mut FilterBank,
        input: &dyn AudioProperties,
        export_sample_idx: Option<usize>,
    ) {
        let sr = input.sample_rate() as f32;

        let num_ch = input.num_channels() as usize;
        let s = input.audio_buffer();

        let gap = (Duration::from_millis(if matches!(self.kind, OscilloscopeKind::DelayPlot) {
            self.delay_plot_density
        } else {
            self.waveform_window_sz
        } as u64)
        .as_secs_f32()
            * sr) as usize;

        let mut start_idx = export_sample_idx.unwrap_or_else(|| {
            if input.is_live() {
                (s.len() / num_ch).saturating_sub(gap)
            } else {
                (input.position().as_secs_f32() * sr) as usize
            }
        });

        let mut end_idx = if input.is_live() {
            s.len() / num_ch
        } else {
            start_idx + gap + 1
        };

        if self.phase_aligned {
            self.align_phase(&mut start_idx, &mut end_idx, input, num_ch);
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
            .flat_map(|(i, frame)| {
                let l = frame[0];
                let delay_amt = self.delay_samples as usize;
                let d_idx = start_idx + i;

                let (dl, dr) =
                    if d_idx + delay_amt >= gap * num_ch || d_idx + delay_amt * 2 >= gap * num_ch {
                        /* Reflect delay to maintain more clarity */
                        (
                            d_idx.saturating_sub(delay_amt * 2),
                            d_idx.saturating_sub(delay_amt),
                        )
                    } else {
                        (d_idx + delay_amt, d_idx + delay_amt * 2)
                    };

                let delay_l = get_audio_frame(s, dl, num_ch).0;
                let delay_r = get_audio_frame(s, dr, num_ch).1;

                if self.lpf.is_none() || self.last_freq != self.filter_params.filter_freq {
                    let fp = self.filter_params;
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
                    // (start_idx as f32 / pl.contents.samples.len() as f32)
                    //     * pl.contents.duration.as_secs_f32(),
                    input.position().as_secs_f32(),
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
        input: &dyn AudioProperties,
        export_sample_idx: Option<usize>,
    ) {
        self.draw(f, input, export_sample_idx);
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
            point_size: (s.point_size
                + env(s.point_size_mod_src, s.point_size_rng) * MAX_POINT_SIZE)
                .clamp(MIN_POINT_SIZE, MAX_POINT_SIZE),
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

                slider_row(ui, "DELAY", &mut self.delay_samples, 1., 1024.0, 0, false);
                self.delay_samples = self.delay_samples.round();

                slider_row(
                    ui,
                    "LPF BYPASS",
                    &mut self.filter_bypass_threshold,
                    0.0,
                    1.0,
                    2,
                    false,
                );
                let fp = &mut self.filter_params;
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
        Some(&mut self.filter_params)
    }
}
