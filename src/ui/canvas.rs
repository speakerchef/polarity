#![allow(dead_code, unused)]
use std::ops::Div;
use std::time::{Duration, Instant};

use eframe::egui::{self, Pos2, Vec2};
use eframe::egui::{Align, Color32, FontId, StrokeKind, pos2, vec2};
use eframe::egui_wgpu;

use crate::GenKindLabel;
use crate::generators::DAMP_FACTOR;
use crate::generators::fluidwave::EnergyTransferMode;
use crate::generators::rendering::{EffectsCallback, OutputCallback, RendererCallback};
use crate::ui::{SHARP, canvas_widgets::fullscreen_button, timeline_widgets::border};
use crate::{audio::audio_player::AudioPlayer, state::AppState};

use crate::ui::{custom_text, palette};

pub const TARGET_FPS: f32 = 30.0;
pub const SUBSTEP_DIV: f32 = 6.0;
pub const MIN_SUBSTEP_DIV: f32 = 3.0;
pub const TARGET_DT: f32 = 1. / TARGET_FPS / SUBSTEP_DIV;

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
    const MAX_FRAME_TIME: f32 = 1. / 12. / SUBSTEP_DIV;
    let sim_speed_scale = 100.0 / st.fwave.sim_speed.max(1.0);
    let sim_speed = (sim_speed_scale * SUBSTEP_DIV) /* higher == slower */
        .clamp(MIN_SUBSTEP_DIV, 100.0)
        .round();
    let now = Instant::now();

    let frame_time = if live {
        now.duration_since(st.fwave.last_frame).as_secs_f32() / sim_speed
    } else {
        1. / fps as f32 / sim_speed
    };
    st.fwave.frame_time_accumulator += frame_time.min(MAX_FRAME_TIME);
    let dat = RendererCallback {
        canvas_size,
        gen_kind: st.gen_kind,

        /* stereometer params */
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

        /* Fluidwave params */
        uniform_color: st.fwave.uniform_color,
        color_mode: st.fwave.color_mode,
        energy_transfer_mode: st.fwave.energy_transfer_mode,
        force_direction: st.fwave.force_direction,
        frame_time_accumulator: st.fwave.frame_time_accumulator,
        particle_pos: st.fwave.env.envelope(),
        gravity: st.fwave.gravity,

        substeps: sim_speed,
        pressure_multiplier: st.fwave.pressure_multiplier
            - if st.fwave.envelope_pressure_link {
                (400.0 * st.fwave.env.envelope().powf(DAMP_FACTOR))
            } else {
                0.0
            },

        target_density: st.fwave.target_density,
        smoothing_radius: st.fwave.smoothing_radius,
        edge_damping_factor: st.fwave.edge_damping_factor,
        near_pressure_multiplier: st.fwave.near_pressure_multiplier,
        viscosity_amount: st.fwave.viscosity_amount,
        point_size: st.fwave.point_size,
        vignette: st.fwave.vignette,
        color_arrangement: st.fwave.color_arrangement,
        color_invert: st.fwave.color_invert,
        luminance_mode: st.fwave.luminance_mode,
        luminance_floor: st.fwave.luminance_floor,
    };
    st.fwave.last_frame = now;
    st.fwave.frame_time_accumulator %= TARGET_DT; // leftover frametime
    dat
}

fn effects_callback(ui: &mut egui::Ui, st: &mut AppState, rect: egui::Rect) {
    const MAX_VIGNETTE: f32 = 0.5;
    let (bloom_amt, vignette) = match st.gen_kind {
        GenKindLabel::Stereometer => (st.stereo.bloom, st.stereo.vignette * MAX_VIGNETTE),
        GenKindLabel::Fluidwave => (st.fwave.bloom, st.fwave.vignette * MAX_VIGNETTE),
    };
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        EffectsCallback {
            top_left: rect.left_top(),
            bloom_amt,
            vignette,
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
    let frame_time = Instant::now()
        .duration_since(st.fwave.last_frame)
        .as_secs_f32()
        .max(1. / 60.);

    st.active_gen().prepare(pl, None);
    let rcb_dat = get_render_callback_data(st, rect.size(), true, 0);
    ui.painter()
        .add(egui_wgpu::Callback::new_paint_callback(rect, rcb_dat));
    effects_callback(ui, st, rect);
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        OutputCallback,
    ));
}
