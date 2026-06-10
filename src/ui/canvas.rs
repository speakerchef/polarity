#![allow(dead_code, unused)]
use eframe::egui_wgpu;
use egui::pos2;

use crate::generators::stereometer::CustomStereometerCallback;
use crate::{audio::audio_player::AudioPlayer, state::AppState};

use crate::ui::palette;

fn custom_painting(ui: &mut egui::Ui, st: &mut AppState, pl: &AudioPlayer) {
    let mut canvas = ui.available_size();
    let min = canvas.x.min(canvas.y);
    let center = pos2(ui.available_width() / 2.0, ui.available_height() / 2.0);
    canvas.x = canvas.x.clamp(0.0, 0.9 * min);
    canvas.y = canvas.y.clamp(0.0, 0.9 * min);
    let rect = ui
        .allocate_rect(
            egui::Rect::from_min_size(
                pos2(center.x - canvas.x / 2.2, center.y - canvas.y / 2.2),
                canvas,
            ),
            egui::Sense::click(),
        )
        .rect;

    let (live_pos, trace_pos) = st.stereo.draw(pl, canvas);

    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        CustomStereometerCallback {
            live_pos,
            trace_pos,
            color: st.stereo.fs_color.into(),
        },
    ));
}

pub fn draw(ui: &mut egui::Ui, st: &mut AppState, pl: &Option<AudioPlayer>) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(palette::VOID))
        .show_inside(ui, |ui| {
            let Some(p) = pl else {
                return;
            };
            custom_painting(ui, st, p);
        });
}
