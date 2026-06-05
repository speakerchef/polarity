use crate::ui::palette;

pub fn draw(ui: &mut egui::Ui) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(palette::VOID))
        .show_inside(ui, |_| {});
}
