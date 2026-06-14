#![allow(dead_code, unused)]
use eframe::egui_wgpu;
use egui::{Color32, pos2, vec2};

use crate::generators::stereometer::{EffectsCallback, RendererCallback};
use crate::{audio::audio_player::AudioPlayer, state::AppState};

use crate::ui::palette;

fn custom_painting(ui: &mut egui::Ui, st: &mut AppState, pl: &Option<AudioPlayer>) {
    let (h, w) = (ui.available_height(), ui.available_width());
    let l = h.min(w);
    let canvas = vec2(l, l);

    let center = ui.max_rect().center();
    let top_left = pos2(center.x - l / 2.0, center.y - l / 2.0);
    let rect = ui
        .allocate_rect(
            egui::Rect::from_min_size(top_left, canvas),
            egui::Sense::click(),
        )
        .rect;
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(6, 10, 10, 255),
    );
    let Some(pl) = pl else {
        return;
    };

    let (live_pos, trace_pos) = st.stereo.draw(pl, canvas);

    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        RendererCallback {
            live_pos,
            trace_pos,
            color: st.stereo.fs_color.into(),
            canvas_size: canvas,
        },
    ));
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        EffectsCallback {
            top_left: rect.left_top(),
        },
    ));
}

pub fn draw(ui: &mut egui::Ui, st: &mut AppState, pl: &Option<AudioPlayer>) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(palette::VOID)
                .inner_margin(0.0)
                .outer_margin(0.0),
        )
        .show_inside(ui, |ui| {
            custom_painting(ui, st, pl);
        });
}
