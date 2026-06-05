use std::time::Duration;

use egui::{Align, vec2};

use crate::audio::audio_player::AudioPlayer;
use crate::state::*;
use crate::ui::{palette as plt, timeline_widgets::*};

pub fn draw(ui: &mut egui::Ui, st: &mut AppState, mut pl: &mut Option<AudioPlayer>) {
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

                        if transport_button(ui, SKIP_START, plt::DIM, false).clicked()
                            && let Some(p) = &mut pl
                        {
                            p.try_seek(Duration::from_secs(0))
                                .expect("could not seek to start");
                        }
                        ui.add_space(12.0);

                        let (icn, col) = if let Some(p) = pl
                            && !p.is_paused()
                        {
                            (PAUSE, plt::WARN)
                        } else {
                            (PLAY, plt::LIVE)
                        };
                        if transport_button(ui, icn, col, true).clicked()
                            && let Some(p) = &mut pl
                        {
                            match p.is_paused() {
                                true => p.play(),
                                _ => p.pause(),
                            }
                        };
                        ui.add_space(12.0);

                        if transport_button(ui, SKIP_END, plt::DIM, false).clicked()
                            && let Some(p) = &mut pl
                        {
                            p.try_seek(p.contents.duration)
                                .expect("could not seek to end");
                        };
                        ui.add_space(16.0);

                        timecode(
                            ui,
                            &st.elapsed_time,
                            &if let Some(pl) = pl {
                                pl.contents.duration
                            } else {
                                Duration::from_secs(0)
                            },
                        );
                        ui.add_space(16.0);

                        let (fname, sr) = if let Some(p) = pl {
                            (
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
                            ("".to_string(), None)
                        };
                        file_info(ui, &fname);
                        ui.add_space(6.0);
                        file_info(
                            ui,
                            &if let Some(sr) = sr {
                                let sr = sr as f64 / 1000.0;
                                format!("• {:.1}kHz", sr)
                            } else {
                                "".to_string()
                            },
                        );
                    },
                )
                .response
                .rect;
            let bgr = egui::Shape::rect_filled(
                egui::Rect::from_min_size(inner_rect.left_top(), vec2(ui.available_width(), H)),
                SHARP,
                plt::BG,
            );
            let rect = bgr.visual_bounding_rect();
            ui.painter().set(bg, bgr);
            ui.painter()
                .line_segment([rect.left_bottom(), rect.right_bottom()], border())
            // ui.label(format!("{:?}", st.elapsed_time));
        });
}
