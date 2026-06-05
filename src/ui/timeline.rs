use crate::state::*;
use crate::ui::{palette as plt, widgets::*};

pub fn draw(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Panel::bottom("timeline")
        .exact_size(104.0)
        .resizable(false)
        .frame(egui::Frame::NONE.fill(plt::BG))
        .show_inside(ui, |ui| {
            ui.label("GOOBER");
        });
}
