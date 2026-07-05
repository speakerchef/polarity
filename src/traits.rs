use eframe::egui;

use crate::{audio::audio_player::AudioPlayer, generators::PostFx, state::BoolStates};

pub trait Labeled: PartialEq + Copy {
    fn text(self) -> &'static str;
}
pub trait Generator {
    fn prepare(&mut self, pl: &AudioPlayer, export_sample_idx: Option<usize>);
    fn draw_post_fx(&mut self, ui: &mut egui::Ui, bool: &mut BoolStates);
    fn draw_visual_menu(&mut self, ui: &mut egui::Ui, bool: &mut BoolStates);
}
pub trait Textured {
    fn texture(&self) -> Option<&wgpu::Texture>;
    fn target_format(&self) -> wgpu::TextureFormat;
    fn set_texture(&mut self, tex: wgpu::Texture);
}
pub trait PostFxParams {
    fn post_fx(&self) -> PostFx;
}
pub trait ActiveGenerator: Generator + PostFxParams {}
