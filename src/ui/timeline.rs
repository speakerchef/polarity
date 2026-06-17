use std::time::Duration;

use egui::{Align, CornerRadius, Stroke, pos2, vec2};

use crate::audio::audio_player::AudioPlayer;
use crate::state::*;
use crate::ui::{custom_text, palette as plt, timeline_widgets::*};

fn render_waveform(ui: &mut egui::Ui, p: &AudioPlayer, rect: &egui::Rect) {
    let (left_channel, right_channel) = p
        .contents
        .samples
        .chunks_exact(2)
        .map(|frame| {
            let l = frame.first().unwrap();
            let r = frame.last().unwrap_or(l);
            (l, r)
        })
        .collect::<(Vec<f32>, Vec<f32>)>();

    let avail_w = rect.width() * 6.0;
    let avail_h = rect.height();
    let step: usize = (left_channel.len() as f32 / avail_w) as usize;
    let mut prev_left_top = pos2(0.0, 0.0);
    (0..avail_w as usize).for_each(|i| {
        let slice = &left_channel[i * step..i * step + step];
        let max = slice
            .iter()
            .fold(0.0, |max, &v| if v.abs() >= max { v.abs() } else { max });
        let h = max * (avail_h / 2.0);
        let fill_col = plt::WAVEFORM;
        let mut inner = ui.allocate_rect(
            egui::Rect::from_min_size(
                pos2(
                    rect.left_top().x + i as f32 / 6.0,
                    rect.left_center().y - 0.25,
                ),
                vec2(0.3, h.abs()),
            ),
            egui::Sense::hover(),
        );
        // remove interaction
        inner.interact_rect.set_width(0.0);
        inner.interact_rect.set_height(0.0);

        ui.painter().rect_filled(inner.rect, SHARP, fill_col);
        ui.painter().line_segment(
            [prev_left_top, inner.rect.left_bottom()],
            Stroke {
                width: 0.3,
                color: fill_col,
            },
        );
        prev_left_top = inner.rect.left_bottom();
    });

    let step: usize = (right_channel.len() as f32 / avail_w) as usize;
    let mut prev_left_top = pos2(0.0, 0.0);
    (0..avail_w as usize).for_each(|i| {
        let slice = &right_channel[i * step..i * step + step];
        let max = slice
            .iter()
            .fold(0.0, |max, &v| if v.abs() >= max { v.abs() } else { max });
        let h = max * (avail_h / 2.0);
        let fill_col = plt::WAVEFORM;
        let mut inner = ui.allocate_rect(
            egui::Rect::from_min_size(
                pos2(
                    rect.left_top().x + i as f32 / 6.0,
                    rect.left_center().y - h.abs(),
                ),
                vec2(0.3, h.abs()),
            ),
            egui::Sense::hover(),
        );
        // remove interaction
        inner.interact_rect.set_width(0.0);
        inner.interact_rect.set_height(0.0);

        ui.painter().rect_filled(inner.rect, SHARP, fill_col);
        ui.painter().line_segment(
            [prev_left_top, inner.rect.left_top()],
            Stroke {
                width: 0.3,
                color: fill_col,
            },
        );
        prev_left_top = inner.rect.left_top();
    });
}

pub fn draw(ui: &mut egui::Ui, st: &mut AppState, pl: &mut Option<AudioPlayer>) {
    egui::Panel::bottom("timeline")
        .exact_size(104.0)
        .resizable(false)
        .frame(egui::Frame::NONE.fill(plt::BG))
        .show_inside(ui, |ui| {
            const H: f32 = 32.0;
            let bg = ui.painter().add(egui::Shape::Noop);
            let inner_rect = ui
                .allocate_ui_with_layout(
                    vec2(ui.available_width(), H),
                    egui::Layout::left_to_right(Align::Center),
                    |ui| {
                        const SKIP_START: &str = "\u{e045}";
                        const PLAY: &str = "\u{e037}";
                        const PAUSE: &str = "\u{e034}";
                        const SKIP_END: &str = "\u{e044}";
                        ui.add_space(16.0);

                        let skip_start_resp = transport_button(ui, SKIP_START, plt::DIM, false);
                        ui.add_space(12.0);

                        let (icn, col) = if let Some(p) = pl
                            && !p.is_paused()
                        {
                            (PAUSE, plt::YELLO)
                        } else {
                            (PLAY, plt::LIVE)
                        };
                        let playback_resp = transport_button(ui, icn, col, true);
                        ui.add_space(12.0);

                        let skip_end_resp = transport_button(ui, SKIP_END, plt::DIM, false);
                        ui.add_space(16.0);

                        let (elap, dur, fname, sr) = if let Some(p) = pl {
                            if skip_start_resp.clicked() {
                                p.try_seek(Duration::from_secs(0)).unwrap_or_default();
                            }
                            if playback_resp.clicked() {
                                p.toggle_playback();
                            }
                            if skip_end_resp.clicked() {
                                p.try_seek(p.contents.duration).unwrap_or_default();
                            }
                            (
                                p.position(),
                                p.contents.duration,
                                p.contents
                                    .path
                                    .clone()
                                    .iter()
                                    .next_back()
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string(),
                                Some(p.contents.sample_rate),
                            )
                        } else {
                            (
                                Duration::default(),
                                Duration::default(),
                                "".to_string(),
                                None,
                            )
                        };

                        timecode(ui, &elap, &dur);
                        ui.add_space(16.0);

                        file_info(
                            ui,
                            &format!(
                                "{} {}",
                                fname,
                                &if let Some(sr) = sr {
                                    let sr = sr as f64 / 1000.0;
                                    format!("• {:.1}kHz", sr)
                                } else {
                                    "".to_string()
                                }
                            ),
                        );
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(16.0);
                            loop_button(ui, &mut st.playback_mode);
                        })
                    },
                )
                .response
                .rect;

            let bgr = egui::Shape::rect_filled(
                egui::Rect::from_min_size(inner_rect.left_top(), vec2(ui.available_width(), H)),
                SHARP,
                plt::BG,
            );
            let transport_rect = bgr.visual_bounding_rect();
            ui.painter().set(bg, bgr);
            ui.painter().line_segment(
                [transport_rect.left_bottom(), transport_rect.right_bottom()],
                border(),
            );

            let avail_size = ui.available_size();
            let waveform = ui
                .allocate_rect(
                    egui::Rect::from_min_size(transport_rect.left_bottom(), avail_size),
                    egui::Sense::click(),
                )
                .on_hover_and_drag_cursor(egui::CursorIcon::Text);
            if let Some(p) = pl {
                render_waveform(ui, p, &waveform.rect);
                let playback_head = ui.allocate_rect(
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
            } else {
                custom_text(
                    ui,
                    "No File Loaded",
                    egui::FontId {
                        size: plt::font_size::BIG,
                        family: egui::FontFamily::Name("inter_regular".into()),
                    },
                    pos2(
                        waveform.rect.center_top().x - 48.0,
                        waveform.rect.left_center().y - 8.0,
                    ),
                    plt::letter_spacing::BASE,
                    plt::WAVEFORM,
                    Align::LEFT,
                );
            }
        });
}
