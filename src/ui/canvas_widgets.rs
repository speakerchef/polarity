use eframe::egui;
use eframe::egui::{Align, FontId, pos2, vec2};
use egui_winit::winit::dpi::PhysicalSize;

use crate::{
    state::AppState,
    ui::{custom_text, palette},
};

pub fn presentation_buttons(ui: &mut egui::Ui, st: &mut AppState, frame: &eframe::Frame) {
    egui::Area::new("presentation_buttons".into())
        .movable(false)
        .show(ui.ctx(), |ui| {
            let fs_resp = ui.allocate_rect(
                egui::Rect::from_min_size(
                    pos2(
                        ui.viewport_rect().left_top().x
                            + if st.bool.fullscreen { 14.0 } else { 22.0 },
                        ui.viewport_rect().left_top().y
                            + if st.bool.fullscreen { 17.0 } else { 48.0 },
                    ),
                    vec2(20., 20.),
                ),
                egui::Sense::click(),
            );
            let aspect_resp = ui.allocate_rect(
                egui::Rect::from_min_size(
                    pos2(
                        ui.viewport_rect().left_top().x
                            + if st.bool.fullscreen { 14.0 } else { 22.0 },
                        ui.viewport_rect().left_top().y
                            + if st.bool.fullscreen { 53.0 } else { 83.0 },
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
                    fs_resp.rect.left_center().x - 1.0,
                    fs_resp.rect.center_top().y - 3.5,
                ),
                palette::letter_spacing::BASE,
                if fs_resp.hovered() {
                    palette::YELLO
                } else {
                    palette::BORDER(ui.style().visuals.dark_mode)
                },
                Align::LEFT,
            );

            custom_text(
                ui,
                if st.bool.lock_aspect_ratio {
                    "\u{e897}"
                } else {
                    "\u{f656}"
                },
                FontId {
                    size: palette::font_size::ICON + 6.0,
                    family: egui::FontFamily::Name("icons".into()),
                },
                pos2(
                    aspect_resp.rect.left_center().x - 1.0,
                    aspect_resp.rect.center_top().y - 3.5,
                ),
                palette::letter_spacing::BASE,
                if aspect_resp.hovered() {
                    palette::YELLO
                } else {
                    palette::BORDER(ui.style().visuals.dark_mode)
                },
                Align::LEFT,
            );

            if aspect_resp.clicked() {
                st.bool.lock_aspect_ratio = !st.bool.lock_aspect_ratio;
            }

            if fs_resp.clicked() {
                if let Some(win) = frame.winit_window() {
                    if !st.bool.fullscreen {
                        let cur_sz = win.inner_size();
                        st.last_editor_window_size = cur_sz;

                        let last_fs_sz = st.last_fullscreen_window_size;
                        let (w, h) = if last_fs_sz != PhysicalSize::new(0, 0) {
                            (last_fs_sz.width, last_fs_sz.height)
                        } else {
                            (cur_sz.width, cur_sz.height)
                        };
                        let (w, h) = if st.bool.lock_aspect_ratio {
                            (w.min(h), w.min(h))
                        } else {
                            (w, h)
                        };
                        let fs_size = PhysicalSize::new(w, h);
                        let _ = win.request_inner_size(fs_size);
                        st.last_fullscreen_window_size = fs_size;
                    } else {
                        st.last_fullscreen_window_size = win.inner_size();
                        let last_editor_sz = st.last_editor_window_size;
                        let (w, h) = (last_editor_sz.width, last_editor_sz.height);
                        let _ = win.request_inner_size(PhysicalSize::new(w, h));
                    }
                }

                st.bool.fullscreen = !st.bool.fullscreen;
            }
        });
}
