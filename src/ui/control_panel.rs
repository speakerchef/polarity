use crate::generators::fluidwave::{
    ColorArrangement, ColorMode, EnergyTransferMode, ForceDirection,
};
use crate::generators::stereometer::{FilterMode, RenderMode, StereometerKind};
use crate::ui::{control_panel_widgets::*, palette as plt};
use crate::{GenKindLabel, state::*};
use eframe::egui::{self, vec2};

fn generator_options(ui: &mut egui::Ui, st: &mut AppState) {
    section_header_submenu(ui, "RENDER", &mut st.bool.render_open);
    if st.bool.render_open {
        match st.gen_kind {
            GenKindLabel::Stereometer => {
                dropdown_row(
                    ui,
                    "MODE",
                    &mut st.stereo.render_mode,
                    RenderMode::ALL,
                    &mut st.bool.render_mode_options_open,
                );
                dropdown_row(
                    ui,
                    "STYLE",
                    &mut st.stereo.kind,
                    StereometerKind::ALL,
                    &mut st.bool.stereo_kind_options_open,
                );
            }
            GenKindLabel::Fluidwave => {
                dropdown_row(
                    ui,
                    "ENERGY TRANSFER",
                    &mut st.fwave.energy_transfer_mode,
                    EnergyTransferMode::ALL,
                    &mut st.bool.energy_transfer_mode_options_open,
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
                        &mut st.bool.force_direction_options_open,
                    );
                }
            }
        }
    }
    if matches!(st.stereo.render_mode, RenderMode::FullSpectrum) {
        section_header_submenu(ui, "FILTERING", &mut st.bool.filtering_open);
        if st.bool.filtering_open {
            dropdown_row(
                ui,
                "FILTER",
                &mut st.stereo.filter_mode,
                FilterMode::ALL,
                &mut st.bool.filter_mode_options_open,
            );
            if st.bool.set_default_freqs {
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

    section_header_submenu(ui, "COLOR", &mut st.bool.color_open);
    if st.bool.color_open {
        match st.gen_kind {
            GenKindLabel::Stereometer => match st.stereo.render_mode {
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
            GenKindLabel::Fluidwave => {
                dropdown_row(
                    ui,
                    "COLOR MODE",
                    &mut st.fwave.color_mode,
                    ColorMode::ALL,
                    &mut st.bool.color_mode_options_open,
                );
                match st.fwave.color_mode {
                    ColorMode::VelocityGradient => {
                        dropdown_row(
                            ui,
                            "COLOR ORDER",
                            &mut st.fwave.color_arrangement,
                            ColorArrangement::ALL,
                            &mut st.bool.color_arrangement_options_open,
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

    section_header_submenu(ui, "VISUAL", &mut st.bool.visual_open);
    if st.bool.visual_open {
        let mut bs = std::mem::take(&mut st.bool);
        st.active_gen().draw_visual_menu(ui, &mut bs);
        st.bool = bs;
    }

    section_header_submenu(ui, "REACTIVITY", &mut st.bool.envelope_follower_open);
    let (Some(env_a), Some(env_b)) = (&mut st.env_a, &mut st.env_b) else {
        return;
    };
    if st.bool.envelope_follower_open {
        static_label(ui, "ENVELOPE A");
        if st.bool.advanced_mode {
            slider_row(ui, "ATTACK (ms)", &mut env_a.attack, 1., 500.0, 0);
            slider_row(ui, "RELEASE (ms)", &mut env_a.release, 1., 1000.0, 0);
        }
        slider_row(ui, "SENSITIVITY", &mut env_a.sensitivity, -1.0, 1.0, 2);

        static_label(ui, "ENVELOPE B");
        if st.bool.advanced_mode {
            slider_row(ui, "ATTACK (ms)", &mut env_b.attack, 1., 500.0, 0);
            slider_row(ui, "RELEASE (ms)", &mut env_b.release, 1., 1000.0, 0);
        }
        slider_row(ui, "SENSITIVITY", &mut env_b.sensitivity, -1.0, 1.0, 2);
    }
}

fn postfx_options(ui: &mut egui::Ui, st: &mut AppState) {
    let mut os = std::mem::take(&mut st.bool);
    st.active_gen().draw_post_fx(ui, &mut os);
    st.bool = os;
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
                                    section_header(ui, 1, "GENERATOR", &mut st.bool.gen_open).rect;
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
                                                    GenKindLabel::ALL,
                                                    &mut st.bool.gen_kind_options_open,
                                                );
                                            },
                                        );
                                    },
                                );

                                if st.bool.gen_open {
                                    generator_options(ui, st);
                                }

                                section_header(ui, 2, "POST FX", &mut st.bool.postfx_open);
                                if st.bool.postfx_open {
                                    postfx_options(ui, st);
                                }
                            })
                        });
                    });
            })
        });
}
