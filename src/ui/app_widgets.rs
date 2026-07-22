use std::{
    ops::{Add, Sub},
    time::{Duration, Instant},
};

use crate::{
    audio::audio_player::AudioPlayer,
    state::{AppState, ExportQuality, Fps, Resolution},
    ui::{
        SHARP, canvas, control_panel,
        control_panel_widgets::{dropdown_row, menu_bar_option, path_picker},
        custom_text, get_text_size, palette as plt, timeline,
        timeline_widgets::border,
    },
};
use eframe::egui::{self, pos2};
use eframe::egui::{Align, FontId, Key, Pos2, StrokeKind, Vec2, vec2};
use egui_winit::winit::dpi::LogicalSize;

const MB_H: f32 = 24.0;
const MB_GAP: f32 = 12.0;

pub fn menu_bar(st: &mut AppState, ui: &mut egui::Ui) {
    egui::MenuBar::new().ui(ui, |ui| {
        ui.set_min_height(MB_H + MB_GAP);
        egui::Area::new("menu_bar".into())
            .fixed_pos(ui.viewport_rect().left_top() + vec2(0.0, MB_GAP))
            .order(egui::Order::Foreground)
            .movable(false)
            .show(ui.ctx(), |ui| {
                ui.set_max_height(MB_H);
                ui.set_width(ui.content_rect().width());

                ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                    let resp = ui.allocate_rect(
                        egui::Rect::from_min_size(
                            pos2(
                                ui.content_rect().left_top().x,
                                ui.content_rect().left_top().y + 12.0,
                            ),
                            vec2(ui.available_width(), ui.available_height()),
                        ),
                        egui::Sense::focusable_noninteractive(),
                    );
                    let mut rect = resp.rect;
                    rect.min.y -= MB_GAP;
                    ui.painter()
                        .rect_filled(rect, SHARP, plt::BG(ui.style().visuals.dark_mode));

                    ui.set_max_width(ui.available_rect_before_wrap().width());
                    ui.set_min_width(ui.available_rect_before_wrap().width());
                    ui.add_space(12.0);

                    menu_bar_option(
                        ui,
                        "file",
                        None,
                        FontId {
                            family: egui::FontFamily::Name("inter_medium".into()),
                            size: plt::font_size::TINY,
                        },
                        &mut st.bool.show_file_options,
                        &["Import", "Export"],
                        &mut [&mut st.bool.import_open, &mut st.bool.show_export_modal],
                        MB_H,
                        false,
                    );
                    ui.add_space(1.0);

                    menu_bar_option(
                        ui,
                        "presets",
                        None,
                        FontId {
                            family: egui::FontFamily::Name("inter_medium".into()),
                            size: plt::font_size::TINY,
                        },
                        &mut st.bool.show_preset_options,
                        &["Save", "Load"],
                        &mut [
                            &mut st.bool.show_preset_save_modal,
                            &mut st.bool.show_preset_load_modal,
                        ],
                        MB_H,
                        false,
                    );

                    ui.add_space(ui.available_width() - 37.0);
                    menu_bar_option(
                        ui,
                        "\u{e8b8}",
                        Some(25.0),
                        FontId {
                            family: egui::FontFamily::Name("icons".into()),
                            size: plt::font_size::TINY,
                        },
                        &mut st.bool.show_settings,
                        &[
                            &format!(
                                "{} Theme",
                                if st.bool.dark_mode {
                                    "Graphite"
                                } else {
                                    "Midnight"
                                }
                            ),
                            &format!(
                                "{} Mode",
                                if st.bool.advanced_mode {
                                    "Simple"
                                } else {
                                    "Advanced"
                                }
                            ),
                        ],
                        &mut [&mut st.bool.dark_mode, &mut st.bool.advanced_mode],
                        MB_H,
                        true,
                    );
                });
            });
    });
}

pub fn main_window(
    ui: &mut egui::Ui,
    st: &mut AppState,
    player: &mut Option<AudioPlayer>,
    frame: &eframe::Frame,
) {
    let mut resp = egui::CentralPanel::default()
        .frame(
            egui::Frame::NONE
                .inner_margin(if !st.bool.fullscreen {
                    egui::Margin {
                        bottom: 12,
                        left: 12,
                        right: 12,
                        top: 0,
                    }
                } else {
                    0.into()
                })
                .fill(plt::BG(st.bool.dark_mode)),
        )
        .show_inside(ui, |ui| {
            if !st.bool.fullscreen {
                timeline::draw(ui, st, player);
                control_panel::draw(ui, st);
                editor_window_behavior(frame);
            } else {
                ui.request_repaint_after(Duration::from_millis(16));
                show_window_drag_tooltip_modal(st, ui);
                fullscreen_window_behavior(ui, frame);
            }

            if st.bool.show_export_modal {
                export_modal(ui, st);
            }
            if st.bool.show_preset_load_modal || st.bool.show_preset_save_modal {
                preset_modal(ui, st);
            }

            if st.bool.rendering {
                player.as_ref().unwrap().pause();
            } else {
                canvas::draw(ui, st, player);
            }
        })
        .response;

    if !st.bool.fullscreen {
        resp.rect = resp.rect.translate(vec2(0., -6.));
        resp.rect = resp.rect.shrink2(vec2(12.0, 6.0));
        ui.painter().rect_stroke(
            resp.rect,
            SHARP,
            border(ui.style().visuals.dark_mode),
            StrokeKind::Inside,
        );
    }
}

fn show_window_drag_tooltip_modal(st: &mut AppState, ui: &mut egui::Ui) {
    if st.window_drag_tooltip_modal_deadline.is_none() {
        st.window_drag_tooltip_modal_deadline = Some(Instant::now());
        st.bool.window_drag_tooltip_modal_open = true;
    }

    if st.bool.window_drag_tooltip_modal_open {
        window_drag_tooltip(ui);
    }

    if let Some(start_time) = st.window_drag_tooltip_modal_deadline
        && Instant::now().duration_since(start_time) >= Duration::from_secs(5)
    {
        st.bool.window_drag_tooltip_modal_open = false;
    }
}

fn editor_window_behavior(frame: &eframe::Frame) {
    frame.winit_window().unwrap().set_decorations(true);
    frame
        .winit_window()
        .unwrap()
        .set_min_inner_size(Some(LogicalSize::new(720.0, 480.0)));
}

fn fullscreen_window_behavior(ui: &mut egui::Ui, frame: &eframe::Frame) {
    // remove titlebar and corners
    frame.winit_window().unwrap().set_decorations(false);
    frame
        .winit_window()
        .unwrap()
        .set_min_inner_size(Some(LogicalSize::new(240.0, 240.0)));

    // allow drag anywhere if any key is down
    if ui.ctx().input(|i| {
        (!i.keys_down.is_empty() && !i.key_down(Key::Space))
            || (i.modifiers.matches_logically(egui::Modifiers::COMMAND)
                || i.modifiers.matches_logically(egui::Modifiers::SHIFT)
                || i.modifiers.matches_logically(egui::Modifiers::ALT))
    }) {
        frame
            .winit_window()
            .unwrap()
            .drag_window()
            .unwrap_or_default();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn modal_button(
    ui: &mut egui::Ui,
    top_left: Pos2,
    size: Vec2,
    button_label: &str,
    font_size: f32,
    text_col: egui::Color32,
    control: &mut bool,
    is_icon: bool,
) {
    let resp = ui.allocate_rect(
        egui::Rect::from_min_size(top_left, size),
        egui::Sense::click(),
    );
    let rect = resp.rect;
    let bg = if resp.hovered() {
        plt::SURFACE_HOVER(ui.style().visuals.dark_mode)
    } else {
        plt::BG(ui.style().visuals.dark_mode)
    };
    ui.painter().rect_filled(rect, SHARP, bg);
    ui.painter().rect_stroke(
        rect,
        SHARP,
        border(ui.style().visuals.dark_mode),
        egui::StrokeKind::Middle,
    );

    let font = FontId {
        size: font_size,
        family: if is_icon {
            egui::FontFamily::Name("icons".into())
        } else {
            egui::FontFamily::Name("inter_medium".into())
        },
    };
    let (_, h) = get_text_size(ui, button_label, font.clone()).into();

    custom_text(
        ui,
        button_label,
        font,
        rect.center() - vec2(0.0, h / 2.0),
        plt::letter_spacing::MINIMAL,
        text_col,
        Align::Center,
    );

    if resp.clicked() {
        *control = !*control;
    }
}
pub fn export_modal(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Area::new("Export Modal".into())
        .order(egui::Order::Foreground)
        .movable(false)
        .show(ui.ctx(), |ui| {
            let (w, h) = (
                ui.viewport_rect().width() / 2.0,
                ui.viewport_rect().height() / 2.0,
            );
            let aspect_ratio = 16.0 / 9.0;
            let (rw, rh) = if w / h > aspect_ratio {
                (h * aspect_ratio, h)
            } else {
                (w, w / aspect_ratio)
            };
            let (rw, rh) = (rw.min(360.), rh.min(180.));

            let resp = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.content_rect().center().sub(vec2(rw / 2.0, rh / 2.0)),
                    vec2(rw, rh),
                ),
                egui::Sense::click(),
            );

            ui.painter()
                .rect_filled(resp.rect, SHARP, plt::BG(ui.visuals().dark_mode));

            const BUTTON_H: f32 = 30.0;
            if !st.bool.rendering {
                ui.scope_builder(egui::UiBuilder::new().max_rect(resp.rect), |ui| {
                    if !st.bool.export_enabled {
                        ui.painter().rect_stroke(
                            resp.rect,
                            SHARP,
                            border(ui.visuals().dark_mode),
                            StrokeKind::Middle,
                        );
                        custom_text(
                            ui,
                            "Export not enabled on this device; Missing 'ffmpeg'. \nEither install manually or use live mode.",
                            FontId {
                                size: plt::font_size::MED,
                                family: egui::FontFamily::Name("inter_regular".into()),
                            },
                            resp.rect.center() - vec2(0.0, BUTTON_H),
                            plt::letter_spacing::MINIMAL,
                            plt::YELLO,
                            Align::Center,
                        );
                        return;
                    }
                    dropdown_row(
                        ui,
                        "Resolution",
                        &mut st.export_config.resolution,
                        Resolution::ALL,
                        &mut st.bool.show_export_resolution,
                        false,
                    );
                    dropdown_row(
                        ui,
                        "FPS",
                        &mut st.export_config.frame_rate,
                        Fps::ALL,
                        &mut st.bool.show_export_fps,
                        false,
                    );
                    let path_rect = dropdown_row(
                        ui,
                        "Quality",
                        &mut st.export_config.quality,
                        ExportQuality::ALL,
                        &mut st.bool.show_export_quality,
                        false,
                    )
                    .rect;

                    path_picker(
                        ui,
                        st.export_path.as_ref(),
                        path_rect.left_bottom(),
                        rw,
                        "Export Location:",
                        &mut st.bool.open_export_path_picker,
                        true,
                    );
                });

                modal_button(
                    ui,
                    resp.rect.left_bottom() - vec2(0.0, BUTTON_H),
                    vec2(resp.rect.width() / 2.0, 30.0),
                    "CANCEL",
                    plt::font_size::BODY,
                    plt::TEXT,
                    &mut st.bool.show_export_modal,
                    false,
                );
                modal_button(
                    ui,
                    resp.rect.right_bottom() - vec2(resp.rect.width() / 2.0, BUTTON_H),
                    vec2(resp.rect.width() / 2.0, 30.0),
                    "EXPORT",
                    plt::font_size::BODY,
                    plt::LIVE,
                    &mut st.bool.start_render,
                    false,
                );

                if ui.ctx().input(|i| i.key_pressed(Key::Escape)) {
                    st.bool.show_export_modal = false;
                }
            } else {
                ui.scope_builder(egui::UiBuilder::new().max_rect(resp.rect), |ui| {
                    let uirect = ui.available_rect_before_wrap();
                    ui.painter().rect_stroke(
                        uirect,
                        SHARP,
                        border(ui.visuals().dark_mode),
                        StrokeKind::Outside,
                    );
                    const BAR_H: f32 = 30.0;
                    let bar_w: f32 = uirect.width() / 1.25;
                    let pbar_resp = ui.allocate_rect(
                        egui::Rect::from_min_size(
                            uirect.left_center() + vec2((uirect.width() - bar_w) / 2.0, -BAR_H),
                            vec2(bar_w, BAR_H),
                        ),
                        egui::Sense::focusable_noninteractive(),
                    );
                    ui.painter().rect_filled(pbar_resp.rect, SHARP, plt::BLACK);
                    ui.painter().rect_stroke(
                        resp.rect,
                        SHARP,
                        border(ui.visuals().dark_mode),
                        StrokeKind::Outside,
                    );

                    let font_id = FontId {
                        size: plt::font_size::MED,
                        family: egui::FontFamily::Name("inter_regular".into()),
                    };

                    custom_text(
                        ui,
                        "PROGRESS",
                        font_id,
                        pbar_resp.rect.left_top()
                            + vec2(
                                pbar_resp.rect.width() / 2.0,
                                -pbar_resp.rect.height() / 1.25,
                            ),
                        plt::letter_spacing::BASE,
                        plt::DIM,
                        Align::Center,
                    );

                    ui.scope_builder(egui::UiBuilder::new().max_rect(pbar_resp.rect), |ui| {
                        let (rect, _) = ui.allocate_exact_size(
                            vec2(
                                (st.cur_frame_idx as f32 / st.export_config.total_frames as f32)
                                    .max(0.0)
                                    * pbar_resp.rect.width(),
                                BAR_H,
                            ),
                            egui::Sense::focusable_noninteractive(),
                        );
                        ui.painter().rect_filled(rect, SHARP, plt::LIVE);
                        ui.painter().rect_stroke(
                            pbar_resp.rect,
                            SHARP,
                            border(ui.style().visuals.dark_mode),
                            StrokeKind::Inside,
                        );
                    });

                    custom_text(
                        ui,
                        "Time Elapsed:",
                        FontId {
                            size: plt::font_size::META,
                            family: egui::FontFamily::Name("inter_regular".into()),
                        },
                        pbar_resp.rect.left_bottom() + vec2(0.0, 4.0),
                        plt::letter_spacing::MINIMAL,
                        plt::DIM,
                        Align::LEFT,
                    );

                    let t = st.export_elapsed_time.unwrap_or_default();
                    let (h, m, s) = (
                        t.as_secs_f64() / 3600.0,
                        t.as_secs_f64() / 60.0,
                        t.as_secs_f64() % 60.0,
                    );
                    let (h, m, s) = (h as u32, m as u32, s as u32);
                    custom_text(
                        ui,
                        &format!("{h:02}:{m:02}:{s:02}"),
                        FontId {
                            size: plt::font_size::META,
                            family: egui::FontFamily::Name("inter_regular".into()),
                        },
                        pbar_resp.rect.right_bottom() + vec2(0.0, 4.0),
                        plt::letter_spacing::MINIMAL,
                        plt::DIM,
                        Align::RIGHT,
                    );
                    modal_button(
                        ui,
                        pbar_resp.rect.center() + vec2(-36.5, pbar_resp.rect.height() * 1.75),
                        vec2(70.0, 24.0),
                        "Cancel",
                        plt::font_size::META,
                        plt::TEXT,
                        &mut st.bool.export_canceled,
                        false,
                    );
                });
            }
        });
}

pub fn preset_modal(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Area::new("Export Modal".into())
        .default_size(vec2(100., 50.))
        .order(egui::Order::Foreground)
        .movable(false)
        .show(ui.ctx(), |ui| {
            let (w, h) = (
                ui.viewport_rect().width() / 2.0,
                ui.viewport_rect().height() / 2.0,
            );
            let aspect_ratio = 16.0 / 9.0;
            let (rw, rh) = if w / h > aspect_ratio {
                (h * aspect_ratio, h)
            } else {
                (w, w / aspect_ratio)
            };
            let (rw, rh) = (rw.min(360.), rh.min(180.));

            let resp = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.content_rect().center().sub(vec2(rw / 2.0, rh / 4.0)),
                    vec2(rw, 0.0),
                ),
                egui::Sense::click(),
            );
            const BUTTON_H: f32 = 30.0;
            const OPTION_H: f32 = 37.0;
            const INNER_H: f32 = 21.0;
            ui.painter().rect_stroke(
                resp.rect,
                SHARP,
                border(ui.style().visuals.dark_mode),
                StrokeKind::Inside,
            );
            if st.bool.show_preset_save_modal {
                path_picker(
                    ui,
                    st.preset_save_path.as_ref(),
                    resp.rect.left_top(),
                    resp.rect.width(),
                    "Save Location:",
                    &mut st.bool.open_preset_save_file_picker,
                    true,
                );

                modal_button(
                    ui,
                    resp.rect.left_bottom() - vec2(0.0, BUTTON_H),
                    vec2(resp.rect.width() / 2.0, 30.0),
                    "CANCEL",
                    plt::font_size::BODY,
                    plt::DANGER,
                    &mut st.bool.show_preset_save_modal,
                    false,
                );
                modal_button(
                    ui,
                    resp.rect.right_bottom() - vec2(resp.rect.width() / 2.0, BUTTON_H),
                    vec2(resp.rect.width() / 2.0, 30.0),
                    "SAVE",
                    plt::font_size::BODY,
                    plt::LIVE,
                    &mut st.bool.save_preset,
                    false,
                );
                if ui.ctx().input(|i| i.key_pressed(Key::Escape)) {
                    st.bool.show_preset_save_modal = false;
                }
            } else {
                ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                    let path_rect = ui
                        .allocate_rect(
                            egui::Rect::from_min_size(
                                resp.rect.left_top(),
                                vec2(resp.rect.width(), OPTION_H),
                            ),
                            egui::Sense::focusable_noninteractive(),
                        )
                        .rect;
                    ui.painter()
                        .rect_filled(path_rect, SHARP, plt::BG(ui.visuals().dark_mode));
                    ui.painter().rect_stroke(
                        path_rect,
                        SHARP,
                        border(ui.style().visuals.dark_mode),
                        StrokeKind::Outside,
                    );
                    let font = FontId {
                        size: plt::font_size::META,
                        family: egui::FontFamily::Name("inter_medium".into()),
                    };
                    let (tw, th) = get_text_size(ui, "Preset Path:", font.clone()).into();
                    custom_text(
                        ui,
                        "Preset Path:",
                        font.clone(),
                        path_rect.left_center() + vec2(12.0, -th / 2.0),
                        plt::letter_spacing::BASE,
                        plt::TEXT,
                        Align::LEFT,
                    );

                    // Path name slot
                    let path_name_rect = ui
                        .allocate_rect(
                            egui::Rect::from_min_size(
                                path_rect.left_top()
                                    + vec2(36.0 + tw, (path_rect.height() - INNER_H) / 2.0),
                                vec2(path_rect.width() - (INNER_H + tw + 56.0), INNER_H),
                            ),
                            egui::Sense::focusable_noninteractive(),
                        )
                        .rect;
                    ui.painter().rect_filled(path_name_rect, SHARP, plt::BLACK);
                    ui.painter().rect_stroke(
                        path_name_rect,
                        SHARP,
                        border(ui.style().visuals.dark_mode),
                        StrokeKind::Inside,
                    );

                    let (cw, th) = get_text_size(ui, "K", font.clone()).into();
                    custom_text(
                        ui,
                        &if let Some(path) = &st.preset_load_path {
                            let dir = path.to_string_lossy().to_string();
                            let lw = path_name_rect.width();
                            let limit = (lw / cw) as usize;
                            if dir.len() > limit {
                                let mut out = "...".to_string();
                                out.push_str(
                                    &path.to_string_lossy().to_string()[dir.len() - limit..],
                                );
                                out
                            } else {
                                path.to_string_lossy().to_string()
                            }
                        } else {
                            "".to_string()
                        },
                        font,
                        path_name_rect.center() - vec2(0.0, th / 2.0),
                        plt::letter_spacing::MINIMAL,
                        plt::TEXT,
                        Align::Center,
                    );

                    // Browse path
                    modal_button(
                        ui,
                        path_rect.left_top()
                            + vec2(
                                path_rect.width() - (INNER_H + 12.0),
                                (path_rect.height() - (INNER_H)) / 2.0,
                            ),
                        vec2(INNER_H + 4.0, INNER_H),
                        "\u{e2c7}",
                        plt::font_size::ICON,
                        plt::YELLO,
                        &mut st.bool.open_preset_load_file_picker,
                        true,
                    );
                });

                modal_button(
                    ui,
                    resp.rect.left_bottom() - vec2(0.0, BUTTON_H),
                    vec2(resp.rect.width() / 2.0, 30.0),
                    "CANCEL",
                    plt::font_size::BODY,
                    plt::DANGER,
                    &mut st.bool.show_preset_load_modal,
                    false,
                );
                modal_button(
                    ui,
                    resp.rect.right_bottom() - vec2(resp.rect.width() / 2.0, BUTTON_H),
                    vec2(resp.rect.width() / 2.0, 30.0),
                    "LOAD",
                    plt::font_size::BODY,
                    plt::LIVE,
                    &mut st.bool.load_preset,
                    false,
                );
                if ui.ctx().input(|i| i.key_pressed(Key::Escape)) {
                    st.bool.show_preset_load_modal = false;
                }
            }
        });
}

pub fn window_drag_tooltip(ui: &mut egui::Ui) {
    egui::Area::new("window drag tooltip".into())
        .order(egui::Order::Background)
        .show(ui.ctx(), |ui| {
            let mut resp = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.content_rect().left_top().add(vec2(48., 14.)),
                    vec2(360., 30.),
                ),
                egui::Sense::click(),
            );
            resp.interact_rect.set_height(0.0);
            resp.interact_rect.set_width(0.0);

            custom_text(
                ui,
                "PRESS AND HOLD ANY KEY TO MOVE THE WINDOW",
                FontId {
                    size: plt::font_size::META,
                    family: egui::FontFamily::Name("inter_regular".into()),
                },
                pos2(resp.rect.left(), resp.rect.left_center().y - 9.0),
                plt::letter_spacing::BASE,
                plt::YELLO,
                Align::LEFT,
            );
        });
}
