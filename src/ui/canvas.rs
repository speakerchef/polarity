#![allow(dead_code, unused)]
use std::ops::Div;
use std::time::{Duration, Instant};

use eframe::egui::{self, Pos2, Vec2};
use eframe::egui::{Align, Color32, FontId, StrokeKind, pos2, vec2};
use eframe::egui_wgpu;

use crate::generators::fluidwave::{EnergyTransferMode, ModSrc};
use crate::generators::rendering::{EffectsCallback, OutputCallback, RendererCallback};
use crate::traits::Generator;
use crate::ui::{SHARP, canvas_widgets::fullscreen_button, timeline_widgets::border};
use crate::{audio::audio_player::AudioPlayer, state::AppState};

use crate::ui::{custom_text, palette};

const MAX_VIGNETTE: f32 = 0.5;

pub fn draw(ui: &mut egui::Ui, st: &mut AppState, pl: Option<&AudioPlayer>) {
    ui.ctx().request_repaint();
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(palette::VOID(ui.style().visuals.dark_mode))
                .inner_margin(0.0)
                .outer_margin(0.0),
        )
        .show(ui, |ui| {
            if ui.ctx().input(
                |i| i.pointer.hover_pos().is_some(), /* is cursor on window */
            ) || !st.bool.fullscreen
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

fn custom_painting(ui: &mut egui::Ui, st: &mut AppState, pl: Option<&AudioPlayer>) {
    let (h, w) = (ui.available_height(), ui.available_width());
    // let l = h.min(w) * 0.99;
    let l = h.min(w);
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
    st.env_bank.run_follower(pl, None);

    let mut fbank = std::mem::take(&mut st.filterbank);
    let env_bank = std::mem::take(&mut st.env_bank);

    st.active_gen().prepare(&mut fbank, &env_bank, pl, None);

    st.filterbank = fbank;
    st.env_bank = env_bank;

    let renderer_params = st.build_renderer_callback_params(true, 0);
    let efx_params = st.build_effects_callback_params();
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        RendererCallback {
            canvas_size,
            params: renderer_params,
        },
    ));
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        EffectsCallback {
            top_left: rect.left_top(),
            ..efx_params
        },
    ));
}
