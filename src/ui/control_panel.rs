use crate::generators::{Envelope, GenKind};
use crate::state::*;
use crate::traits::Labeled;
use crate::ui::{control_panel_widgets::*, palette as plt};
use eframe::egui::{self, vec2};
use rodio::cpal;
use rodio::cpal::traits::HostTrait;

fn draw_reactivity_options(st: &mut AppState, ui: &mut egui::Ui) {
    let b = &mut st.env_bank;
    let (Some(env_a), Some(env_b), Some(env_c), Some(env_d)) =
        (&mut b.env_a, &mut b.env_b, &mut b.env_c, &mut b.env_d)
    else {
        return;
    };
    let sens = |ui: &mut egui::Ui, env: &mut Envelope| {
        slider_row(ui, "SENSITIVITY", &mut env.sensitivity, -1.0, 1.0, 2, false);
    };

    if st.bool.envelope_follower_open {
        section_header_submenu(ui, "ENVELOPE A", &mut st.bool.env_a_open);
        if st.bool.env_a_open {
            slider_row(ui, "ATTACK (ms)", &mut env_a.attack, 1., 500.0, 0, false);
            slider_row(ui, "RELEASE (ms)", &mut env_a.release, 1., 1000.0, 0, false);
            sens(ui, env_a);
        }

        section_header_submenu(ui, "ENVELOPE B", &mut st.bool.env_b_open);
        if st.bool.env_b_open {
            slider_row(ui, "ATTACK (ms)", &mut env_b.attack, 1., 500.0, 0, false);
            slider_row(ui, "RELEASE (ms)", &mut env_b.release, 1., 1000.0, 0, false);
            sens(ui, env_b);
        }

        section_header_submenu(ui, "ENVELOPE C", &mut st.bool.env_c_open);
        if st.bool.env_c_open {
            slider_row(ui, "ATTACK (ms)", &mut env_c.attack, 1., 500.0, 0, false);
            slider_row(ui, "RELEASE (ms)", &mut env_c.release, 1., 1000.0, 0, false);
            sens(ui, env_c);
        }

        section_header_submenu(ui, "ENVELOPE D", &mut st.bool.env_d_open);
        if st.bool.env_d_open {
            slider_row(ui, "ATTACK (ms)", &mut env_d.attack, 1., 500.0, 0, false);
            slider_row(ui, "RELEASE (ms)", &mut env_d.release, 1., 1000.0, 0, false);
            sens(ui, env_d);
        }
    }
}

fn input_mode_options(ui: &mut egui::Ui, st: &mut AppState) {
    dropdown_row(
        ui,
        "INPUT MODE",
        &mut st.input_mode,
        InputMode::ALL,
        &mut st.bool.input_mode_options_open,
        false,
    );
    if st.input_mode == InputMode::Live {
        let available_inputs = if st.bool.input_device_options_open {
            if let Some(avail_inputs) = &st.available_input_devices {
                avail_inputs.clone()
            } else {
                let dev = cpal::default_host().output_devices();
                let mut input_devices = vec![InputDevice(DEFAULT_DEVICE.into())];
                input_devices.extend(if let Ok(dev_list) = dev {
                    dev_list
                        .map(InputDevice::from)
                        .collect::<Vec<InputDevice>>()
                } else {
                    Vec::default()
                });
                st.available_input_devices = Some(input_devices.clone());
                input_devices
            }
        } else {
            st.available_input_devices.take();
            Vec::default()
        };
        dropdown_row(
            ui,
            "INPUT DEVICE",
            &mut st.new_live_input_device,
            &available_inputs,
            &mut st.bool.input_device_options_open,
            false,
        );
    }
    let gain_label = if st.input_mode == InputMode::Live {
        "IN GAIN"
    } else {
        "OUT GAIN"
    };
    slider_row(ui, gain_label, &mut st.gain_db, -30.0, 6.0, 1, false);
    toggle_button_row(ui, "INPUT METER", &mut st.bool.show_level_meter, false);
    if st.bool.show_level_meter {
        toggle_button_row(ui, "METER GRADIENT", &mut st.bool.use_meter_gradient, false);
        if !st.bool.use_meter_gradient {
            static_label(ui, "METER COLOR");
            slider_row(ui, "RED", &mut st.meter_color.r, 0.0, 255.0, 0, false);
            slider_row(ui, "GREEN", &mut st.meter_color.g, 0.0, 255.0, 0, false);
            slider_row(ui, "BLUE", &mut st.meter_color.b, 0.0, 255.0, 0, false);
        }
    }
}

fn generator_options(ui: &mut egui::Ui, st: &mut AppState) {
    let mut open = std::mem::take(&mut st.bool);
    st.active_gen().draw_render_menu(ui, &mut open);
    st.active_gen().draw_filtering_menu(ui, &mut open);
    st.active_gen().draw_color_menu(ui, &mut open);
    st.active_gen().draw_visual_menu(ui, &mut open);
    section_header_submenu(ui, "REACTIVITY", &mut open.envelope_follower_open);
    st.bool = open;

    draw_reactivity_options(st, ui);
}

fn inner_header_dropdown<T: Labeled + Default>(
    ui: &mut egui::Ui,
    target_rect: &egui::Rect,
    value: &mut T,
    opts: &[T],
    open: &mut bool,
) {
    const ITEM_W: f32 = 110.0;
    const ITEM_H: f32 = 21.0;

    ui.allocate_ui_with_layout(
        vec2(0.0, 0.0),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            let offset_padding = (target_rect.height() - ITEM_H) / 2.0;
            let inner = ui
                .allocate_rect(
                    egui::Rect::from_min_size(
                        target_rect.right_center()
                            - vec2((ITEM_W + 1.0) + offset_padding, ITEM_H / 2.0),
                        vec2(ITEM_W, ITEM_H),
                    ),
                    egui::Sense::click(),
                )
                .rect;

            ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                dropdown_menu(ui, (ITEM_W, ITEM_H), value, opts, open);
            });
        },
    );
}

pub fn draw(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Panel::right("control_panel")
        .exact_size(320.0)
        .resizable(false)
        .frame(egui::Frame::NONE.fill(plt::BG(ui.style().visuals.dark_mode)))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                egui::Panel::right("control_panel_inner")
                    .exact_size(ui.available_size().x)
                    .resizable(false)
                    .frame(egui::Frame::new().inner_margin(12.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                section_header(ui, 1, "INPUT", &mut st.bool.input_open);
                                if st.bool.input_open {
                                    input_mode_options(ui, st);
                                }

                                let gen_rect =
                                    section_header(ui, 2, "GENERATOR", &mut st.bool.gen_open).rect;
                                inner_header_dropdown(
                                    ui,
                                    &gen_rect,
                                    &mut st.gen_kind,
                                    GenKind::ALL,
                                    &mut st.bool.gen_kind_options_open,
                                );
                                if st.bool.gen_open {
                                    generator_options(ui, st);
                                }

                                section_header(ui, 3, "POST FX", &mut st.bool.postfx_open);
                                if st.bool.postfx_open {
                                    st.draw_post_fx(ui);
                                }
                            })
                        });
                    });
            })
        });
}
