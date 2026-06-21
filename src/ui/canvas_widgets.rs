use eframe::egui;
use eframe::egui::{Align, FontId, pos2, vec2};

use crate::{
    state::AppState,
    ui::{custom_text, palette},
};

pub fn fullscreen_button(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Area::new("fullscreen_button".into())
        .movable(false)
        .show(ui.ctx(), |ui| {
            let resp = ui.allocate_rect(
                egui::Rect::from_min_size(
                    pos2(
                        ui.viewport_rect().left_top().x + if st.fullscreen { 16.0 } else { 22.0 },
                        ui.viewport_rect().left_top().y + if st.fullscreen { 17.0 } else { 44.0 },
                    ),
                    vec2(20., 20.),
                ),
                egui::Sense::click(),
            );
            custom_text(
                ui,
                "\u{e5d0}",
                FontId {
                    size: palette::font_size::ICON + 6.0,
                    family: egui::FontFamily::Name("icons".into()),
                },
                pos2(
                    resp.rect.left_center().x - 1.0,
                    resp.rect.center_top().y - 3.5,
                ),
                palette::letter_spacing::BASE,
                if resp.hovered() {
                    palette::YELLO
                } else {
                    palette::BORDER(ui.style().visuals.dark_mode)
                },
                Align::LEFT,
            );
            if resp.clicked() {
                st.fullscreen = !st.fullscreen;
                st.window_drag_tooltip_modal_deadline.take();
            }
        });
}
