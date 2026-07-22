use core::f32;
use std::{collections::VecDeque, f32::consts::TAU, time::Duration};

use eframe::egui::{Pos2, Vec2, lerp, pos2};

use crate::{
    Rgba,
    audio::audio_player::AudioPlayer,
    generators::{
        FilterBank, FilterParams, PostFx,
        fluidwave::ModSrc,
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
    DelayPlot => "Delay Plot",
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Oscilloscope {
    kind: OscilloscopeKind,
    wave_dir: WaveformDir,
    window_sz: f32,

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
    filter_params: Option<FilterParams>,
    low_end_focus: bool,
    filter_bypass_threshold: f32,

    #[serde(skip)]
    live_buffer: Vec<Pos2>,
    #[serde(skip)]
    trace_buffer: VecDeque<Pos2>,
}

impl Default for Oscilloscope {
    fn default() -> Self {
        Self {
            // window_sz: 25.0,
            window_sz: 100.0,
            kind: OscilloscopeKind::DelayPlot,
            wave_dir: WaveformDir::In,
            continuous: false,
            phase_aligned: true,
            circular_wave_radius: 0.7,
            delay_samples: 113.0,
            use_rotation: true,
            angle: 100.0,
            upsample_factor: 1.0,
            camera_z: 10.0,
            total_scale: 0.6,
            rot_freq: 0.1,
            pitch: 0.3,
            fs_color: Rgba {
                r: 80.,
                g: 60.,
                b: 255.,
                a: 255.0,
            },
            filter_params: {
                Some(FilterParams {
                    filter_mode: super::stereometer::FilterMode::Lpf,
                    last_freq: 450.0,
                    filter_freq: 450.0,
                })
            },
            low_end_focus: true,
            filter_bypass_threshold: 0.8,

            efx: PostFx {
                use_bloom: true,
                bloom: 3.0,
                bloom_mod_src: ModSrc::EnvA,
                bloom_range: 20.0,
                use_vignette: true,
                use_chroma: true,
                chroma_shift_mod_src: ModSrc::EnvB,
                chroma_shift_range: 100.0,
                chroma_blur: 4.0,
                chroma_type: super::ChromaType::Radial,
                ..Default::default()
            },
            point_size: 0.0025,
            point_size_mod_src: ModSrc::None,
            point_size_rng: 0.0,
            point_size_mod_open: false,
            max_height: 0.7,

            live_buffer: Default::default(),
            trace_buffer: Default::default(),
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
                let mut offset = 10.0 / self.camera_z.max(3.5) * MIN_OFFSET;
                if self.pitch.abs() < 0.1 {
                    offset -= lerp(0.0..=offset, 1.0 - (10.0 * self.pitch.abs()));
                }
                offset *= self.pitch.signum();
                pos2(x1, y2 + offset) * perspective * self.total_scale
            }
        }
    }

    fn filter_samples(&self, f: &mut FilterBank, l: f32, r: f32) -> (f32, f32) {
        if l.abs() >= self.filter_bypass_threshold || r.abs() >= self.filter_bypass_threshold {
            return (l, r);
        }
        let fil = f
            .live_fs_filters
            .as_mut()
            .expect("filter cannot exist without audio");

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
        let num_ch = pl.contents.num_channels as usize;
        let s = &pl.contents.samples;
        let mut start_idx =
            export_sample_idx.unwrap_or_else(|| (pl.position().as_secs_f64() * sr as f64) as usize);
        let gap = (Duration::from_millis(self.window_sz as u64).as_secs_f32() * sr as f32) as usize;
        let mut end_idx = start_idx + gap + 1;

        if self.phase_aligned {
            self.align_phase(&mut start_idx, &mut end_idx, pl, num_ch);
        }

        let mut last_l = *s.get(start_idx * num_ch).unwrap_or(&0_f32);
        let mut last_r = *s.get(start_idx * num_ch + 1).unwrap_or(&0_f32);

        let live_window = s
            .get(start_idx * num_ch..end_idx * num_ch)
            .unwrap_or_default();

        let us_factor = self.upsample_factor as usize;
        let live_window = if us_factor > 1 {
            &live_window
                .chunks_exact(num_ch)
                .flat_map(|s| {
                    let (l, r) = (s[0], *s.last().unwrap_or(&s[0]));
                    let (dl, dr) = (l - last_l, r - last_r);
                    let (il, ir) = (dl / self.upsample_factor, dr / self.upsample_factor);
                    let (mut cl, mut cr) = (last_l, last_r);
                    let v: Vec<_> = (0..us_factor)
                        .flat_map(|_| {
                            let o = [cl, cr];
                            cl += il;
                            cr += ir;
                            o
                        })
                        .collect();
                    (last_l, last_r) = (l, r);
                    v
                })
                .collect::<Vec<f32>>()
        } else {
            live_window
        };
        let dist = |pos: Vec2| {
            let dx = pos.x;
            let dy = pos.y;
            (dx * dx + dy * dy).sqrt()
        };

        let mut x: f32 = -1.0;
        let mut last_pos = Pos2::ZERO;

        let mut angle: f32 = 0.0;
        let angular_increment = TAU / live_window.len() as f32;
        self.live_buffer = live_window
            .chunks_exact(num_ch)
            .enumerate()
            .flat_map(|(i, s)| {
                let l = s[0];
                let cur_sample = l * if !matches!(self.kind, OscilloscopeKind::DelayPlot) {
                    self.max_height
                } else {
                    1.0
                };
                let cur_idx = start_idx + i;
                let mut delay_l = get_audio_frame(
                    pl,
                    cur_idx + us_factor * self.delay_samples as usize,
                    num_ch,
                )
                .0;
                let mut delay_r = get_audio_frame(
                    pl,
                    cur_idx + 2 * us_factor * self.delay_samples as usize,
                    num_ch,
                )
                .1;

                if self.low_end_focus {
                    (delay_l, delay_r) = self.filter_samples(f, delay_l, delay_r);
                }

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
    }
}

impl ActiveGenerator for Oscilloscope {}
impl Generator for Oscilloscope {
    fn prepare(
        &mut self,
        f: &mut FilterBank,
        pl: &crate::audio::audio_player::AudioPlayer,
        export_sample_idx: Option<usize>,
    ) {
        self.draw(f, pl, export_sample_idx);
    }

    fn into_gen_callback_params(
        &mut self,
        _st: &crate::state::AppState,
        _live: bool,
        _fps: usize,
    ) -> super::rendering::GenCbParams {
        let s = self;
        GenCbParams::Particle2D(Particle2DCbParams {
            render_mode: ParticleRenderMode::FullSpectrum,
            point_size: s.point_size,
            add_point_border: false,

            live_pos: std::mem::take(&mut s.live_buffer),
            trace_pos: s.trace_buffer.clone(),

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
                toggle_button_row(ui, "EXTRA DEFINITION", &mut self.low_end_focus, false);
                if !self.use_rotation {
                    slider_row(ui, "ANGLE", &mut self.angle, 0.0, 360.0, 0, false);
                }
                slider_row(ui, "DISTANCE", &mut self.camera_z, 1.0, 10.0, 1, false);
                slider_row(ui, "SCALE", &mut self.total_scale, 0.1, 1.0, 2, false);
                slider_row(ui, "DELAY AMT", &mut self.delay_samples, 1., 512., 0, false);
            } else {
                self.low_end_focus = false;
                toggle_button_row(ui, "PHASE ALIGNED", &mut self.phase_aligned, false);
                slider_row(ui, "MAX HEIGHT", &mut self.max_height, 0.0, 1.0, 2, false);
            }

            slider_row(
                ui,
                if matches!(self.kind, OscilloscopeKind::DelayPlot) {
                    "DENSITY"
                } else {
                    "WINDOW"
                },
                &mut self.window_sz,
                1.0,
                750.0,
                2,
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

            if open.advanced_mode {
                static_label(ui, "ADVANCED");
                if matches!(self.kind, OscilloscopeKind::DelayPlot) {
                    if self.use_rotation {
                        slider_row(ui, "SPEED", &mut self.rot_freq, 0.01, 2.0, 2, false);
                    }
                    slider_row(ui, "PITCH", &mut self.pitch, -1.0, 1.0, 2, false);
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

                slider_row(
                    ui,
                    "LPF THRESH",
                    &mut self.filter_bypass_threshold,
                    0.0,
                    1.0,
                    2,
                    false,
                );
                let fp = self.filter_params.as_mut().expect("safe");
                slider_row(ui, "LPF FREQ", &mut fp.filter_freq, 1.0, 20000.0, 0, false);
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
