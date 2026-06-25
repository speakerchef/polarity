#![allow(dead_code, unused)]
use std::ops::Mul;
use std::time::Instant;

use eframe::egui::{self, Pos2};
use eframe::egui::{Align, Color32, FontId, StrokeKind, pos2, vec2};
use eframe::egui_wgpu;

use crate::generators::rendering::{EffectsCallback, OutputCallback, RendererCallback};
use crate::ui::canvas_widgets::fullscreen_button;
use crate::ui::timeline_widgets::{SHARP, border};
use crate::{GeneratorKind, points_to_quad_vertices};
use crate::{audio::audio_player::AudioPlayer, state::AppState};

use crate::ui::{custom_text, palette};

pub fn draw(ui: &mut egui::Ui, st: &mut AppState, pl: &Option<AudioPlayer>) {
    ui.ctx().request_repaint();
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(palette::VOID(ui.style().visuals.dark_mode))
                .inner_margin(0.0)
                .outer_margin(0.0),
        )
        .show_inside(ui, |ui| {
            if ui.ctx().input(
                |i| i.pointer.hover_pos().is_some(), /* is cursor on window */
            ) || !st.fullscreen
            {
                fullscreen_button(ui, st);
            }
            custom_painting(ui, st, pl);
        });
}

pub const NUM_PARTICLES: i32 = 50;
pub const fn generate_particle_grid() -> [[f32; 8]; (NUM_PARTICLES * NUM_PARTICLES) as usize] {
    let mut pos =
        [[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; (NUM_PARTICLES * NUM_PARTICLES) as usize];
    let mut idx = 0;
    let mut i = 0;
    while i < NUM_PARTICLES {
        let mut j = 0;
        while j < NUM_PARTICLES {
            pos[idx] = [
                i as f32 / NUM_PARTICLES as f32,
                j as f32 / NUM_PARTICLES as f32,
                -i as f32 / NUM_PARTICLES as f32,
                -j as f32 / NUM_PARTICLES as f32,
                -i as f32 / NUM_PARTICLES as f32,
                j as f32 / NUM_PARTICLES as f32,
                i as f32 / NUM_PARTICLES as f32,
                -j as f32 / NUM_PARTICLES as f32,
            ];
            idx += 1;
            j += 1;
        }
        i += 1;
    }
    pos
}

pub const fn generate_rand_particles() -> [[f32; 8]; (NUM_PARTICLES * NUM_PARTICLES) as usize] {
    let mut pos = [[0.0f32; 8]; (NUM_PARTICLES * NUM_PARTICLES) as usize];
    let mut seed: u64 = 0xDEADBEEF;
    let mut idx = 0;
    while idx < (NUM_PARTICLES * NUM_PARTICLES) as usize {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let x = (seed as i64 % 1000) as f32 / 1000.0;
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let y = (seed as i64 % 1000) as f32 / 1000.0;
        pos[idx] = [x, y, -x, -y, -x, y, x, -y];
        idx += 1;
    }
    pos
}

fn custom_painting(ui: &mut egui::Ui, st: &mut AppState, pl: &Option<AudioPlayer>) {
    let (h, w) = (ui.available_height(), ui.available_width());
    let l = h.min(w) * 0.99;
    let canvas_size = vec2(l, l);

    let center = ui.max_rect().center();
    let top_left = pos2(center.x - l / 2.0, center.y - l / 2.0);
    let rect = ui
        .allocate_rect(
            egui::Rect::from_min_size(top_left, canvas_size),
            egui::Sense::focusable_noninteractive(),
        )
        .rect;
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, Color32::BLACK);
    let Some(pl) = pl else {
        return;
    };
    match st.gen_kind {
        GeneratorKind::Stereometer => st.stereo.draw(pl, None),
        GeneratorKind::Fluidwave => (),
    }

    let num_channels = pl.contents.num_channels as usize;

    let sample_pos = pl.position().as_secs_f64();
    let sample_idx = (sample_pos * pl.contents.sample_rate as f64) as usize;
    let live_window = pl
        .contents
        .samples
        .get(sample_idx * num_channels..sample_idx * num_channels + 50 * 50)
        .unwrap_or_default();
    let speaker_pos: Vec<_> = live_window
        .chunks_exact(2)
        .map(|x| pos2(*x.first().unwrap(), *x.last().unwrap_or(x.first().unwrap())).mul(0.1))
        .collect();

    let now = Instant::now();
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        RendererCallback {
            canvas_size,
            gen_kind: st.gen_kind,

            render_mode: st.stereo.render_mode,
            live_pos: std::mem::take(&mut st.stereo.live_buffer),
            trace_pos: st.stereo.trace_buffer.clone().into(),

            live_low_pos: std::mem::take(&mut st.stereo.live_low_buffer),
            live_mid_pos: std::mem::take(&mut st.stereo.live_mid_buffer),
            live_high_pos: std::mem::take(&mut st.stereo.live_high_buffer),
            trace_low_pos: st.stereo.trace_low_buffer.clone().into(),
            trace_mid_pos: st.stereo.trace_mid_buffer.clone().into(),
            trace_high_pos: st.stereo.trace_high_buffer.clone().into(),

            fs_color: st.stereo.fs_color.into(),
            lb_color: st.stereo.mb_color[0].into(),
            mb_color: st.stereo.mb_color[1].into(),
            hb_color: st.stereo.mb_color[2].into(),

            particle_pos: speaker_pos,

            frame_time: now
                .duration_since(st.fwave.last_frame)
                .as_secs_f32()
                .min(0.016)
                / 8.0,
            // frame_time: 1. / 120.,
            g: st.fwave.gravity,
            pm: st.fwave.pressure_multiplier,
            td: st.fwave.target_density,
            r: st.fwave.smoothing_radius,
            npm: st.fwave.near_pressure_multiplier,
            vs: st.fwave.viscosity_strength,
        },
    ));
    st.fwave.last_frame = now;

    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        EffectsCallback {
            top_left: rect.left_top(),
            // bloom_amt: st.stereo.bloom,
            bloom_amt: 0.0,
        },
    ));

    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        OutputCallback,
    ));
}
