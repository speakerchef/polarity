use std::{ops::Sub, time::Duration};

use eframe::egui;
use eframe::egui::{
    Align, Color32, CornerRadius, FontFamily, FontId, Response, Sense, Stroke, StrokeKind, Vec2,
    pos2, vec2,
};

use crate::ui::{SHARP, apply_side_border, get_text_size};
use crate::{
    audio::audio_inputs::AudioPlayer,
    state::{AppState, PlaybackMode},
    ui::{custom_text, palette as plt},
};

pub fn border(dark: bool) -> Stroke {
    Stroke::new(plt::FRAME_WIDTH, plt::BORDER(dark))
}

pub fn transport_button(ui: &mut egui::Ui, icon: &str, c: Color32, big: bool) -> Response {
    let ui_rect = ui.available_rect_before_wrap();
    let mut resp = ui.allocate_rect(
        egui::Rect::from_min_size(
            pos2(ui_rect.left_top().x, ui_rect.left_top().y + 2.0),
            vec2(28.0, 24.0),
        ),
        egui::Sense::click(),
    );
    resp = resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);

    let rect = resp.rect;
    let d = ui.style().visuals.dark_mode;
    let (bg, stk) = if resp.hovered() {
        (plt::SURFACE_HOVER(d), plt::BORDER(d))
    } else {
        (plt::BG(d), plt::BG(d))
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
    custom_text(
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

pub fn timecode(ui: &mut egui::Ui, elapsed: &Duration, dur: &Duration, h: f32) {
    let w = 96.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::click());
    ui.painter().rect_filled(rect, SHARP, plt::VOID());
    apply_side_border(ui, rect, false);
    let font = FontId {
        size: plt::font_size::BODY,
        family: FontFamily::Name("mono".into()),
    };
    let esecs = elapsed.as_secs_f64();
    let (emins, esecs) = ((esecs / 60.0) as u64, (esecs % 60.0) as u64);
    let dsecs = dur.as_secs_f64();
    let (dmins, dsecs) = ((dsecs / 60.0) as u64, (dsecs % 60.0) as u64);
    let (_, th) = get_text_size(ui, &format!("{emins:02}:{esecs:02}"), font.clone()).into();
    custom_text(
        ui,
        &format!("{emins:02}:{esecs:02}"),
        font.clone(),
        rect.left_center() + vec2(w / 4.0, -th / 2.0),
        0.,
        plt::BRIGHT,
        Align::Center,
    );
    custom_text(
        ui,
        "/",
        FontId {
            size: plt::font_size::MED,
            family: FontFamily::Name("inter_regular".into()),
        },
        pos2(rect.center().x - 2.0, rect.left_center().y - (h / 3.5)),
        plt::letter_spacing::SPACED,
        plt::BRIGHT,
        Align::LEFT,
    );
    custom_text(
        ui,
        &format!("{dmins:02}:{dsecs:02}"),
        font,
        rect.right_center() - vec2(w / 4.0, th / 2.0),
        0.,
        plt::BRIGHT,
        Align::Center,
    );
}
pub fn file_info(ui: &mut egui::Ui, info: &str, h: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(info.len() as f32 * 6., h), Sense::click());
    let font = FontId {
        size: plt::font_size::META,
        family: FontFamily::Name("inter_regular".into()),
    };
    let (_, th) = get_text_size(ui, info, font.clone()).into();
    custom_text(
        ui,
        info,
        font,
        rect.left_center() + vec2(0.0, -th / 2.0),
        plt::letter_spacing::MINIMAL,
        plt::DIM,
        Align::LEFT,
    );
}

pub fn loop_button(ui: &mut egui::Ui, mode: &mut PlaybackMode, h: f32) {
    let loop_icon = "\u{e042}";
    const W: f32 = 72.0;
    let mut resp = ui.allocate_rect(
        egui::Rect::from_min_size(
            ui.available_rect_before_wrap()
                .right_top()
                .sub(pos2(W, 0.0))
                .to_pos2(),
            vec2(W, h),
        ),
        egui::Sense::click(),
    );
    resp = resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
    if resp.clicked() {
        *mode = match mode {
            PlaybackMode::Loop => PlaybackMode::Once,
            _ => PlaybackMode::Loop,
        }
    }
    let (bg, fg) = if matches!(mode, PlaybackMode::Loop) {
        (plt::YELLO, plt::INK)
    } else {
        (plt::VOID(), plt::BRIGHT)
    };
    let bg = if resp.hovered() {
        if bg == plt::YELLO {
            bg
        } else {
            plt::SURFACE_HOVER(ui.style().visuals.dark_mode)
        }
    } else {
        bg
    };
    ui.painter().rect_filled(resp.rect, SHARP, bg);
    apply_side_border(ui, resp.rect, false);
    let font_text = FontId {
        size: plt::font_size::BODY,
        family: FontFamily::Name("inter_medium".into()),
    };
    let font_icon = FontId {
        size: plt::font_size::MED,
        family: FontFamily::Name("icons".into()),
    };
    custom_text(
        ui,
        loop_icon,
        font_icon,
        pos2(
            resp.rect.center().x + 14.0,
            resp.rect.left_center().y - (h / 4.0),
        ),
        plt::letter_spacing::BASE,
        fg,
        Align::LEFT,
    );
    custom_text(
        ui,
        "LOOP",
        font_text,
        pos2(
            resp.rect.center().x + 10.0,
            resp.rect.left_center().y - (h / 4.0),
        ),
        plt::letter_spacing::BASE,
        fg,
        Align::RIGHT,
    );
}

pub fn render_waveform(ui: &mut egui::Ui, p: &AudioPlayer, rect: &egui::Rect) {
    let avail_w = rect.width() * 6.0;
    let avail_h = rect.height() - 1.0;

    let s = p.contents.samples.as_ref();
    let num_ch = p.contents.num_channels as usize;

    let step: usize = ((s.len() / num_ch) as f32 / avail_w) as usize;
    let mut prev_left_top_l = pos2(0.0, 0.0);
    let mut prev_left_top_r = pos2(0.0, 0.0);

    (0..avail_w as usize).for_each(|i| {
        let idx = i * num_ch;
        let left_slice = s.get(idx * step..idx * step + step).unwrap_or_default();
        let right_slice = s
            .get((idx + 1) * step..(idx + 1) * step + step)
            .unwrap_or_default();
        let left_max = left_slice
            .iter()
            .fold(0.0, |max, &v| if v.abs() >= max { v.abs() } else { max })
            .min(1.0);
        let right_max = right_slice
            .iter()
            .fold(0.0, |max, &v| if v.abs() >= max { v.abs() } else { max })
            .min(1.0);
        let hl = left_max * (avail_h / 2.0);
        let hr = right_max * (avail_h / 2.0);

        let fill_col = plt::WAVEFORM_BG(ui.style().visuals.dark_mode);
        let left_rect = ui
            .allocate_rect(
                egui::Rect::from_min_size(
                    pos2(
                        rect.left_top().x + i as f32 / 6.0,
                        rect.left_center().y - 0.25,
                    ),
                    vec2(0.3, hl.abs()),
                ),
                egui::Sense::hover(),
            )
            .rect;
        let right_rect = ui
            .allocate_rect(
                egui::Rect::from_min_size(
                    pos2(
                        rect.left_top().x + i as f32 / 6.0,
                        rect.left_center().y - hr.abs(),
                    ),
                    vec2(0.3, hr.abs()),
                ),
                egui::Sense::hover(),
            )
            .rect;

        ui.painter().rect_filled(left_rect, SHARP, fill_col);
        ui.painter().rect_filled(right_rect, SHARP, fill_col);

        let stroke = Stroke {
            width: 0.3,
            color: fill_col,
        };
        ui.painter()
            .line_segment([prev_left_top_l, left_rect.left_bottom()], stroke);
        ui.painter()
            .line_segment([prev_left_top_r, right_rect.left_top()], stroke);
        prev_left_top_l = left_rect.left_bottom();
        prev_left_top_r = right_rect.left_top();
    });
}

pub fn playback_head(
    ui: &mut egui::Ui,
    avail_size: Vec2,
    transport_rect: egui::Rect,
    waveform: egui::Response,
    p: &AudioPlayer,
) {
    let mut playback_head = ui.allocate_rect(
        egui::Rect::from_min_size(
            pos2(
                transport_rect.left_bottom().x
                    + (p.position().as_secs_f32() / p.contents.duration.as_secs_f32())
                        * avail_size.x,
                transport_rect.left_bottom().y,
            ),
            vec2(1.5, avail_size.y),
        ),
        egui::Sense::click(),
    );
    playback_head.interact_rect.set_height(0.0);
    playback_head.interact_rect.set_width(0.0);

    ui.painter().rect_filled(
        playback_head.rect,
        CornerRadius::from(1.),
        if p.is_paused() { plt::YELLO } else { plt::LIVE },
    );

    // click to seek
    if ui
        .ctx()
        .input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
        && (waveform.hovered() || waveform.dragged())
    {
        ui.ctx().input(|i| {
            i.pointer.hover_pos().inspect(|pos| {
                let ratio = (pos.x - 14.0) / avail_size.x;
                p.try_seek(Duration::from_secs_f32(
                    (ratio * p.contents.duration.as_secs_f32()).max(0.0),
                ))
                .unwrap();
            })
        });
    }
}

pub fn file_import_prompt(ui: &mut egui::Ui, st: &mut AppState, waveform: egui::Response) {
    if waveform.clicked() {
        st.bool.import_open = true;
    }
    custom_text(
        ui,
        "\u{f09b}",
        egui::FontId {
            size: plt::font_size::ICON,
            family: egui::FontFamily::Name("icons".into()),
        },
        pos2(
            waveform.rect.center_top().x - 64.0,
            waveform.rect.left_center().y - 9.5,
        ),
        plt::letter_spacing::BASE,
        plt::DIM,
        Align::LEFT,
    );
    custom_text(
        ui,
        "IMPORT AUDIO",
        egui::FontId {
            size: plt::font_size::BODY,
            family: egui::FontFamily::Name("inter_regular".into()),
        },
        pos2(
            waveform.rect.center_top().x - 40.0,
            waveform.rect.left_center().y - 6.0,
        ),
        plt::letter_spacing::BASE,
        plt::DIM,
        Align::LEFT,
    );
}
