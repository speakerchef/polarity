#![allow(dead_code, unused)]
use std::ops::Div;
use std::time::{Duration, Instant};

use eframe::egui::{self, Pos2, Vec2};
use eframe::egui::{Align, Color32, FontId, StrokeKind, pos2, vec2};
use eframe::egui_wgpu;

use crate::generators::fluidwave::EnergyTransferMode;
use crate::generators::rendering::{EffectsCallback, OutputCallback, RendererCallback};
use crate::ui::canvas_widgets::fullscreen_button;
use crate::ui::timeline_widgets::{SHARP, border};
use crate::{GeneratorKind, envelope_follower, points_to_quad_vertices};
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

pub const NUM_PARTICLES: i32 = 70;
pub const fn generate_particle_grid() -> [[f32; 8]; (NUM_PARTICLES * NUM_PARTICLES) as usize] {
    let mut pos =
        [[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; (NUM_PARTICLES * NUM_PARTICLES) as usize];
    let mut idx = 0;
    let mut i = 0;
    while i < NUM_PARTICLES {
        let mut j = 0;
        while j < NUM_PARTICLES {
            let x = (i as f32 + 0.5) / NUM_PARTICLES as f32 * 0.9;
            let y = (j as f32 + 0.5) / NUM_PARTICLES as f32 * 0.9;
            pos[idx] = [x, y, -x, -y, x, -y, -x, y];
            idx += 1;
            j += 1;
        }
        i += 1;
    }
    pos
}

pub fn get_render_callback_data(
    st: &mut AppState,
    canvas_size: Vec2,
    live: bool,
    fps: usize,
) -> RendererCallback {
    const MAX_RANGE: f32 = 0.95;
    const DAMP_FACTOR: f32 = 1.25;
    let now = Instant::now();
    let dat = RendererCallback {
        canvas_size,
        gen_kind: st.gen_kind,

        // stereometer params
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

        // Fluidwave params
        uniform_color: st.fwave.uniform_color,
        color_mode: st.fwave.color_mode,
        energy_transfer_mode: st.fwave.energy_transfer_mode,
        force_direction: st.fwave.force_direction,
        frame_time: if live {
            now.duration_since(st.fwave.last_frame)
                .as_secs_f32()
                .min(0.016)
                / 8.0
        } else {
            (1. / fps as f32) / 8.0
        },
        particle_pos: st
            .fwave
            .envelope_last_sample
            .div(
                if matches!(
                    st.fwave.energy_transfer_mode,
                    EnergyTransferMode::ForceField
                ) {
                    100.0 / st.fwave.range
                } else {
                    1.25
                },
            )
            .powf(DAMP_FACTOR)
            .min((100.0 / st.fwave.envelope_sensitivity) * MAX_RANGE)
            + ((1.0 - 100.0 / st.fwave.envelope_sensitivity) * MAX_RANGE),
        gravity: st.fwave.gravity,
        pressure_multiplier: st.fwave.pressure_multiplier,
        target_density: st.fwave.target_density,
        smoothing_radius: st.fwave.smoothing_radius,
        edge_damping_factor: st.fwave.edge_damping_factor,
        near_pressure_multiplier: st.fwave.near_pressure_multiplier,
        viscosity_amount: st.fwave.viscosity_amount,
        point_size: st.fwave.point_size,
        vignette: st.fwave.vignette,
    };
    st.fwave.last_frame = now;
    dat
}

fn effects_callback(ui: &mut egui::Ui, st: &mut AppState, rect: egui::Rect) {
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        EffectsCallback {
            top_left: rect.left_top(),
            bloom_amt: match st.gen_kind {
                GeneratorKind::Stereometer => st.stereo.bloom,
                GeneratorKind::Fluidwave => st.fwave.bloom,
            },
        },
    ));
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
        GeneratorKind::Fluidwave => envelope_follower(pl, st, true, 0),
    }

    let rcb_dat = get_render_callback_data(st, rect.size(), true, 0);
    ui.painter()
        .add(egui_wgpu::Callback::new_paint_callback(rect, rcb_dat));
    effects_callback(ui, st, rect);
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        OutputCallback,
    ));
}
