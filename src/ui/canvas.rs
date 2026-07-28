#![allow(dead_code, unused)]
use std::ops::Div;
use std::time::{Duration, Instant};

use crate::audio::level_meter;
use crate::ui::app_widgets::{modal, modal_button};
use crate::ui::{get_text_size, palette as plt};
use eframe::egui::{self, Pos2, Rect, Stroke, Vec2};
use eframe::egui::{Align, Color32, FontId, StrokeKind, pos2, vec2};
use eframe::egui_wgpu;
use egui_winit::winit::dpi::PhysicalSize;

use crate::AudioCapturePermission;
use crate::audio::audio_inputs::LiveInput;
use crate::generators::fluidwave::{EnergyTransferMode, ModSrc};
use crate::generators::rendering::{EffectsCallback, MeterData, OutputCallback, RendererCallback};
use crate::state::InputMode;
use crate::traits::{AudioSrc, Generator};
use crate::ui::{SHARP, canvas_widgets::presentation_buttons, timeline_widgets::border};
use crate::{audio::audio_inputs::AudioPlayer, state::AppState};

use crate::ui::{custom_text, palette};

const MAX_VIGNETTE: f32 = 0.5;
const METER_WIDTH: f32 = 10.0;

pub fn draw(ui: &mut egui::Ui, st: &mut AppState, frame: &eframe::Frame) {
    ui.ctx().request_repaint();
    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(palette::VOID())
                .inner_margin(0.0)
                .outer_margin(0.0),
        )
        .show(ui, |ui| {
            if ui.ctx().input(
                |i| i.pointer.hover_pos().is_some(), /* is cursor on window */
            ) || !st.bool.fullscreen
            {
                presentation_buttons(ui, st, frame);
            }

            lock_aspect_ratio(ui, st, frame);

            #[cfg(target_os = "macos")]
            if let Some(win) = frame.winit_window() {
                if st.bool.fullscreen {
                    win.set_window_level(egui_winit::winit::window::WindowLevel::AlwaysOnTop);
                } else {
                    win.set_window_level(egui_winit::winit::window::WindowLevel::Normal);
                }
            }

            if st.input_mode == InputMode::Live
                && st.audio_capture_permission == AudioCapturePermission::Denied
            {
                audio_capture_permission_prompt(ui, st);
            } else {
                custom_painting(ui, st);
            }
        });
}

pub const NUM_PARTICLES: i32 = 70;
pub const fn generate_particle_grid() -> [[f32; 8]; (NUM_PARTICLES * NUM_PARTICLES) as usize] {
    let mut pos =
        [[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; (NUM_PARTICLES * NUM_PARTICLES) as usize];
    let mut idx = 0;
    let mut i = 0;
    while i < NUM_PARTICLES {
        let mut j = 0;
        while j < NUM_PARTICLES {
            let x = (i as f32 + 0.5) / NUM_PARTICLES as f32 * 0.9;
            let y = (j as f32 + 0.5) / NUM_PARTICLES as f32 * 0.9;
            pos[idx] = [x, y, -x, -y, x, -y, -x, y];
            idx += 1;
            j += 1;
        }
        i += 1;
    }
    pos
}

fn custom_painting(ui: &mut egui::Ui, st: &mut AppState) {
    let (h, w) = (ui.available_height(), ui.available_width());
    let l = (h).min(w);
    let canvas_size = vec2(l, l);

    let center = ui.max_rect().center();
    let top_left = pos2(center.x - l / 2.0, center.y - l / 2.0);
    let rect = ui
        .allocate_rect(
            egui::Rect::from_min_size(top_left, canvas_size),
            egui::Sense::focusable_noninteractive(),
        )
        .rect;

    ui.painter().rect_stroke(
        rect,
        SHARP,
        Stroke {
            width: 0.5,
            color: plt::GRAY,
        },
        StrokeKind::Outside,
    );

    let live_input = std::mem::take(&mut st.live_input);
    let player = st.player.take();
    let active_input: &dyn AudioSrc = {
        match st.input_mode {
            InputMode::Live => &live_input,
            InputMode::File => {
                if let Some(pl) = player.as_ref() {
                    pl
                } else {
                    st.live_input = live_input;
                    st.player = player;
                    return;
                }
            }
        }
    };

    st.env_bank.run_follower(active_input, None);

    let mut fbank = std::mem::take(&mut st.filterbank);
    let env_bank = std::mem::take(&mut st.env_bank);

    st.active_gen()
        .prepare(&mut fbank, &env_bank, active_input, None);

    st.filterbank = fbank;
    st.env_bank = env_bank;
    st.replace_inputs(player, live_input);

    let renderer_params = st.build_renderer_callback_params(true, 0);
    let efx_params = st.build_effects_callback_params(None);
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        RendererCallback {
            canvas_size,
            params: renderer_params,
        },
    ));
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        EffectsCallback {
            top_left: rect.left_top(),
            ..efx_params
        },
    ));
}

fn lock_aspect_ratio(ui: &mut egui::Ui, st: &mut AppState, frame: &eframe::Frame) {
    if st.bool.fullscreen
        && st.bool.lock_aspect_ratio
        && let Some(win) = frame.winit_window()
        && win.inner_size().height != win.inner_size().width
    {
        let (w, h) = (win.inner_size().width, win.inner_size().height);
        let l = if w != st.last_window_width {
            st.last_window_width = w;
            w
        } else {
            h
        };
        let _ = win.request_inner_size(PhysicalSize::new(l, l));
    }
}

fn audio_capture_permission_prompt(ui: &mut egui::Ui, st: &mut AppState) {
    let msg = cfg_select! {
        target_os = "macos" => "You haven't granted Polarity audio-capture \
            permissions to work in live mode.\n\n Please grant them through \
            system settings under:\n\n Privacy & Security > Screen & System \
            Audio Recording > System Audio Recording only.",
        _ => "You haven't granted Polarity audio-capture permissions to work in live mode.\n Please grant them in your settings to use live mode."
    };
    let font = FontId {
        size: plt::font_size::MED,
        family: egui::FontFamily::Name("inter_medium".into()),
    };
    let (tw, th) = get_text_size(ui, msg, font.clone()).into();

    let size = vec2(600.0, 175.0);
    modal(ui, size, "Denied permissions modal", |ui| {
        ui.set_min_size(size);
        let rect = ui
            .allocate_rect(
                Rect::from_min_size(
                    ui.viewport_rect().center() - vec2(size.x / 2.0, size.y / 2.0),
                    size,
                ),
                egui::Sense::focusable_noninteractive(),
            )
            .rect;

        let dark = ui.visuals().dark_mode;
        ui.painter().rect_filled(rect, SHARP, plt::BG(dark));
        ui.painter()
            .rect_stroke(rect, SHARP, border(dark), StrokeKind::Inside);

        let button_size = vec2(100.0, 25.0);
        custom_text(
            ui,
            msg,
            font,
            ui.viewport_rect().center() - vec2(0.0, th / 2.0 + button_size.y / 2.0),
            plt::letter_spacing::MINIMAL,
            plt::TEXT,
            Align::Center,
        );

        #[cfg(target_os = "macos")]
        {
            use crate::open_macos_privacy_settings;

            modal_button(
                ui,
                rect.center_bottom() - vec2(button_size.x / 2.0, button_size.y * 1.75),
                button_size,
                "Open Settings",
                plt::font_size::MED,
                plt::YELLO,
                &mut st.bool.open_macos_privacy_settings,
                false,
            );

            if st.bool.open_macos_privacy_settings {
                open_macos_privacy_settings();
                st.bool.open_macos_privacy_settings = false;
            }
        }
    });
}
