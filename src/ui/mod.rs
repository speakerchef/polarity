use egui::{Align, Color32, FontId, Pos2};

pub mod canvas;
pub mod control_panel;
pub mod control_panel_widgets;
pub mod palette;
pub mod theme;
pub mod timeline;
pub mod timeline_widgets;

pub fn custom_text(
    ui: &mut egui::Ui,
    text: &str,
    font_id: FontId,
    pos: Pos2,
    extra_letter_spacing: f32,
    color: Color32,
    justify: Align,
) {
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
    ui.painter().galley(pos, galley, Color32::default());
}
