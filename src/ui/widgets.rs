use std::ops::RangeInclusive;

use egui::{
    self, Align, Color32, CornerRadius, DragValue, FontFamily, FontId, Layout, Pos2, Rect,
    Response, Sense, Stroke, StrokeKind, Vec2, pos2, vec2,
};

use crate::{state::Labeled, ui::palette as plt};

const SHARP: CornerRadius = CornerRadius::ZERO;

fn border() -> Stroke {
    Stroke::new(plt::FRAME_WIDTH, plt::BORDER)
}

fn text(
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

/// Root control panel headers. eg. "GENERATOR", "POST FX"
pub fn section_header(ui: &mut egui::Ui, index: usize, name: &str, open: &mut bool) {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(
        vec2(w, plt::height::ROWHEAD),
        Sense::click().union(Sense::hover()),
    );
    let (mut bg, fg) = if *open {
        (plt::TEXT, plt::INK)
    } else {
        (plt::SURFACE, plt::BRIGHT)
    };

    let num_w = 46.0;
    let p = ui.painter();

    // interactions
    bg = if !*open && (resp.hovered()) {
        plt::SURFACE_HOVER
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
        border(),
    );
    p.rect_stroke(rect, SHARP, border(), StrokeKind::Middle);
    let fonts = FontId {
        size: plt::font_size::BIG,
        family: FontFamily::Name("inter_bold".into()),
    };
    let lbl_pos = pos2(rect.left() + num_w + 12.0, rect.center().y - 8.0);
    let idx_pos = pos2(rect.left() + num_w / 3.25, rect.center().y - 8.0);
    text(
        ui,
        name,
        fonts.clone(),
        lbl_pos,
        plt::letter_spacing::SPACED,
        fg,
        Align::LEFT,
    );
    text(
        ui,
        &format!("{index:02}"),
        fonts,
        idx_pos,
        plt::letter_spacing::SPACED,
        fg,
        Align::LEFT,
    );
}

/// One level below section headers
pub fn section_header_submenu(ui: &mut egui::Ui, name: &str, open: &mut bool) {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(w, plt::height::DROPDOWN_ITEM), Sense::click());
    let (bg, fg) = if *open {
        (plt::TEXT, plt::INK)
    } else if resp.hovered() {
        (plt::SURFACE_HOVER, plt::BRIGHT)
    } else {
        (plt::SURFACE, plt::BRIGHT)
    };
    let p = ui.painter();
    p.rect_filled(rect, SHARP, bg);
    p.line_segment([rect.left_bottom(), rect.right_bottom()], border());
    p.line_segment([rect.left_top(), rect.right_top()], border());
    p.line_segment([rect.left_bottom(), rect.left_top()], border());
    p.line_segment([rect.right_bottom(), rect.right_top()], border());
    let fonts = FontId {
        size: plt::font_size::BODY,
        family: FontFamily::Name("inter_medium".into()),
    };
    text(
        ui,
        name,
        fonts.clone(),
        pos2(rect.left() + 14.0, rect.center().y - 7.0),
        plt::letter_spacing::BASE,
        fg,
        Align::LEFT,
    );
    if resp.clicked() {
        *open = !*open;
    }
}

/// Non-interactive Label
pub fn static_label(ui: &mut egui::Ui, name: &str) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(w, plt::height::MENU_ITEM), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, SHARP, plt::GRAY);
    p.line_segment([rect.left_bottom(), rect.right_bottom()], border());
    p.line_segment([rect.left_bottom(), rect.left_top()], border());
    p.line_segment([rect.right_bottom(), rect.right_top()], border());
    let fonts = FontId {
        size: plt::font_size::BODY,
        family: FontFamily::Name("inter_medium".into()),
    };
    text(
        ui,
        name,
        fonts.clone(),
        pos2(rect.left() + 14.0, rect.center().y - 8.0),
        plt::letter_spacing::BASE,
        plt::TEXT,
        Align::LEFT,
    );
}

fn selector_box(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, plt::height::INNER), Sense::click());
    let border_color = if resp.hovered() {
        plt::TEXT
    } else {
        plt::BORDER
    };
    let p = ui.painter();
    p.rect_filled(rect, SHARP, plt::VOID);
    p.rect_stroke(
        rect,
        SHARP,
        Stroke {
            width: 1.0,
            color: border_color,
        },
        StrokeKind::Inside,
    );
    let textfont = FontId {
        size: plt::font_size::TINY,
        family: FontFamily::Name("inter_regular".into()),
    };
    let iconfont = FontId {
        size: plt::font_size::ICON,
        family: FontFamily::Name("icons".into()),
    };
    text(
        ui,
        &label.to_uppercase(),
        textfont,
        pos2(rect.left() + 10.0, rect.center().y - 6.0),
        plt::letter_spacing::BASE,
        plt::BRIGHT,
        Align::LEFT,
    );
    text(
        ui,
        "\u{e5c5}",
        iconfont,
        pos2(rect.right() - 4.0, rect.center().y - 9.0),
        plt::letter_spacing::BASE,
        plt::DIM,
        Align::RIGHT,
    );
    resp
}

fn popup_item(ui: &mut egui::Ui, label: &str, width: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, plt::height::INNER), Sense::click());
    let bc = if resp.hovered() {
        plt::TEXT
    } else {
        plt::BORDER
    };
    let p = ui.painter();
    p.rect_filled(rect, SHARP, plt::VOID);
    p.rect_stroke(
        rect,
        SHARP,
        Stroke {
            width: 1.0,
            color: bc,
        },
        StrokeKind::Inside,
    );
    let fonts = FontId {
        size: plt::font_size::META,
        family: FontFamily::Name("inter_regular".into()),
    };
    text(
        ui,
        &label.to_uppercase(),
        fonts.clone(),
        pos2(rect.left() + 10.0, rect.center().y - 7.0),
        plt::letter_spacing::BASE,
        plt::BRIGHT,
        Align::LEFT,
    );
    resp
}

/// Menu item with `label        [ item ]` structure
pub fn dropdown_row<T: Labeled>(
    ui: &mut egui::Ui,
    id: &str,
    label: &str,
    value: &mut T,
    options: &[T],
    open: &mut bool,
) {
    const SEL_W: f32 = 150.0;
    let mut sel_rect = egui::Rect::NOTHING;
    const PAD: f32 = 12.0;

    let bg = ui.painter().add(egui::Shape::Noop);
    let inner_rect = ui
        .allocate_ui_with_layout(
            vec2(ui.available_width(), plt::height::MENU_ITEM),
            egui::Layout::left_to_right(Align::Center),
            |ui| {
                ui.add_space(PAD);
                label_text(ui, label, ui.available_width() - (SEL_W + PAD * 1.0));
                let resp = selector_box(ui, value.text(), SEL_W);
                sel_rect = resp.rect;
                if resp.clicked() {
                    *open = !*open;
                }
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
        plt::BG,
    );
    ui.painter().set(bg, bg_rect.clone());
    let br = bg_rect.visual_bounding_rect();
    ui.painter()
        .line_segment([br.left_bottom(), br.right_bottom()], border());
    ui.painter()
        .line_segment([br.left_bottom(), br.left_top()], border());
    ui.painter()
        .line_segment([br.right_bottom(), br.right_top()], border());

    if *open {
        egui::Area::new(egui::Id::new(id).with("popup"))
            .fixed_pos(sel_rect.left_bottom())
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::new().show(ui, |ui| {
                    for &opt in options {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        if popup_item(ui, opt.text(), SEL_W).clicked() {
                            *value = opt;
                            *open = false;
                        }
                    }
                });
            });
    }
}

fn slider(ui: &mut egui::Ui, value: &mut f32, min: f32, max: f32, width: f32) {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 10.0), Sense::click_and_drag());
    if let Some(p) = resp.interact_pointer_pos() {
        let t = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        *value = min + t * (max - min);
    }
    let t = ((*value - min) / (max - min)).clamp(0.0, 1.0);
    let p = ui.painter();
    p.rect_filled(rect, SHARP, plt::VOID);
    p.rect_stroke(rect, SHARP, border(), StrokeKind::Inside);
    let tw = 10.0;
    let x = rect.left() + t * (rect.width() - tw);
    let thumb = Rect::from_min_size(pos2(x, rect.top() - 6.0), vec2(tw, rect.height() + 12.0));
    p.rect_filled(thumb, SHARP, plt::TEXT);
    p.rect_stroke(thumb, SHARP, border(), StrokeKind::Inside);
}

fn value_box(
    ui: &mut egui::Ui,
    value: &mut f32,
    decimals: usize,
    range: RangeInclusive<f32>,
    width: f32,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, plt::height::INNER), Sense::hover());
    ui.painter().rect_filled(rect, SHARP, plt::VOID);
    ui.painter()
        .rect_stroke(rect, SHARP, border(), StrokeKind::Inside);

    let sty = ui.style_mut();
    let fonts = FontId {
        size: plt::font_size::BODY,
        family: FontFamily::Name("inter_bold".into()),
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
        size: plt::font_size::META,
        family: FontFamily::Name("inter_medium".into()),
    };
    text(
        ui,
        s,
        fonts.clone(),
        pos2(rect.left() + 2.0, rect.center().y - 7.0),
        plt::letter_spacing::MINIMAL,
        plt::TEXT,
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
            vec2(ui.available_width(), plt::height::MENU_ITEM),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_min_height(plt::height::DROPDOWN_ITEM);
                ui.add_space(10.0);
                label_text(ui, label, 90.0);
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
        plt::BG,
    );
    ui.painter().set(bg, bg_rect.clone());
    ui.painter().line_segment(
        [
            bg_rect.visual_bounding_rect().left_bottom(),
            bg_rect.visual_bounding_rect().right_bottom(),
        ],
        border(),
    );
    ui.painter().line_segment(
        [
            bg_rect.visual_bounding_rect().left_bottom(),
            bg_rect.visual_bounding_rect().left_top(),
        ],
        border(),
    );
    ui.painter().line_segment(
        [
            bg_rect.visual_bounding_rect().right_bottom(),
            bg_rect.visual_bounding_rect().right_top(),
        ],
        border(),
    );
}

/// import/export
pub fn project_handler_button(ui: &mut egui::Ui, icon: &str, label: &str, width: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 42.7), Sense::click());
    let bg = if resp.hovered() { plt::TEXT } else { plt::BG };
    let fg = if resp.hovered() { plt::INK } else { plt::DIM };
    let p = ui.painter();
    p.rect_filled(rect, SHARP, bg);
    p.line_segment([rect.left_top(), rect.left_bottom()], border());
    p.line_segment([rect.left_bottom(), rect.right_bottom()], border());
    let fonts = (
        FontId {
            size: plt::font_size::ICON,
            family: FontFamily::Name("icons".into()),
        },
        FontId {
            size: plt::font_size::MED,
            family: FontFamily::Name("inter_medium".into()),
        },
    );
    text(
        ui,
        icon,
        fonts.0.clone(),
        pos2(rect.left_center().x + 52.0, rect.left_center().y - 9.5),
        plt::letter_spacing::BASE,
        fg,
        Align::LEFT,
    );
    text(
        ui,
        label,
        fonts.1.clone(),
        pos2(
            rect.right_center().x - rect.width() / 1.65,
            rect.left_center().y - 8.0,
        ),
        plt::letter_spacing::BASE,
        fg,
        Align::LEFT,
    );
    resp
}
