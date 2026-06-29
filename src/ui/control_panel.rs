use crate::generators::fluidwave::{
    ColorArrangement, ColorMode, EnergyTransferMode, ForceDirection,
};
use crate::generators::stereometer::{
    FilterMode, LiveDensity, RenderMode, StereometerKind, TraceDensity,
};
use crate::ui::canvas::NUM_PARTICLES;
use crate::ui::{control_panel_widgets::*, palette as plt};
use crate::{GeneratorKind, state::*};
use eframe::egui::{self, vec2};

fn generator_options(ui: &mut egui::Ui, st: &mut AppState) {
    section_header_submenu(ui, "RENDER", &mut st.render_open);
    if st.render_open {
        match st.gen_kind {
            GeneratorKind::Stereometer => {
                dropdown_row(
                    ui,
                    "MODE",
                    &mut st.stereo.render_mode,
                    RenderMode::ALL,
                    &mut st.render_mode_options_open,
                );
                dropdown_row(
                    ui,
                    "STYLE",
                    &mut st.stereo.kind,
                    StereometerKind::ALL,
                    &mut st.stereo_kind_options_open,
                );
            }
            GeneratorKind::Fluidwave => {
                dropdown_row(
                    ui,
                    "ENERGY TRANSFER",
                    &mut st.fwave.energy_transfer_mode,
                    EnergyTransferMode::ALL,
                    &mut st.energy_transfer_mode_options_open,
                );
                if matches!(
                    st.fwave.energy_transfer_mode,
                    EnergyTransferMode::ForceField
                ) {
                    dropdown_row(
                        ui,
                        "FORCE DIRECTION",
                        &mut st.fwave.force_direction,
                        ForceDirection::ALL,
                        &mut st.force_direction_options_open,
                    );
                }
            }
        }
    }
    if matches!(st.stereo.render_mode, RenderMode::FullSpectrum) {
        section_header_submenu(ui, "FILTERING", &mut st.filtering_open);
        if st.filtering_open {
            dropdown_row(
                ui,
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
            slider_row(ui, "FREQ", &mut st.stereo.filter_freq, 1.0, 20000.0, 0);
        }
    }

    section_header_submenu(ui, "COLOR", &mut st.color_open);
    if st.color_open {
        match st.gen_kind {
            GeneratorKind::Stereometer => match st.stereo.render_mode {
                RenderMode::FullSpectrum => {
                    slider_row(ui, "RED", &mut st.stereo.fs_color.r, 0.0, 255.0, 0);
                    slider_row(ui, "GREEN", &mut st.stereo.fs_color.g, 0.0, 255.0, 0);
                    slider_row(ui, "BLUE", &mut st.stereo.fs_color.b, 0.0, 255.0, 0);
                }
                RenderMode::MultiBand => {
                    for (band, name) in ["LOW BAND", "MID BAND", "HIGH BAND"].iter().enumerate() {
                        static_label(ui, name);
                        slider_row(ui, "RED", &mut st.stereo.mb_color[band].r, 0.0, 255.0, 0);
                        slider_row(ui, "GREEN", &mut st.stereo.mb_color[band].g, 0.0, 255.0, 0);
                        slider_row(ui, "BLUE", &mut st.stereo.mb_color[band].b, 0.0, 255.0, 0);
                    }
                }
            },
            GeneratorKind::Fluidwave => {
                dropdown_row(
                    ui,
                    "COLOR MODE",
                    &mut st.fwave.color_mode,
                    ColorMode::ALL,
                    &mut st.color_mode_options_open,
                );
                match st.fwave.color_mode {
                    ColorMode::VelocityGradient => {
                        dropdown_row(
                            ui,
                            "COLOR ORDER",
                            &mut st.fwave.color_arrangement,
                            ColorArrangement::ALL,
                            &mut st.color_arrangement_options_open,
                        );
                        toggle_button_row(ui, "INVERT COLOR", &mut st.fwave.color_invert);
                        toggle_button_row(ui, "LUMINANCE MODE", &mut st.fwave.luminance_mode);
                        if st.fwave.luminance_mode {
                            slider_row(
                                ui,
                                "LUM FLOOR",
                                &mut st.fwave.luminance_floor,
                                0.0,
                                100.0,
                                0,
                            );
                        }
                    }
                    ColorMode::Uniform => {
                        slider_row(ui, "RED", &mut st.fwave.uniform_color.r, 0.0, 255.0, 0);
                        slider_row(ui, "GREEN", &mut st.fwave.uniform_color.g, 0.0, 255.0, 0);
                        slider_row(ui, "BLUE", &mut st.fwave.uniform_color.b, 0.0, 255.0, 0);
                    }
                }
            }
        }
    }

    section_header_submenu(ui, "VISUAL", &mut st.visual_open);
    if st.visual_open {
        match st.gen_kind {
            GeneratorKind::Stereometer => {
                dropdown_row(
                    ui,
                    "DENSITY",
                    &mut st.stereo.live_density,
                    LiveDensity::ALL,
                    &mut st.density_open,
                );
                dropdown_row(
                    ui,
                    "TRACE",
                    &mut st.stereo.trace_density,
                    TraceDensity::ALL,
                    &mut st.trace_open,
                );
                if matches!(
                    st.stereo.kind,
                    StereometerKind::ScaledBipolar | StereometerKind::ScaledLissajous
                ) {
                    slider_row(
                        ui,
                        "RADIUS",
                        &mut st.stereo.radial_scale_factor,
                        0.0,
                        1.0,
                        3,
                    );
                }
                slider_row(ui, "DOT SIZE", &mut st.stereo.point_size, 0.0005, 0.01, 4);
            }
            GeneratorKind::Fluidwave => {
                slider_row(ui, "SIM SPEED", &mut st.fwave.sim_speed, 1.0, 200.0, 1);
                slider_row(
                    ui,
                    "VISCOSITY",
                    &mut st.fwave.viscosity_amount,
                    0.0,
                    0.05,
                    3,
                );
                slider_row(ui, "DOT SIZE", &mut st.fwave.point_size, 0.0005, 0.02, 4);
                if st.advanced_mode {
                    static_label(ui, "ADVANCED SETTINGS");
                    toggle_button_row(
                        ui,
                        "ENVELOPE-PRESSURE LINK",
                        &mut st.fwave.envelope_pressure_link,
                    );
                    slider_row(
                        ui,
                        "STABILITY",
                        &mut st.fwave.target_density,
                        0.0,
                        (86.0 * NUM_PARTICLES as f32).round(),
                        0,
                    );
                    slider_row(
                        ui,
                        "PRESSURE",
                        &mut st.fwave.pressure_multiplier,
                        0.0,
                        400.0,
                        0,
                    );
                    slider_row(
                        ui,
                        "DAMPING",
                        &mut st.fwave.edge_damping_factor,
                        0.0,
                        1.0,
                        2,
                    );
                    slider_row(
                        ui,
                        "F-BOUNDS",
                        &mut st.fwave.smoothing_radius,
                        0.05,
                        0.25,
                        2,
                    );
                }
            }
        }
    }

    if matches!(st.gen_kind, GeneratorKind::Fluidwave) {
        section_header_submenu(ui, "REACTIVITY", &mut st.envelope_follower_open);
        if st.envelope_follower_open {
            slider_row(ui, "ATTACK", &mut st.fwave.attack, 0.01, 1.0, 2);
            slider_row(ui, "RELEASE", &mut st.fwave.release, 0.01, 1.0, 2);
            slider_row(ui, "RANGE", &mut st.fwave.range, 0.0, 100.0, 0);
            slider_row(
                ui,
                "AMOUNT",
                &mut st.fwave.envelope_sensitivity,
                0.0,
                200.0,
                0,
            );
        }
    }
}

fn postfx_options(ui: &mut egui::Ui, st: &mut AppState) {
    match st.gen_kind {
        GeneratorKind::Stereometer => {
            slider_row(ui, "BLOOM", &mut st.stereo.bloom, 0.0, 10.0, 1);
        }
        GeneratorKind::Fluidwave => {
            slider_row(ui, "BLOOM", &mut st.fwave.bloom, 0.0, 10.0, 1);
            slider_row(ui, "VIGNETTE", &mut st.fwave.vignette, 0.0, 1.0, 2);
        }
    }
}

pub fn draw(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Panel::right("control_panel")
        .exact_size(320.0)
        .resizable(false)
        .frame(egui::Frame::NONE.fill(plt::BG(ui.style().visuals.dark_mode)))
        .show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                egui::Panel::right("control_panel_inner")
                    .exact_size(ui.available_size().x)
                    .resizable(false)
                    .frame(egui::Frame::new().inner_margin(12.0))
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let gen_rect =
                                    section_header(ui, 1, "GENERATOR", &mut st.gen_open).rect;
                                const ITEM_W: f32 = 110.0;
                                const ITEM_H: f32 = 21.0;

                                ui.allocate_ui_with_layout(
                                    vec2(0.0, 0.0),
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let offset_padding = (gen_rect.height() - ITEM_H) / 2.0;
                                        let inner = ui
                                            .allocate_rect(
                                                egui::Rect::from_min_size(
                                                    gen_rect.right_center()
                                                        - vec2(
                                                            (ITEM_W + 1.0) + offset_padding,
                                                            ITEM_H / 2.0,
                                                        ),
                                                    vec2(ITEM_W, ITEM_H),
                                                ),
                                                egui::Sense::click(),
                                            )
                                            .rect;

                                        ui.scope_builder(
                                            egui::UiBuilder::new().max_rect(inner),
                                            |ui| {
                                                dropdown_menu(
                                                    ui,
                                                    (ITEM_W, ITEM_H),
                                                    &mut st.gen_kind,
                                                    GeneratorKind::ALL,
                                                    &mut st.gen_kind_options_open,
                                                );
                                            },
                                        );
                                    },
                                );

                                if st.gen_open {
                                    generator_options(ui, st);
                                }

                                section_header(ui, 2, "POST FX", &mut st.postfx_open);
                                if st.postfx_open {
                                    postfx_options(ui, st);
                                }
                            })
                        });
                    });
            })
        });
}
