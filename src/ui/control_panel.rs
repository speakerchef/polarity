use crate::generators::stereometer::{
    FilterMode, LiveDensity, RenderMode, StereometerKind, TraceDensity,
};
use crate::state::*;
use crate::ui::{control_panel_widgets::*, palette};

const ICON_IMPORT: &str = "\u{e5db}";
const ICON_EXPORT: &str = "\u{e5d8}";

pub fn draw(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Panel::right("control_panel")
        .exact_size(360.0)
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
                                    section_header_submenu(ui, "RENDER", &mut st.render_open);
                                    if st.render_open {
                                        dropdown_row(
                                            ui,
                                            "render_mode",
                                            "MODE",
                                            &mut st.stereo.render_mode,
                                            RenderMode::ALL,
                                            &mut st.render_mode_options_open,
                                        );
                                        dropdown_row(
                                            ui,
                                            "render_style",
                                            "STYLE",
                                            &mut st.stereo.kind,
                                            StereometerKind::ALL,
                                            &mut st.stereo_kind_options_open,
                                        );
                                    }
                                    section_header_submenu(ui, "FILTERING", &mut st.filtering_open);
                                    if st.filtering_open {
                                        dropdown_row(
                                            ui,
                                            "filter_mode",
                                            "FILTER",
                                            &mut st.stereo.filter_mode,
                                            FilterMode::ALL,
                                            &mut st.filter_mode_options_open,
                                        );
                                        if st.set_default_freqs {
                                            let f = match st.stereo.filter_mode {
                                                FilterMode::Off => 1.0,
                                                FilterMode::Lpf => 200.,
                                                FilterMode::Bpf => 1000.,
                                                FilterMode::Hpf => 5000.,
                                            };
                                            st.stereo.filter_freq = f;
                                            st.stereo.last_freq = f;
                                        }
                                        slider_row(
                                            ui,
                                            "FREQUENCY",
                                            &mut st.stereo.filter_freq,
                                            1.0,
                                            20000.0,
                                            0,
                                        );
                                    }

                                    section_header_submenu(ui, "COLOR", &mut st.color_open);
                                    if st.color_open {
                                        match st.stereo.render_mode {
                                            RenderMode::FullSpectrum => {
                                                slider_row(
                                                    ui,
                                                    "RED",
                                                    &mut st.stereo.fs_color.r,
                                                    0.0,
                                                    255.0,
                                                    0,
                                                );
                                                slider_row(
                                                    ui,
                                                    "GREEN",
                                                    &mut st.stereo.fs_color.g,
                                                    0.0,
                                                    255.0,
                                                    0,
                                                );
                                                slider_row(
                                                    ui,
                                                    "BLUE",
                                                    &mut st.stereo.fs_color.b,
                                                    0.0,
                                                    255.0,
                                                    0,
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
                                                        "RED",
                                                        &mut st.stereo.mb_color[band].r,
                                                        0.0,
                                                        255.0,
                                                        0,
                                                    );
                                                    slider_row(
                                                        ui,
                                                        "GREEN",
                                                        &mut st.stereo.mb_color[band].g,
                                                        0.0,
                                                        255.0,
                                                        0,
                                                    );
                                                    slider_row(
                                                        ui,
                                                        "BLUE",
                                                        &mut st.stereo.mb_color[band].b,
                                                        0.0,
                                                        255.0,
                                                        0,
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    section_header_submenu(ui, "VISUAL", &mut st.visual_open);
                                    if st.visual_open {
                                        dropdown_row(
                                            ui,
                                            "live_density",
                                            "DENSITY",
                                            &mut st.stereo.live_density,
                                            LiveDensity::ALL,
                                            &mut st.density_open,
                                        );
                                        dropdown_row(
                                            ui,
                                            "trace_density",
                                            "TRACE",
                                            &mut st.stereo.trace_density,
                                            TraceDensity::ALL,
                                            &mut st.trace_open,
                                        );
                                        slider_row(
                                            ui,
                                            "POINT SIZE",
                                            &mut st.stereo.point_size,
                                            0.2,
                                            4.0,
                                            2,
                                        );
                                        slider_row(
                                            ui,
                                            "SCALE (%)",
                                            &mut st.stereo.scale_factor,
                                            100.0,
                                            500.0,
                                            0,
                                        );
                                    }
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
}
