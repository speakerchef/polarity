use std::ops::Sub;

use crate::{
    state::{AppState, ExportQuality, Fps, Resolution},
    ui::{
        control_panel_widgets::dropdown_row,
        custom_text, palette as plt,
        timeline_widgets::{SHARP, border},
    },
};
use eframe::egui;
use eframe::egui::{Align, FontId, Key, Pos2, StrokeKind, Vec2, vec2};

pub fn export_modal_button(
    ui: &mut egui::Ui,
    top_left: Pos2,
    size: Vec2,
    button_label: &str,
    text_col: egui::Color32,
    control: &mut bool,
) {
    let resp = ui.allocate_rect(
        egui::Rect::from_min_size(top_left, size),
        egui::Sense::click(),
    );
    let rect = resp.rect;
    let (bg, fg) = if resp.hovered() {
        (plt::SURFACE_HOVER, text_col)
    } else {
        (plt::BG, text_col)
    };
    ui.painter().rect_filled(rect, SHARP, bg);
    ui.painter()
        .rect_stroke(rect, SHARP, border(), egui::StrokeKind::Inside);

    let font = FontId {
        size: plt::font_size::MED,
        family: egui::FontFamily::Name("inter_medium".into()),
    };
    custom_text(
        ui,
        button_label,
        font,
        rect.center() - vec2(0.0, rect.height() / 4.0),
        plt::letter_spacing::BASE,
        fg,
        Align::Center,
    );

    if resp.clicked() {
        *control = !*control;
    }
}
pub fn export_modal(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Area::new("Export Modal".into())
        .default_size(vec2(100., 50.))
        .order(egui::Order::Foreground)
        .movable(false)
        .show(ui.ctx(), |ui| {
            let (w, h) = (
                ui.viewport_rect().width() / 2.0,
                ui.viewport_rect().height() / 2.0,
            );
            const ASPECT_RATIO: f32 = 16.0 / 10.0;
            let (rw, rh) = if w / h > ASPECT_RATIO {
                (h * ASPECT_RATIO, h)
            } else {
                (w, w / ASPECT_RATIO)
            };
            let (rw, rh) = (rw.min(540.), rh.min(360.));
            let (rw, rh) = (rw.max(360.), rh.max(240.));

            let resp = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.content_rect().center().sub(vec2(rw / 2.0, rh / 2.0)),
                    vec2(rw, rh),
                ),
                egui::Sense::click(),
            );

            ui.painter().rect_filled(resp.rect, SHARP, plt::BG);

            ui.scope_builder(egui::UiBuilder::new().max_rect(resp.rect), |ui| {
                dropdown_row(
                    ui,
                    "resolution",
                    "Resolution",
                    &mut st.export_config.resolution,
                    Resolution::ALL,
                    &mut st.show_export_resolution,
                );
                dropdown_row(
                    ui,
                    "fps",
                    "FPS",
                    &mut st.export_config.frame_rate,
                    Fps::ALL,
                    &mut st.show_export_fps,
                );
                dropdown_row(
                    ui,
                    "quality",
                    "Quality",
                    &mut st.export_config.quality,
                    ExportQuality::ALL,
                    &mut st.show_export_quality,
                );
            });

            ui.painter()
                .rect_stroke(resp.rect, SHARP, border(), StrokeKind::Inside);

            const BUTTON_H: f32 = 30.0;
            export_modal_button(
                ui,
                resp.rect.left_bottom() - vec2(0.0, BUTTON_H),
                vec2(resp.rect.width() / 2.0, 30.0),
                "CANCEL",
                plt::WARN,
                &mut st.show_export_modal,
            );
            export_modal_button(
                ui,
                resp.rect.right_bottom() - vec2(resp.rect.width() / 2.0, BUTTON_H),
                vec2(resp.rect.width() / 2.0, 30.0),
                "EXPORT",
                plt::LIVE,
                &mut st.start_render,
            );
            if ui.ctx().input(|i| i.key_pressed(Key::Escape)) {
                st.show_export_modal = false;
            }
        });
}
