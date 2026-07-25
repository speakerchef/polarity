use eframe::egui;

use crate::{
    audio::audio_player::AudioPlayer,
    generators::{EnvelopeBank, FilterBank, FilterParams, PostFx, rendering::GenCbParams},
    state::{AppState, BoolStates},
};

pub trait Labeled: PartialEq + Copy {
    fn text(self) -> &'static str;
}

#[allow(unused)]
pub trait Generator {
    fn prepare(
        &mut self,
        filterbank: &mut FilterBank,
        env_bank: &EnvelopeBank,
        pl: &AudioPlayer,
        export_sample_idx: Option<usize>,
    );
    fn get_gen_callback_params(&mut self, st: &AppState, live: bool, fps: usize) -> GenCbParams;
    fn draw_render_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {}
    fn draw_filtering_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {}
    fn draw_color_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {}
    fn draw_visual_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {}
}
pub trait Textured {
    fn texture(&self) -> Option<&wgpu::Texture>;
    fn target_format(&self) -> wgpu::TextureFormat;
    fn set_texture(&mut self, tex: wgpu::Texture);
}
pub trait ParamAccess {
    fn post_fx(&self) -> PostFx;
    fn post_fx_mut(&mut self) -> &mut PostFx;
    fn filter_params(&mut self) -> Option<&mut FilterParams>;
}
pub trait ActiveGenerator: Generator + ParamAccess {}
