use crate::ui::{custom_text, get_text_size};
use std::ops::RangeInclusive;

use eframe::egui::{
    self, Align, CornerRadius, DragValue, FontFamily, FontId, Layout, Rect, Response, Sense,
    Stroke, StrokeKind, Vec2, pos2, vec2,
};

use crate::{state::Labeled, ui::palette as plt};

const SHARP: CornerRadius = CornerRadius::ZERO;

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
    p.line_segment(
        [rect.left_bottom(), rect.left_top()],
        border(ui.style().visuals.dark_mode),
    );
    p.line_segment(
        [rect.right_bottom(), rect.right_top()],
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
    p.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        border(ui.style().visuals.dark_mode),
    );
    p.line_segment(
        [rect.left_bottom(), rect.left_top()],
        border(ui.style().visuals.dark_mode),
    );
    p.line_segment(
        [rect.right_bottom(), rect.right_top()],
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
        plt::TEXT,
        Align::LEFT,
    );
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
        // plt::BRIGHT,
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
    p.rect_stroke(
        rect,
        SHARP,
        border(ui.visuals().dark_mode),
        StrokeKind::Inside,
    );
    let fonts = FontId {
        size: font_size,
        family: FontFamily::Name("mono_medium".into()),
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
pub fn dropdown_row<T: Labeled>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut T,
    options: &[T],
    open: &mut bool,
) {
    const SEL_W: f32 = 150.0;
    const PAD: f32 = 12.0;

    let bg = ui.painter().add(egui::Shape::Noop);
    let inner_rect = ui
        .allocate_ui_with_layout(
            // vec2(ui.available_width(), plt::height::MENU_ITEM),
            vec2(ui.available_width(), plt::height::DROPDOWN_ITEM),
            egui::Layout::left_to_right(Align::Center),
            |ui| {
                ui.add_space(PAD);
                label_text(ui, label, ui.available_width() - (SEL_W + PAD * 1.0));
                dropdown_menu(ui, (SEL_W, plt::height::INNER), value, options, open);
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
    let br = bg_rect.visual_bounding_rect();
    ui.painter().line_segment(
        [br.left_bottom(), br.right_bottom()],
        border(ui.style().visuals.dark_mode),
    );
    ui.painter().line_segment(
        [br.left_bottom(), br.left_top()],
        border(ui.style().visuals.dark_mode),
    );
    ui.painter().line_segment(
        [br.right_bottom(), br.right_top()],
        border(ui.style().visuals.dark_mode),
    );

    // close dropdown if clicked elsewhere
    ui.ctx().input(|i| {
        if i.pointer.primary_clicked()
            && !br.contains(i.pointer.interact_pos().unwrap_or_default())
            && !inner_rect.contains(i.pointer.interact_pos().unwrap_or_default())
        {
            *open = false;
        }
    });
}

pub fn dropdown_menu<T: Labeled>(
    ui: &mut egui::Ui,
    dim: (f32, f32),
    value: &mut T,
    options: &[T],
    open: &mut bool,
) {
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
        .rect_stroke(inner, SHARP, border, egui::StrokeKind::Inside);

    let font = FontId {
        size: plt::font_size::TINY,
        family: egui::FontFamily::Name("mono_medium".into()),
    };
    let (_, th) = get_text_size(ui, &value.text().to_uppercase(), font.clone()).into();
    let offset = (dim.1 - th) / 1.25;
    custom_text(
        ui,
        &value.text().to_uppercase(),
        font.clone(),
        inner.left_center() - vec2(-offset, (th / 2.0) + 1.0),
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

    if *open {
        egui::Area::new(egui::Id::new(value.text()).with("popup"))
            .fixed_pos(inner.left_bottom())
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::new().show(ui, |ui| {
                    for &opt in options {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        if popup_item(ui, opt.text(), plt::font_size::TINY, dim.0, Some(dim.1))
                            .clicked()
                        {
                            *value = opt;
                            *open = false;
                        }
                    }
                });
            });
    }
    // close dropdown if clicked elsewhere
    ui.ctx().input(|i| {
        if i.pointer.primary_clicked()
            && !inner.contains(i.pointer.interact_pos().unwrap_or_default())
        {
            *open = false;
        }
    });
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
        // family: FontFamily::Name("mono_medium".into()),
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
) {
    let bg = ui.painter().add(egui::Shape::Noop);
    let inner_rect = ui
        .allocate_ui_with_layout(
            vec2(ui.available_width(), plt::height::DROPDOWN_ITEM),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_min_height(plt::height::DROPDOWN_ITEM);
                ui.add_space(12.0);
                label_text(ui, label, 65.0);
                slider(ui, value, min, max, plt::width::SLIDER);
                ui.add_space(13.0);
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
    ui.painter().line_segment(
        [
            bg_rect.visual_bounding_rect().left_bottom(),
            bg_rect.visual_bounding_rect().right_bottom(),
        ],
        border(ui.style().visuals.dark_mode),
    );
    ui.painter().line_segment(
        [
            bg_rect.visual_bounding_rect().left_bottom(),
            bg_rect.visual_bounding_rect().left_top(),
        ],
        border(ui.style().visuals.dark_mode),
    );
    ui.painter().line_segment(
        [
            bg_rect.visual_bounding_rect().right_bottom(),
            bg_rect.visual_bounding_rect().right_top(),
        ],
        border(ui.style().visuals.dark_mode),
    );
}
