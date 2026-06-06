use crate::ui::palette as plt;
use egui::{Mesh, Rect, pos2, vec2};

use crate::ui::palette;

pub fn draw(ui: &mut egui::Ui) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(palette::VOID))
        .show_inside(ui, |ui| {
            let mut mesh = Mesh::default();
            mesh.add_colored_rect(
                Rect::from_min_size(
                    pos2(ui.available_width() / 2.0, ui.available_height() / 2.0),
                    vec2(100.0, 100.0),
                ),
                plt::DANGER,
            );
            ui.painter().add(egui::Shape::mesh(mesh));
        });
}
