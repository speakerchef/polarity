use std::time::Duration;

use egui::{
    self, Align, Color32, CornerRadius, FontFamily, FontId, Response, Sense, Stroke, StrokeKind,
    pos2, vec2,
};

use crate::{
    state::PlaybackMode,
    ui::{palette as plt, text},
};

pub const SHARP: CornerRadius = CornerRadius::ZERO;

pub fn border() -> Stroke {
    Stroke::new(plt::FRAME_WIDTH, plt::BORDER)
}

pub fn transport_button(ui: &mut egui::Ui, icon: &str, c: Color32, big: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(28.0, 24.0), Sense::click());
    let (bg, stk) = if resp.hovered() {
        (plt::SURFACE_HOVER, plt::BORDER)
    } else {
        (plt::BG, plt::BG)
    };

    ui.painter().rect_filled(rect, SHARP, bg);
    ui.painter().rect_stroke(
        rect,
        SHARP,
        Stroke {
            width: 1.,
            color: stk,
        },
        StrokeKind::Inside,
    );
    let font = FontId {
        size: plt::font_size::ICON + if big { 2.0 } else { 0.0 },
        family: FontFamily::Name("icons".into()),
    };
    let pos = if big {
        pos2(rect.left() + 5.0, rect.bottom() - 22.0)
    } else {
        pos2(rect.left() + 6.0, rect.bottom() - 21.0)
    };
    text(
        ui,
        icon,
        font,
        pos,
        plt::letter_spacing::BASE,
        c,
        Align::LEFT,
    );
    resp
}

pub fn timecode(ui: &mut egui::Ui, elapsed: &Duration, dur: &Duration) {
    let (rect, _) = ui.allocate_exact_size(vec2(96.0, 32.0), Sense::click());
    ui.painter().rect_filled(rect, SHARP, plt::VOID);
    ui.painter()
        .line_segment([rect.left_bottom(), rect.left_top()], border());
    ui.painter()
        .line_segment([rect.right_bottom(), rect.right_top()], border());
    let font = FontId {
        size: plt::font_size::BODY,
        family: FontFamily::Name("inter_regular".into()),
    };
    let esecs = elapsed.as_secs_f64();
    let (emins, esecs) = ((esecs / 60.0) as u64, (esecs % 60.0) as u64);
    let dsecs = dur.as_secs_f64();
    let (dmins, dsecs) = ((dsecs / 60.0) as u64, (dsecs % 60.0) as u64);
    let timeposl = pos2(rect.left() + 7.0, rect.bottom() - 23.);
    let timeposr = pos2(rect.right() - 7.0, rect.bottom() - 23.);
    text(
        ui,
        &format!("{emins:02}:{esecs:02}"),
        font.clone(),
        timeposl,
        plt::letter_spacing::MINIMAL,
        plt::BRIGHT,
        Align::LEFT,
    );
    text(
        ui,
        "/",
        FontId {
            size: plt::font_size::MED,
            family: FontFamily::Name("inter_regular".into()),
        },
        pos2(rect.center().x - 2.0, rect.bottom() - 24.),
        plt::letter_spacing::SPACED,
        plt::BRIGHT,
        Align::LEFT,
    );
    text(
        ui,
        &format!("{dmins:02}:{dsecs:02}"),
        font,
        timeposr,
        plt::letter_spacing::MINIMAL,
        plt::BRIGHT,
        Align::RIGHT,
    );
}
pub fn file_info(ui: &mut egui::Ui, info: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(info.len() as f32 * 6., 32.0), Sense::click());
    let font = FontId {
        size: plt::font_size::META,
        family: FontFamily::Name("inter_regular".into()),
    };
    text(
        ui,
        info,
        font,
        pos2(rect.left() + 1.0, rect.center().y - 7.0),
        plt::letter_spacing::MINIMAL,
        plt::DIM,
        Align::LEFT,
    );
}

pub fn loop_button(ui: &mut egui::Ui, mode: &mut PlaybackMode) {
    let loop_icon = "\u{e042}";
    let (rect, resp) = ui.allocate_exact_size(vec2(76.0, 32.0), Sense::click());
    if resp.clicked() {
        *mode = match mode {
            PlaybackMode::Loop => PlaybackMode::Once,
            _ => PlaybackMode::Loop,
        }
    }
    let (bg, fg) = if matches!(mode, PlaybackMode::Loop) {
        (plt::TEXT, plt::INK)
    } else {
        (plt::VOID, plt::BRIGHT)
    };
    let bg = if resp.hovered() {
        if bg == plt::TEXT {
            bg
        } else {
            plt::SURFACE_HOVER
        }
    } else {
        bg
    };
    ui.painter().rect_filled(rect, SHARP, bg);
    ui.painter()
        .line_segment([rect.left_bottom(), rect.left_top()], border());
    ui.painter()
        .line_segment([rect.right_bottom(), rect.right_top()], border());
    let font_text = FontId {
        size: plt::font_size::MED,
        family: FontFamily::Name("inter_regular".into()),
    };
    let font_icon = FontId {
        size: plt::font_size::BIG,
        family: FontFamily::Name("icons".into()),
    };
    text(
        ui,
        loop_icon,
        font_icon,
        pos2(rect.center().x + 14.0, rect.bottom() - 24.),
        plt::letter_spacing::BASE,
        fg,
        Align::LEFT,
    );
    text(
        ui,
        "LOOP",
        font_text,
        pos2(rect.center().x + 10.0, rect.bottom() - 23.5),
        plt::letter_spacing::BASE,
        fg,
        Align::RIGHT,
    );
}
