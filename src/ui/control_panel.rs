use egui::Margin;

use crate::state::*;
use crate::ui::{control_panel_widgets::*, palette};

const ICON_IMPORT: &str = "\u{e5db}";
const ICON_EXPORT: &str = "\u{e5d8}";

pub fn draw(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Panel::right("control_panel")
        .exact_size(360.0)
        .resizable(false)
        .frame(egui::Frame::NONE.fill(palette::BG).inner_margin(Margin {
            left: 6,
            ..0.into()
        }))
        .show_inside(ui, |ui| {
            egui::Panel::right("inner_panel")
                .exact_size(ui.available_width())
                .resizable(false)
                .frame(egui::Frame::NONE.fill(palette::BG))
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        let half = ui.available_width() / 2.0;
                        if project_handler_button(ui, ICON_IMPORT, "IMPORT", half).clicked() {
                            st.import_open = true;
                        }
                        if project_handler_button(ui, ICON_EXPORT, "EXPORT", half).clicked() {}
                    });

                    ui.vertical_centered(|ui| {
                        egui::Panel::right("control_panel_inner")
                            .exact_size(ui.available_size().x)
                            .resizable(false)
                            .frame(egui::Frame::new().inner_margin(12.0))
                            .show_inside(ui, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        section_header(ui, 1, "GENERATOR", &mut st.gen_open);
                                        if st.gen_open {
                                            section_header_submenu(
                                                ui,
                                                "RENDER",
                                                &mut st.render_open,
                                            );
                                            if st.render_open {
                                                dropdown_row(
                                                    ui,
                                                    "render_mode",
                                                    "MODE",
                                                    &mut st.render_mode,
                                                    RenderMode::ALL,
                                                    &mut st.render_mode_options_open,
                                                );
                                                dropdown_row(
                                                    ui,
                                                    "render_style",
                                                    "STYLE",
                                                    &mut st.stereo_kind,
                                                    StereometerKind::ALL,
                                                    &mut st.stereo_kind_options_open,
                                                );
                                            }
                                            section_header_submenu(
                                                ui,
                                                "FILTERING",
                                                &mut st.filtering_open,
                                            );
                                            if st.filtering_open {
                                                dropdown_row(
                                                    ui,
                                                    "filter_mode",
                                                    "FILTER",
                                                    &mut st.filter_mode,
                                                    FilterMode::ALL,
                                                    &mut st.filter_mode_options_open,
                                                );
                                                slider_row(
                                                    ui,
                                                    "FREQUENCY",
                                                    &mut st.filter_freq,
                                                    1.0,
                                                    20000.0,
                                                    0,
                                                );
                                            }

                                            section_header_submenu(ui, "COLOR", &mut st.color_open);
                                            if st.color_open {
                                                match st.render_mode {
                                                    RenderMode::FullSpectrum => {
                                                        slider_row(
                                                            ui,
                                                            "HUE",
                                                            &mut st.hsl_color_bands[1].0,
                                                            0.0,
                                                            360.0,
                                                            1,
                                                        );
                                                        slider_row(
                                                            ui,
                                                            "SATURATION",
                                                            &mut st.hsl_color_bands[1].1,
                                                            0.0,
                                                            1.0,
                                                            2,
                                                        );
                                                        slider_row(
                                                            ui,
                                                            "LUMINANCE",
                                                            &mut st.hsl_color_bands[1].2,
                                                            0.0,
                                                            1.0,
                                                            2,
                                                        );
                                                    }
                                                    RenderMode::MultiBand => {
                                                        for (band, name) in
                                                            ["LOW BAND", "MID BAND", "HIGH BAND"]
                                                                .iter()
                                                                .enumerate()
                                                        {
                                                            static_label(ui, name);
                                                            slider_row(
                                                                ui,
                                                                "HUE",
                                                                &mut st.hsl_color_bands[band].0,
                                                                0.0,
                                                                360.0,
                                                                1,
                                                            );
                                                            slider_row(
                                                                ui,
                                                                "SATURATION",
                                                                &mut st.hsl_color_bands[band].1,
                                                                0.0,
                                                                1.0,
                                                                2,
                                                            );
                                                            slider_row(
                                                                ui,
                                                                "LUMINANCE",
                                                                &mut st.hsl_color_bands[band].2,
                                                                0.0,
                                                                1.0,
                                                                2,
                                                            );
                                                        }
                                                    }
                                                }
                                            }

                                            section_header_submenu(
                                                ui,
                                                "VISUAL",
                                                &mut st.visual_open,
                                            );
                                        }

                                        section_header(ui, 2, "POST FX", &mut st.postfx_open);
                                        // if st.postfx_open {
                                        //     sub_header(ui, "SPARKLE", &mut st.sparkle_open);
                                        //     if st.sparkle_open {
                                        //         hsl_row(ui, "BLOOM", &mut st.bloom, 0.0, 1.0, 2);
                                        //     }
                                        // }
                                    })
                                });
                            });
                    })
                });
        });
}
