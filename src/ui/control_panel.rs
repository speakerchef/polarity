use crate::generators::fluidwave::{EnergyTransferMode, ForceDirection};
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
                    false,
                );
                dropdown_row(
                    ui,
                    "STYLE",
                    &mut st.stereo.kind,
                    StereometerKind::ALL,
                    &mut st.bool.stereo_kind_options_open,
                    false,
                );
            }
            GenKindLabel::Fluidwave => {
                dropdown_row(
                    ui,
                    "ENERGY TRANSFER",
                    &mut st.fwave.energy_transfer_mode,
                    EnergyTransferMode::ALL,
                    &mut st.bool.energy_transfer_mode_options_open,
                    false,
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
                        false,
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
                false,
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
            slider_row(
                ui,
                "FREQ",
                &mut st.stereo.filter_freq,
                1.0,
                20000.0,
                0,
                false,
            );
        }
    }

    section_header_submenu(ui, "COLOR", &mut st.bool.color_open);
    if st.bool.color_open {
        let mut b_st = std::mem::take(&mut st.bool);
        st.active_gen().draw_color_menu(ui, &mut b_st);
        st.bool = b_st;
    }

    section_header_submenu(ui, "VISUAL", &mut st.bool.visual_open);
    if st.bool.visual_open {
        let mut bs = std::mem::take(&mut st.bool);
        st.active_gen().draw_visual_menu(ui, &mut bs);
        st.bool = bs;
    }

    section_header_submenu(ui, "REACTIVITY", &mut st.bool.envelope_follower_open);
    let (Some(env_a), Some(env_b), Some(env_c), Some(env_d)) =
        (&mut st.env_a, &mut st.env_b, &mut st.env_c, &mut st.env_d)
    else {
        return;
    };
    if st.bool.envelope_follower_open {
        section_header_submenu(ui, "ENVELOPE A", &mut st.bool.env_a_open);
        if st.bool.env_a_open {
            slider_row(ui, "ATTACK (ms)", &mut env_a.attack, 1., 500.0, 0, false);
            slider_row(ui, "RELEASE (ms)", &mut env_a.release, 1., 1000.0, 0, false);
            slider_row(
                ui,
                "SENSITIVITY",
                &mut env_a.sensitivity,
                -1.0,
                1.0,
                2,
                false,
            );
        }

        section_header_submenu(ui, "ENVELOPE B", &mut st.bool.env_b_open);
        if st.bool.env_b_open {
            slider_row(ui, "ATTACK (ms)", &mut env_b.attack, 1., 500.0, 0, false);
            slider_row(ui, "RELEASE (ms)", &mut env_b.release, 1., 1000.0, 0, false);
            slider_row(
                ui,
                "SENSITIVITY",
                &mut env_b.sensitivity,
                -1.0,
                1.0,
                2,
                false,
            );
        }

        section_header_submenu(ui, "ENVELOPE C", &mut st.bool.env_c_open);
        if st.bool.env_c_open {
            slider_row(ui, "ATTACK (ms)", &mut env_c.attack, 1., 500.0, 0, false);
            slider_row(ui, "RELEASE (ms)", &mut env_c.release, 1., 1000.0, 0, false);
            slider_row(
                ui,
                "SENSITIVITY",
                &mut env_c.sensitivity,
                -1.0,
                1.0,
                2,
                false,
            );
        }

        section_header_submenu(ui, "ENVELOPE D", &mut st.bool.env_d_open);
        if st.bool.env_d_open {
            slider_row(ui, "ATTACK (ms)", &mut env_d.attack, 1., 500.0, 0, false);
            slider_row(ui, "RELEASE (ms)", &mut env_d.release, 1., 1000.0, 0, false);
            slider_row(
                ui,
                "SENSITIVITY",
                &mut env_d.sensitivity,
                -1.0,
                1.0,
                2,
                false,
            );
        }
    }
}

fn postfx_options(ui: &mut egui::Ui, st: &mut AppState) {
    st.draw_post_fx(ui);
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
