use std::sync::Arc;

use eframe::egui::{self, CornerRadius, Vec2};
use eframe::egui::{Align, Color32, FontId, Pos2};

pub mod app_widgets;
pub mod canvas;
pub mod canvas_widgets;
pub mod control_panel;
pub mod control_panel_widgets;
pub mod palette;
pub mod theme;
pub mod timeline;
pub mod timeline_widgets;
pub const SHARP: CornerRadius = CornerRadius::ZERO;
pub const ROUND_MAX: CornerRadius = CornerRadius {
    nw: 100,
    ne: 100,
    sw: 100,
    se: 100,
};

pub fn get_text_size(ui: &mut egui::Ui, text: &str, font_id: FontId) -> Vec2 {
    let galley = ui.fonts_mut(|f| f.layout_no_wrap(text.to_string(), font_id, Color32::default()));
    galley.size()
}

pub fn custom_text(
    ui: &mut egui::Ui,
    text: &str,
    font_id: FontId,
    pos: Pos2,
    extra_letter_spacing: f32,
    color: Color32,
    justify: Align,
) -> Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob {
        halign: justify,
        ..Default::default()
    };
    job.append(
        text,
        0.0,
        egui::TextFormat {
            font_id,
            extra_letter_spacing,
            line_height: None,
            color,
            ..Default::default()
        },
    );
    let galley = ui.painter().layout_job(job);
    ui.painter().galley(pos, galley.clone(), Color32::default());
    galley
}
