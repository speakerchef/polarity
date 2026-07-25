use std::time::Duration;

use eframe::egui;
use eframe::egui::{Align, vec2};

use crate::audio::audio_player::AudioPlayer;
use crate::state::*;
use crate::ui::{SHARP, palette as plt, timeline_widgets::*};
const SKIP_START: &str = "\u{e045}";
const PLAY: &str = "\u{e037}";
const PAUSE: &str = "\u{e034}";
const SKIP_END: &str = "\u{e044}";

pub fn draw(ui: &mut egui::Ui, st: &mut AppState, pl: Option<&AudioPlayer>) {
    egui::Panel::bottom("timeline")
        .exact_size(104.0)
        .resizable(false)
        .frame(egui::Frame::NONE.fill(plt::BG(ui.style().visuals.dark_mode)))
        .show(ui, |ui| {
            const H: f32 = 28.0;
            let bg = ui.painter().add(egui::Shape::Noop);
            let inner_rect = ui
                .allocate_ui_with_layout(
                    vec2(ui.available_width(), H),
                    egui::Layout::left_to_right(Align::Center),
                    |ui| {
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

                        timecode(ui, &elap, &dur, H);
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
                            H,
                        );
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            ui.add_space(16.0);
                            loop_button(ui, &mut st.playback_mode, H);
                        })
                    },
                )
                .response
                .rect;

            let bgr = egui::Shape::rect_filled(
                egui::Rect::from_min_size(inner_rect.left_top(), vec2(ui.available_width(), H)),
                SHARP,
                plt::BG(ui.style().visuals.dark_mode),
            );
            let transport_rect = bgr.visual_bounding_rect();
            ui.painter().set(bg, bgr);

            let mut avail_size = ui.available_size();
            avail_size.y -= 1.0;
            let mut waveform = ui.allocate_rect(
                egui::Rect::from_min_size(transport_rect.left_bottom(), avail_size),
                egui::Sense::click(),
            );
            ui.painter().line_segment(
                [waveform.rect.left_top(), waveform.rect.right_top()],
                border(ui.style().visuals.dark_mode),
            );

            if let Some(p) = pl {
                waveform = waveform.on_hover_and_drag_cursor(egui::CursorIcon::Text);
                render_waveform(ui, p, &waveform.rect);
                playback_head(ui, avail_size, transport_rect, waveform, p);
            } else {
                waveform = waveform.on_hover_and_drag_cursor(egui::CursorIcon::Cell);
                file_import_prompt(ui, st, waveform);
            }
        });
}
