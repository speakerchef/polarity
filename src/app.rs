use crate::ui::control_panel;
use egui::{CornerRadius, Stroke, StrokeKind, emath::GuiRounding};

use crate::{state::PanelState, ui::palette as plt, ui::theme};

#[derive(Default)]
pub struct PolarityApp {
    state: PanelState,
}

impl PolarityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::apply_theme(&cc.egui_ctx);
        Self::default()
    }
}

impl eframe::App for PolarityApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let resp = egui::CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(12.0).fill(plt::BG))
            .show_inside(ui, |ui| {
                control_panel::draw(ui, &mut self.state);
            });

        let border_rect = resp.response.rect.shrink(12.0).round_ui();
        ui.painter().rect_stroke(
            border_rect,
            CornerRadius::ZERO,
            Stroke {
                width: 1.0,
                color: plt::BORDER,
            },
            StrokeKind::Inside,
        );
    }
}
