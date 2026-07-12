use core::f32;
use std::{collections::VecDeque, time::Duration};

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
    traits::{ActiveGenerator, Generator, PostFxParams},
    ui::control_panel_widgets::{mod_slider_row, section_header_submenu, slider_row},
};

labeled_enum!(OscilloscopeKind {
    Waveform => "Waveform",
    CircularWaveform => "Circular Waveform" 
}, Waveform);

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Oscilloscope {
    pub kind: OscilloscopeKind,
    pub window_sz: f32,

    pub fs_color: Rgba,
    pub efx: PostFx,
    pub point_size: f32,
    pub point_size_mod_src: ModSrc,
    pub point_size_rng: f32,
    pub point_size_mod_open: bool,
    pub max_height: f32,

    #[serde(skip)]
    pub live_buffer: Vec<Pos2>,
    #[serde(skip)]
    pub trace_buffer: VecDeque<Pos2>,
    // #[serde(skip)]
    // last_idx: usize,
}

impl Default for Oscilloscope {
    fn default() -> Self {
        Self {
            window_sz: 25.0,
            kind: Default::default(),
            fs_color: Rgba {
                r: 0.,
                g: 255.,
                b: 160.,
                a: 255.0,
            },
            efx: PostFx {
                use_bloom: true,
                use_vignette: true,
                use_chroma: true,
                chroma_shift_mod_src: ModSrc::EnvB,
                chroma_shift_range: 100.0,
                chroma_blur: 4.0,
                ..Default::default()
            },
            point_size: 0.004,
            point_size_mod_src: ModSrc::None,
            point_size_rng: 0.0,
            point_size_mod_open: false,
            max_height: 0.6,

            live_buffer: Default::default(),
            trace_buffer: Default::default(),
        }
    }
}

fn rising_zero_crossing(frame: (f32, f32)) -> bool {
    frame.0 <= 0.0 && frame.1 > 0.0
}
fn get_audio_frame(pl: &AudioPlayer, idx: usize, num_ch: usize) -> (f32, f32) {
    let frame = pl
        .contents
        .samples
        .get(idx * num_ch..(idx + 1) * num_ch)
        .unwrap_or_default();

    (
        *frame.first().unwrap_or(&0_f32),
        *frame.last().unwrap_or(&0_f32),
    )
}

impl Oscilloscope {
    pub fn draw(&mut self, pl: &AudioPlayer, export_sample_idx: Option<usize>) {
        let sr = pl.contents.sample_rate as usize;
        let num_ch = pl.contents.num_channels as usize;
        let mut sample_idx =
            export_sample_idx.unwrap_or_else(|| (pl.position().as_secs_f64() * sr as f64) as usize);

        let original = sample_idx;
        const MAX_TIMEOUT_SAMPLES: usize = 2048;
        while !rising_zero_crossing(get_audio_frame(pl, sample_idx, num_ch))
            && (sample_idx - original) < MAX_TIMEOUT_SAMPLES
        {
            sample_idx += 1;
        }

        let live_window = pl
            .contents
            .samples
            .get(
                sample_idx * num_ch
                    ..(sample_idx as f32
                        + Duration::from_millis(self.window_sz as u64).as_secs_f32() * sr as f32)
                        as usize
                        * num_ch,
            )
            .unwrap_or_default();

        let mut x: f32 = -1.0;
        let mut last_pos = Pos2::ZERO;
        let dist = |pos: Vec2| {
            let dx = pos.x;
            let dy = pos.y;
            (dx * dx + dy * dy).sqrt()
        };

        self.live_buffer = live_window
            .chunks_exact(2)
            .flat_map(|s| {
                let (l, r) = (s.first().unwrap_or(&0_f32), s.last().unwrap_or(&0_f32));
                let cur_sample = (l + r) / 2.0 * self.max_height;
                let mut pos: Vec<Pos2> = Vec::new();
                let cur_pos = Pos2 { x, y: cur_sample };
                pos.push(cur_pos);

                let diff = cur_pos - last_pos;
                let dist = dist(diff);
                let dir = diff / dist;
                let mut intervals = (dist / self.point_size).floor() as usize;

                if last_pos != pos2(0.0, 0.0) {
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
            live_pos: std::mem::take(&mut s.live_buffer),
            trace_pos: s.trace_buffer.clone(),

            fs_color: s.fs_color.into(),
            ..Default::default()
        })
    }

    fn draw_render_menu(
        &mut self,
        _ui: &mut eframe::egui::Ui,
        _open: &mut crate::state::BoolStates,
    ) {
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
            slider_row(
                ui,
                "WINDOW (ms)",
                &mut self.window_sz,
                1.0,
                1000.0,
                2,
                false,
            );
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
