#![allow(dead_code, unused)]
use crate::{audio::audio_player::AudioPlayer, state::AppState};

use crate::ui::palette;

pub fn draw(ui: &mut egui::Ui, st: &mut AppState, pl: &Option<AudioPlayer>) {
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(palette::VOID))
        .show_inside(ui, |ui| {

            // let Some(p) = pl else {
            //     return;
            // };
            // let stereo_mesh = st.stereo.draw(
            //     p,
            //     pos2(ui.available_width() / 2.0, ui.available_height() / 2.0),
            // );
            // ui.painter().add(stereo_mesh);
        });
}
