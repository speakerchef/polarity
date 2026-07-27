use crate::generators::fluidwave::ModSrc;
use crate::ui::app_widgets::modal_button;
use crate::ui::{ROUND_MAX, SHARP, apply_side_border};
use crate::ui::{custom_text, get_text_size};
use std::ops::RangeInclusive;
use std::path::PathBuf;

use eframe::egui::{
    self, Align, Color32, CornerRadius, DragValue, FontFamily, FontId, Layout, Pos2, Rect,
    Response, Sense, Shadow, Stroke, StrokeKind, Vec2, pos2, vec2,
};

use crate::traits::Labeled;
use crate::ui::palette as plt;
pub const TOGGLE_BUTTON_W: f32 = 34.0;

fn border(dark: bool) -> Stroke {
    Stroke::new(plt::FRAME_WIDTH, plt::BORDER(dark))
}

/// Root control panel headers. eg. "GENERATOR", "POST FX"
pub fn section_header(ui: &mut egui::Ui, index: usize, name: &str, open: &mut bool) -> Response {
    let w = ui.available_width();
    let (rect, mut resp) = ui.allocate_exact_size(
        vec2(w, plt::height::ROWHEAD),
        Sense::click().union(Sense::hover()),
    );
    resp = resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
    let (mut bg, fg) = if *open {
        (plt::TEXT, plt::INK)
    } else {
        (plt::SURFACE(ui.style().visuals.dark_mode), plt::BRIGHT)
    };

    let num_w = 42.0;
    let p = ui.painter();

    // interactions
    bg = if !*open && (resp.hovered()) {
        plt::SURFACE_HOVER(ui.style().visuals.dark_mode)
    } else {
        bg
    };
    if resp.clicked() {
        *open = !*open;
    }

    p.rect_filled(rect, SHARP, bg);
    p.line_segment(
        [
            pos2(rect.left() + num_w, rect.top()),
            pos2(rect.left() + num_w, rect.bottom()),
        ],
        border(ui.style().visuals.dark_mode),
    );
    p.rect_stroke(
        rect,
        SHARP,
        border(ui.style().visuals.dark_mode),
        StrokeKind::Middle,
    );
    let fonts = FontId {
        size: plt::font_size::BIG,
        family: FontFamily::Name("inter_bold".into()),
    };
    let lbl_pos = pos2(rect.left() + num_w + 12.0, rect.center().y - 8.0);
    custom_text(
        ui,
        name,
        fonts.clone(),
        lbl_pos,
        plt::letter_spacing::SPACED,
        fg,
        Align::LEFT,
    );
    let (_, th) = get_text_size(ui, &format!("{index:02}"), fonts.clone()).into();
    custom_text(
        ui,
        &format!("{index:02}"),
        fonts,
        rect.left_center() + vec2(num_w / 2.0, -th / 2.0),
        plt::letter_spacing::SPACED,
        fg,
        Align::Center,
    );

    resp
}

/// One level below section headers
pub fn section_header_submenu(ui: &mut egui::Ui, name: &str, open: &mut bool) -> Response {
    let w = ui.available_width();
    let (rect, mut resp) =
        ui.allocate_exact_size(vec2(w, plt::height::DROPDOWN_ITEM), Sense::click());
    resp = resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
    let (bg, fg) = if *open {
        (plt::TEXT, plt::INK)
    } else if resp.hovered() {
        (
            plt::SURFACE_HOVER(ui.style().visuals.dark_mode),
            plt::BRIGHT,
        )
    } else {
        (plt::SURFACE(ui.style().visuals.dark_mode), plt::BRIGHT)
    };
    let p = ui.painter();
    p.rect_filled(rect, SHARP, bg);
    p.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        border(ui.style().visuals.dark_mode),
    );
    p.line_segment(
        [rect.left_top(), rect.right_top()],
        border(ui.style().visuals.dark_mode),
    );
    let fonts = FontId {
        size: plt::font_size::BODY,
        family: FontFamily::Name("inter_medium".into()),
    };
    let (_, th) = get_text_size(ui, name, fonts.clone()).into();
    custom_text(
        ui,
        name,
        fonts.clone(),
        rect.left_center() + vec2(12.0, -th / 2.0),
        plt::letter_spacing::BASE,
        fg,
        Align::LEFT,
    );
    if resp.clicked() {
        *open = !*open;
    }
    resp
}

/// Non-interactive Label
pub fn static_label(ui: &mut egui::Ui, name: &str) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, 30.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, SHARP, plt::GRAY);
    p.rect_stroke(
        rect,
        SHARP,
        border(ui.visuals().dark_mode),
        StrokeKind::Middle,
    );
    let fonts = FontId {
        size: plt::font_size::BODY,
        family: FontFamily::Name("inter_medium".into()),
    };
    let (_, th) = get_text_size(ui, name, fonts.clone()).into();
    custom_text(
        ui,
        name,
        fonts.clone(),
        rect.left_center() + vec2(12.0, -th / 2.0),
        plt::letter_spacing::BASE,
        plt::TEXT,
        Align::LEFT,
    );
}
impl Labeled for &'static str {
    fn text(&self) -> &'static str {
        self
    }
}
pub fn mod_button(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut ModSrc,
    h: f32,
    mod_menu_open: &mut bool,
    mod_src_menu_open: &mut bool,
    range: &mut f32,
) {
    const W: f32 = 12.0;
    let (inner, mut resp) = ui.allocate_exact_size(vec2(W, h), egui::Sense::click());
    resp = resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
    if resp.clicked() {
        *mod_menu_open = !*mod_menu_open;
    }
    let (bg, fg) = if *mod_menu_open {
        (
            if !matches!(value, ModSrc::None) {
                plt::YELLO
            } else {
                plt::TEXT
            },
            plt::INK,
        )
    } else {
        if resp.hovered() {
            (plt::YELLO, plt::INK)
        } else {
            (
                if !matches!(value, ModSrc::None) {
                    plt::YELLO
                } else {
                    plt::SURFACE_HOVER(ui.visuals().dark_mode)
                },
                if !matches!(value, ModSrc::None) {
                    plt::INK
                } else {
                    plt::TEXT
                },
            )
        }
    };
    ui.painter().rect_filled(inner, SHARP, bg);

    let font = FontId {
        size: plt::font_size::TINY - 1.0,
        family: egui::FontFamily::Name("inter_bold".into()),
    };
    let (_, th) = get_text_size(ui, "M", font.clone()).into();
    custom_text(
        ui,
        match value {
            ModSrc::None => "•",
            ModSrc::EnvA => "A",
            ModSrc::EnvB => "B",
            ModSrc::EnvC => "C",
            ModSrc::EnvD => "D",
        },
        font.clone(),
        inner.center() - vec2(0.0, th / 2.0),
        0.0,
        fg,
        Align::Center,
    );
    let mut dropdown_rect = inner;
    if *mod_menu_open {
        let resp = egui::Area::new(label.to_string().into())
            .movable(false)
            .fixed_pos(inner.left_bottom())
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                let ctr_pan_w = 320.0;
                let ctr_pan_pad = 24.0;
                ui.set_width(ctr_pan_w - ctr_pan_pad);
                ui.painter()
                    .rect_filled(ui.available_rect_before_wrap(), SHARP, plt::YELLO);
                ui.painter().rect_stroke(
                    ui.available_rect_before_wrap(),
                    SHARP,
                    Stroke {
                        width: 1.0,
                        color: plt::TEXT,
                    },
                    StrokeKind::Outside,
                );
                let shadow = Shadow {
                    offset: [0, 0],
                    blur: 10,
                    spread: 8,
                    color: Color32::from_black_alpha(50),
                };
                ui.painter()
                    .add(shadow.as_shape(ui.available_rect_before_wrap(), SHARP));
                static_label(ui, "MODULATOR OPTIONS");
                dropdown_rect = dropdown_row(
                    ui,
                    "MOD SOURCE",
                    value,
                    ModSrc::ALL,
                    mod_src_menu_open,
                    false,
                )
                .1
                .rect;
                slider_row(ui, "RANGE", range, -100.0, 100.0, 0, false);
            })
            .response;
        // close dropdown if clicked elsewhere
        ui.ctx().input(|i| {
            if i.pointer.primary_clicked()
                && (!resp
                    .interact_rect
                    .contains(i.pointer.interact_pos().unwrap_or_default())
                    && !inner.contains(i.pointer.interact_pos().unwrap_or_default())
                    && !dropdown_rect.contains(i.pointer.interact_pos().unwrap_or_default()))
            {
                *mod_menu_open = false;
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub fn menu_bar_option(
    ui: &mut egui::Ui,
    label: &str,
    width: Option<f32>,
    font: egui::FontId,
    open: &mut bool,
    opts: &[&str],
    states: &mut [&mut bool],
    h: f32,
    reverse_dropdown_pos: bool,
) -> egui::Response {
    let (tw, th) = get_text_size(ui, &label.to_uppercase(), font.clone()).into();
    let width = width.unwrap_or(tw + 24.0);
    let (rect, mut resp) = ui.allocate_exact_size(vec2(width, h - 4.0), Sense::click());
    resp = resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
    if resp.clicked() {
        *open = !*open;
    }
    let (bg, fg) = if resp.hovered() {
        (plt::YELLO, plt::INK)
    } else {
        if *open {
            (plt::YELLO, plt::INK)
        } else {
            (plt::SURFACE(ui.style().visuals.dark_mode), plt::TEXT)
        }
    };

    let p = ui.painter();
    p.rect_filled(rect, SHARP, bg);
    p.rect_stroke(
        rect,
        SHARP,
        border(ui.style().visuals.dark_mode),
        StrokeKind::Inside,
    );
    custom_text(
        ui,
        &label.to_uppercase(),
        font.clone(),
        rect.center() - vec2(0.0, th / 2.0),
        plt::letter_spacing::BASE,
        fg,
        Align::Center,
    );
    let mut popup_rect = egui::Rect::NOTHING;

    let (drop_tw, _) = get_text_size(ui, label, font.clone()).into();
    let padding = width - drop_tw;
    let max_label_w = opts.iter().fold(0.0, |acc, opt| {
        let (tw, _) = get_text_size(ui, opt, font.clone()).into();
        if tw >= acc { tw } else { acc }
    });

    if *open {
        popup_rect = egui::Area::new(egui::Id::new(label).with("popup"))
            .movable(false)
            .fixed_pos(if reverse_dropdown_pos {
                resp.rect.right_bottom() - vec2(max_label_w - 1.0, 0.0)
            } else {
                resp.rect.left_bottom() + vec2(1.0, 0.0)
            })
            .order(egui::Order::Tooltip)
            .show(ui.ctx(), |ui| {
                egui::Frame::new().show(ui, |ui| {
                    for (i, label) in opts.iter().enumerate() {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        if menu_bar_popup(ui, label, max_label_w.max(width), padding).clicked() {
                            *open = false;
                            if let Some(state) = states.get_mut(i) {
                                **state = !**state;
                            }
                        }
                    }
                });
            })
            .response
            .rect;
    }

    // close dropdown if clicked elsewhere
    ui.ctx().input(|i| {
        if i.pointer.primary_clicked()
            && !resp
                .interact_rect
                .contains(i.pointer.interact_pos().unwrap_or_default())
            && !popup_rect.contains(i.pointer.interact_pos().unwrap_or_default())
        {
            *open = false;
        }
    });

    resp
}

pub fn popup_item(
    ui: &mut egui::Ui,
    label: &str,
    font_size: f32,
    w: f32,
    h: Option<f32>,
    bottom_border: bool,
) -> Response {
    let (rect, mut resp) =
        ui.allocate_exact_size(vec2(w, h.unwrap_or(plt::height::INNER)), Sense::click());
    resp = resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
    let (bg, fg) = if resp.hovered() {
        (plt::YELLO, plt::INK)
    } else {
        (plt::VOID(ui.style().visuals.dark_mode), plt::BRIGHT)
    };
    let p = ui.painter();
    p.rect_filled(rect, SHARP, bg);
    // p.rect_stroke(
    //     rect,
    //     SHARP,
    //     border(ui.visuals().dark_mode),
    //     StrokeKind::Inside,
    // );
    p.line_segment(
        [rect.left_top(), rect.right_top()],
        border(ui.visuals().dark_mode),
    );
    apply_side_border(ui, rect, bottom_border);
    let fonts = FontId {
        size: font_size,
        family: FontFamily::Name("inter_medium".into()),
    };
    let (_, th) = get_text_size(ui, &label.to_uppercase(), fonts.clone()).into();
    custom_text(
        ui,
        &label.to_uppercase(),
        fonts.clone(),
        rect.left_center() - vec2(-(h.unwrap_or(plt::height::INNER) - th) / 1.25, th / 2.0),
        plt::letter_spacing::MINIMAL,
        fg,
        Align::LEFT,
    );
    resp
}

pub fn menu_bar_popup(ui: &mut egui::Ui, label: &str, mut width: f32, padding: f32) -> Response {
    width -= 2.0;
    let fonts = FontId {
        size: plt::font_size::TINY,
        family: FontFamily::Name("inter_medium".into()),
    };
    let (tw, th) = get_text_size(ui, label, fonts.clone()).into();

    let rect_sz = vec2(
        if tw + padding > width {
            width.max(tw) + padding
        } else {
            width
        },
        22.0,
    );
    let (rect, mut resp) = ui.allocate_exact_size(rect_sz, Sense::click());
    resp = resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);

    let (bg, fg) = if resp.hovered() {
        (plt::YELLO, plt::BG(ui.style().visuals.dark_mode))
    } else {
        (plt::BG(ui.style().visuals.dark_mode), plt::TEXT)
    };

    ui.painter().rect_filled(rect, SHARP, bg);
    ui.painter().rect_stroke(
        rect,
        SHARP,
        border(ui.visuals().dark_mode),
        StrokeKind::Outside,
    );

    custom_text(
        ui,
        label,
        fonts.clone(),
        rect.left_center() + vec2(8.0, -th / 2.0),
        plt::letter_spacing::MINIMAL,
        fg,
        Align::LEFT,
    );
    resp
}

/// Menu item with `label        [ item ]` structure
pub fn dropdown_row<T: Labeled + Default>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut T,
    options: &[T],
    open: &mut bool,
    bottom_border: bool,
) -> (
    Rect,     /* dropdown row rect*/
    Response, /* dropdown options */
) {
    const SEL_W: f32 = 150.0;
    const PAD: f32 = 12.0;

    let bg = ui.painter().add(egui::Shape::Noop);
    let mut return_resp = None;
    let inner_resp = ui
        .allocate_ui_with_layout(
            vec2(ui.available_width(), plt::height::DROPDOWN_ITEM),
            egui::Layout::left_to_right(Align::Center),
            |ui| {
                ui.add_space(PAD);
                label_text(ui, label, ui.available_width() - (SEL_W + PAD * 1.0));
                return_resp = Some(dropdown_menu(
                    ui,
                    (SEL_W, plt::height::INNER),
                    value,
                    options,
                    open,
                ));
            },
        )
        .response;
    let bg_rect = egui::Shape::rect_filled(
        egui::Rect::from_min_size(
            inner_resp.rect.left_top(),
            vec2(ui.available_width(), inner_resp.rect.height()),
        ),
        SHARP,
        plt::INK,
    );
    ui.painter().set(bg, bg_rect.clone());
    let br = bg_rect.visual_bounding_rect();
    apply_side_border(ui, br, bottom_border);
    // close dropdown if clicked elsewhere
    ui.ctx().input(|i| {
        if i.pointer.primary_clicked()
            && !br.contains(i.pointer.interact_pos().unwrap_or_default())
            && !inner_resp
                .rect
                .contains(i.pointer.interact_pos().unwrap_or_default())
        {
            *open = false;
        }
    });
    (br, return_resp.unwrap_or(inner_resp))
}
pub fn path_picker(
    ui: &mut egui::Ui,
    path: Option<&PathBuf>,
    top_left: Pos2,
    w: f32,
    label: &str,
    open: &mut bool,
    bottom_border: bool,
) -> Response {
    const PAD: f32 = 12.0;

    let bg = ui.painter().add(egui::Shape::Noop);
    let mut inner_rect = egui::Rect::NOTHING;

    let outer_resp = ui.allocate_rect(
        egui::Rect::from_min_size(top_left, vec2(w, plt::height::INNER)),
        egui::Sense::focusable_noninteractive(),
    );
    let outer_rect = outer_resp.rect;
    ui.scope_builder(egui::UiBuilder::new().max_rect(outer_rect), |ui| {
        inner_rect = ui
            .allocate_ui_with_layout(
                vec2(w, plt::height::DROPDOWN_ITEM),
                egui::Layout::left_to_right(Align::Center),
                |ui| {
                    let font = FontId {
                        size: plt::font_size::TINY,
                        family: FontFamily::Name("inter_medium".into()),
                    };
                    let (tw, _) = get_text_size(ui, label, font.clone()).into();
                    let path_rect = outer_rect;
                    ui.add_space(PAD);
                    label_text(ui, label, tw + PAD * 2.0);

                    let path_name_rect_width = w - (plt::height::INNER + tw + 50.0);
                    let vertical_offset = (plt::height::DROPDOWN_ITEM - plt::height::INNER) / 2.0;
                    let path_name_rect = ui
                        .allocate_rect(
                            egui::Rect::from_min_size(
                                path_rect.left_top()
                                    + vec2(
                                        PAD * 2.5 + tw,
                                        (path_rect.height() - plt::height::INNER) / 2.0
                                            + vertical_offset,
                                    ),
                                vec2(path_name_rect_width, plt::height::INNER),
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
                        &if let Some(path) = &path {
                            // length limiting
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
                                path_rect.width() - (plt::height::INNER + 12.0),
                                (path_rect.height() - (plt::height::INNER - 1.0)) / 2.0
                                    + vertical_offset,
                            ),
                        vec2(plt::height::INNER + 4.0, plt::height::INNER - 1.0),
                        "\u{e2c7}",
                        plt::font_size::ICON,
                        plt::YELLO,
                        open,
                        true,
                    );
                },
            )
            .response
            .rect;
    });

    let bg_rect = egui::Shape::rect_filled(
        egui::Rect::from_min_size(inner_rect.left_top(), vec2(w, inner_rect.height())),
        SHARP,
        plt::INK,
    );
    ui.painter().set(bg, bg_rect.clone());
    let br = bg_rect.visual_bounding_rect();
    apply_side_border(ui, br, bottom_border);
    // close dropdown if clicked elsewhere
    ui.ctx().input(|i| {
        if i.pointer.primary_clicked()
            && !br.contains(i.pointer.interact_pos().unwrap_or_default())
            && !inner_rect.contains(i.pointer.interact_pos().unwrap_or_default())
        {
            *open = false;
        }
    });
    outer_resp
}

/// Menu item with `On/Off` toggle button
pub fn toggle_button_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut bool,
    bottom_border: bool,
) -> Response {
    const PAD: f32 = 12.0;

    let bg = ui.painter().add(egui::Shape::Noop);
    let inner_resp = ui
        .allocate_ui_with_layout(
            vec2(ui.available_width(), plt::height::DROPDOWN_ITEM),
            egui::Layout::left_to_right(Align::Center),
            |ui| {
                ui.add_space(PAD);
                label_text(ui, label, ui.available_width() - (TOGGLE_BUTTON_W + PAD));
                ui.add_space(ui.available_width() - (TOGGLE_BUTTON_W + PAD + 1.0));
                toggle_button(ui, value);
            },
        )
        .response;
    let inner_rect = inner_resp.rect;
    let bg_rect = egui::Shape::rect_filled(
        egui::Rect::from_min_size(
            inner_rect.left_top(),
            vec2(ui.available_width(), inner_rect.height()),
        ),
        SHARP,
        plt::INK,
    );
    ui.painter().set(bg, bg_rect.clone());
    apply_side_border(ui, bg_rect.visual_bounding_rect(), bottom_border);
    inner_resp
}

pub fn subheader_toggle_button(ui: &mut egui::Ui, root_rect: &egui::Rect, on: &mut bool) {
    let rect = ui
        .allocate_rect(
            egui::Rect::from_min_size(
                root_rect.right_top() - vec2(TOGGLE_BUTTON_W - 5., -11.0),
                vec2(0.0, root_rect.height()),
            ),
            egui::Sense::focusable_noninteractive(),
        )
        .rect;
    ui.painter().rect_filled(rect, SHARP, plt::LIVE);
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.set_min_height(root_rect.height() - 11.0);
        toggle_button(ui, on);
    });
}

pub fn toggle_button(ui: &mut egui::Ui, on: &mut bool) {
    let (rect, mut resp) =
        ui.allocate_exact_size(vec2(TOGGLE_BUTTON_W, 16.0), egui::Sense::click());
    resp = resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
    let mut thumb = ui.allocate_rect(
        egui::Rect::from_min_size(rect.left_top(), vec2(16.0, 16.0)),
        egui::Sense::click(),
    );
    thumb = thumb.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
    let mut thumb_rect = thumb.rect;
    if resp.clicked() || thumb.clicked() {
        *on = !*on;
    }
    if *on {
        thumb_rect = thumb_rect.translate(vec2(TOGGLE_BUTTON_W - thumb_rect.width(), 0.0));
    }
    let bg = if *on {
        plt::YELLO
    } else {
        plt::SURFACE_HOVER(ui.visuals().dark_mode)
    };
    ui.painter().rect_filled(rect, ROUND_MAX, bg);
    ui.painter().rect_stroke(
        rect,
        ROUND_MAX,
        border(ui.visuals().dark_mode),
        StrokeKind::Outside,
    );
    ui.painter().rect_filled(thumb_rect, ROUND_MAX, plt::BRIGHT);
    ui.painter().rect_stroke(
        thumb_rect,
        ROUND_MAX,
        Stroke {
            width: 0.5,
            color: plt::DIM,
        },
        StrokeKind::Outside,
    );
}

pub fn dropdown_menu<T: Labeled + Default>(
    ui: &mut egui::Ui,
    dim: (f32, f32),
    value: &mut T,
    options: &[T],
    open: &mut bool,
) -> Response {
    let (inner, mut resp) = ui.allocate_exact_size(vec2(dim.0, dim.1), egui::Sense::click());
    resp = resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
    if resp.clicked() {
        *open = !*open;
    }
    let bc = if resp.hovered() {
        plt::YELLO
    } else {
        plt::BORDER(ui.visuals().dark_mode)
    };
    let border = Stroke {
        width: 1.0,
        color: bc,
    };
    ui.painter()
        .rect_filled(inner, SHARP, plt::VOID(ui.visuals().dark_mode));
    ui.painter()
        .rect_stroke(inner, SHARP, border, egui::StrokeKind::Middle);

    let font = FontId {
        size: plt::font_size::TINY,
        family: egui::FontFamily::Name("inter_medium".into()),
    };
    let (_, th) = get_text_size(ui, &value.text().to_uppercase(), font.clone()).into();
    let offset = (dim.1 - th) / 1.25;
    custom_text(
        ui,
        &value.text().to_uppercase(),
        font.clone(),
        inner.left_center() - vec2(-offset, th / 2.0),
        plt::letter_spacing::MINIMAL,
        plt::TEXT,
        Align::LEFT,
    );
    let icon_font = FontId {
        size: plt::font_size::ICON,
        family: egui::FontFamily::Name("icons".into()),
    };

    let (_, th) = get_text_size(ui, "\u{e5c5}", icon_font.clone()).into();
    const CHEVRON_UP: &str = "\u{e5c7}";
    const CHEVRON_DOWN: &str = "\u{e5c5}";
    let offset = (dim.1 - th) / 1.25;
    custom_text(
        ui,
        if *open { CHEVRON_UP } else { CHEVRON_DOWN },
        icon_font,
        inner.right_center() - vec2(offset, th / 2.0),
        plt::letter_spacing::MINIMAL,
        plt::YELLO,
        Align::RIGHT,
    );

    let popup_item_w = options
        .iter()
        .fold(0.0, |acc, opt| {
            let (tw, _) = get_text_size(ui, opt.text(), font.clone()).into();
            if tw * 1.5 > acc { tw * 1.5 } else { acc }
        })
        .max(dim.0);
    if *open {
        resp = egui::Area::new(egui::Id::new(value.text()).with("popup"))
            .fixed_pos(inner.left_bottom() - vec2(popup_item_w - dim.0, -1.0))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::new().show(ui, |ui| {
                    for opt in options {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        if popup_item(
                            ui,
                            opt.text(),
                            plt::font_size::TINY,
                            popup_item_w,
                            Some(dim.1),
                            opt == options.first().unwrap_or(&T::default())
                                || opt == options.last().unwrap_or(&T::default()),
                        )
                        .clicked()
                        {
                            *value = opt.clone();
                            *open = false;
                        }
                    }
                });
            })
            .response;
    }
    // close dropdown if clicked elsewhere
    ui.ctx().input(|i| {
        if i.pointer.primary_clicked()
            && !inner.contains(i.pointer.interact_pos().unwrap_or_default())
        {
            *open = false;
        }
    });
    resp
}

fn slider(ui: &mut egui::Ui, value: &mut f32, min: f32, max: f32, width: f32) {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 5.0), Sense::click_and_drag());
    if let Some(p) = resp.interact_pointer_pos() {
        let t = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        *value = min + t * (max - min);
    }
    let t = ((*value - min) / (max - min)).clamp(0.0, 1.0);
    ui.painter().rect_filled(
        rect,
        CornerRadius::from(2),
        plt::VOID(ui.style().visuals.dark_mode),
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::from(2),
        border(ui.style().visuals.dark_mode),
        StrokeKind::Inside,
    );
    let tw = 10.0;
    let x = rect.left() + t * (rect.width() - tw);
    let thumb = Rect::from_min_size(pos2(x, rect.top() - 6.0), vec2(tw, rect.height() + 12.0));
    ui.painter()
        .rect_filled(thumb, CornerRadius::from(2), plt::YELLO);
    ui.painter().rect_stroke(
        thumb,
        CornerRadius::from(2),
        border(ui.style().visuals.dark_mode),
        StrokeKind::Inside,
    );
}

fn value_box(
    ui: &mut egui::Ui,
    value: &mut f32,
    decimals: usize,
    range: RangeInclusive<f32>,
    width: f32,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, plt::height::INNER), Sense::hover());
    ui.painter()
        .rect_filled(rect, SHARP, plt::VOID(ui.style().visuals.dark_mode));
    ui.painter().rect_stroke(
        rect,
        SHARP,
        border(ui.style().visuals.dark_mode),
        StrokeKind::Inside,
    );

    let sty = ui.style_mut();
    let fonts = FontId {
        size: plt::font_size::META,
        family: FontFamily::Name("mono_medium".into()),
    };
    sty.override_font_id = Some(fonts);

    ui.put(
        rect,
        DragValue::new(value)
            .range(range)
            .update_while_editing(false)
            .min_decimals(decimals)
            .max_decimals(decimals),
    );
}

fn label_text(ui: &mut egui::Ui, s: &str, width: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, plt::height::INNER), Sense::hover());
    let fonts = FontId {
        size: plt::font_size::TINY,
        family: FontFamily::Name("inter_medium".into()),
    };
    let (_, th) = get_text_size(ui, s, fonts.clone()).into();
    custom_text(
        ui,
        s,
        fonts.clone(),
        rect.left_center() + vec2(0.0, -th / 2.0),
        plt::letter_spacing::MINIMAL,
        plt::BRIGHT,
        Align::LEFT,
    );
}

/// `val =====|===== [x.xx]`
pub fn slider_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    decimals: usize,
    bottom_border: bool,
) {
    let bg = ui.painter().add(egui::Shape::Noop);
    let inner_rect = ui
        .allocate_ui_with_layout(
            vec2(ui.available_width(), plt::height::DROPDOWN_ITEM),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_min_height(plt::height::DROPDOWN_ITEM);
                ui.add_space(12.0);
                label_text(ui, label, 70.0);
                slider(ui, value, min, max, plt::width::SLIDER);
                ui.add_space(8.0);
                value_box(ui, value, decimals, min..=max, 54.0);
            },
        )
        .response
        .rect;

    let bg_rect = egui::Shape::rect_filled(
        egui::Rect::from_min_size(
            inner_rect.left_top(),
            vec2(ui.available_width(), inner_rect.height()),
        ),
        SHARP,
        plt::INK,
    );
    ui.painter().set(bg, bg_rect.clone());
    apply_side_border(ui, bg_rect.visual_bounding_rect(), bottom_border);
}
#[allow(clippy::too_many_arguments)]
pub fn mod_slider_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    decimals: usize,
    mod_src: &mut ModSrc,
    mod_open: &mut bool,
    mod_src_open: &mut bool,
    mod_range: &mut f32,
    bottom_border: bool,
) {
    let bg = ui.painter().add(egui::Shape::Noop);
    let inner_rect = ui
        .allocate_ui_with_layout(
            vec2(ui.available_width(), plt::height::DROPDOWN_ITEM),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_min_height(plt::height::DROPDOWN_ITEM);
                mod_button(
                    ui,
                    label,
                    mod_src,
                    plt::height::DROPDOWN_ITEM,
                    mod_open,
                    mod_src_open,
                    mod_range,
                );
                ui.add_space(6.0);
                label_text(ui, label, 64.0);
                slider(ui, value, min, max, plt::width::SLIDER);
                ui.add_space(8.0);
                value_box(ui, value, decimals, min..=max, 54.0);
            },
        )
        .response
        .rect;

    let bg_rect = egui::Shape::rect_filled(
        egui::Rect::from_min_size(
            inner_rect.left_top(),
            vec2(ui.available_width(), inner_rect.height()),
        ),
        SHARP,
        plt::INK,
    );
    ui.painter().set(bg, bg_rect.clone());
    apply_side_border(ui, bg_rect.visual_bounding_rect(), bottom_border);
}
