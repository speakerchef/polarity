use core::f32;
use std::{collections::VecDeque, f32::consts::TAU, time::Duration};

use eframe::egui::{Pos2, Vec2, pos2};

use crate::{
    Rgba,
    audio::audio_player::AudioPlayer,
    generators::{
        PostFx,
        fluidwave::ModSrc,
        rendering::{GenCbParams, Particle2DCbParams},
        stereometer::ParticleRenderMode,
    },
    labeled_enum,
    traits::{ActiveGenerator, Generator, Labeled, PostFxParams},
    ui::control_panel_widgets::{
        dropdown_row, mod_slider_row, section_header_submenu, slider_row, toggle_button_row,
    },
};

labeled_enum!(OscilloscopeKind {
    Waveform => "Waveform",
    CircularWaveform => "Circular Waveform" 
}, Waveform);
impl Labeled for OscilloscopeKind {
    fn text(self) -> &'static str {
        self.label()
    }
}
labeled_enum!(WaveformDir {
    In => "Inward",
Out => "Outward"
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

    #[serde(skip)]
    live_buffer: Vec<Pos2>,
    #[serde(skip)]
    trace_buffer: VecDeque<Pos2>,
    // #[serde(skip)]
    // last_idx: usize,
}

impl Default for Oscilloscope {
    fn default() -> Self {
        Self {
            window_sz: 25.0,
            kind: OscilloscopeKind::CircularWaveform,
            wave_dir: WaveformDir::In,
            continuous: true,
            phase_aligned: true,
            circular_wave_radius: 0.7,
            // fs_color: Rgba {
            //     r: 0.,
            //     g: 255.,
            //     b: 160.,
            //     a: 255.0,
            // },
            // fs_color: Rgba {
            //     r: 80.,
            //     g: 60.,
            //     b: 255.,
            //     a: 255.0,
            // },
            fs_color: Rgba {
                r: 80.,
                g: 255.,
                b: 25.,
                a: 255.0,
            },
            efx: PostFx {
                use_bloom: true,
                bloom_mod_src: ModSrc::EnvA,
                bloom_range: 20.0,
                use_vignette: true,
                use_chroma: true,
                chroma_shift_mod_src: ModSrc::EnvB,
                chroma_shift_range: 100.0,
                chroma_blur: 4.0,
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
        *s.get((idx + 1) * num_ch).unwrap_or(&0_f32),
    )
}

impl Oscilloscope {
    fn get_position_from_kind(
        &self,
        x: f32,
        cur_sample: f32,
        angle: &mut f32,
        angular_inc: f32,
        phase: f32,
    ) -> Pos2 {
        match self.kind {
            OscilloscopeKind::Waveform => pos2(x, cur_sample),
            OscilloscopeKind::CircularWaveform => {
                let r = self.circular_wave_radius;
                let theta = *angle + phase;
                let mut circle_pos = pos2(theta.sin(), theta.cos()) * r;
                let dir = circle_pos / r;
                circle_pos += match self.wave_dir {
                    WaveformDir::In => (dir * -cur_sample.abs() * r).to_vec2(),
                    WaveformDir::Out => (dir * cur_sample.abs() * (1.0 - r)).to_vec2(),
                };
                *angle += angular_inc * 2.0;
                circle_pos
            }
        }
    }

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

    pub fn draw(&mut self, pl: &AudioPlayer, export_sample_idx: Option<usize>) {
        let sr = pl.contents.sample_rate as usize;
        let num_ch = pl.contents.num_channels as usize;
        let mut start_idx =
            export_sample_idx.unwrap_or_else(|| (pl.position().as_secs_f64() * sr as f64) as usize);
        let gap = (Duration::from_millis(self.window_sz as u64).as_secs_f32() * sr as f32) as usize;
        let mut end_idx = start_idx + gap + 1;

        if self.phase_aligned {
            self.align_phase(&mut start_idx, &mut end_idx, pl, num_ch);
        }

        let live_window = pl
            .contents
            .samples
            .get(start_idx * num_ch..end_idx * num_ch)
            .unwrap_or_default();

        let mut x: f32 = -1.0;
        let mut last_pos = Pos2::ZERO;
        let dist = |pos: Vec2| {
            let dx = pos.x;
            let dy = pos.y;
            (dx * dx + dy * dy).sqrt()
        };

        let mut angle: f32 = 0.0;
        let angular_increment = TAU / live_window.len() as f32;
        self.live_buffer = live_window
            .chunks_exact(2)
            .flat_map(|s| {
                let (l, r) = (s.first().unwrap_or(&0_f32), s.last().unwrap_or(&0_f32));
                let cur_sample = (l + r) / 2.0 * self.max_height;
                let mut pos: Vec<Pos2> = Vec::new();

                let cur_pos = self.get_position_from_kind(
                    x,
                    cur_sample,
                    &mut angle,
                    angular_increment,
                    pl.position().as_secs_f32(),
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
            toggle_button_row(ui, "PHASE ALIGNED", &mut self.phase_aligned, false);

            if matches!(self.kind, OscilloscopeKind::CircularWaveform) {
                slider_row(
                    ui,
                    "RADIUS",
                    &mut self.circular_wave_radius,
                    0.0001,
                    1.0,
                    2,
                    false,
                );
            }

            slider_row(ui, "WINDOW(ms)", &mut self.window_sz, 1.0, 750.0, 2, false);
            slider_row(ui, "MAX HEIGHT", &mut self.max_height, 0.0, 1.0, 2, false);
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

impl PostFxParams for Oscilloscope {
    fn post_fx(&self) -> PostFx {
        self.efx
    }
    fn post_fx_mut(&mut self) -> &mut PostFx {
        &mut self.efx
    }
}
