#![allow(dead_code, unused)]
use eframe::egui;
use eframe::egui::{Align, Color32, FontId, StrokeKind, pos2, vec2};
use eframe::egui_wgpu;

use crate::generators::rendering::{EffectsCallback, OutputCallback, RendererCallback};
use crate::ui::canvas_widgets::fullscreen_button;
use crate::ui::timeline_widgets::{SHARP, border};
use crate::{audio::audio_player::AudioPlayer, state::AppState};

use crate::ui::{custom_text, palette};

pub fn draw(ui: &mut egui::Ui, st: &mut AppState, pl: &Option<AudioPlayer>) {
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
    st.stereo.draw(pl, None);
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        RendererCallback {
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
            canvas_size,
        },
    ));
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        EffectsCallback {
            top_left: rect.left_top(),
            bloom_amt: st.stereo.bloom,
        },
    ));

    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        OutputCallback,
    ));
}
