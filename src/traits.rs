use std::time::Duration;

use eframe::egui;

use crate::{
    generators::{EnvelopeBank, FilterBank, FilterParams, PostFx, rendering::GenCbParams},
    state::{AppState, BoolStates},
};

pub trait Labeled: PartialEq + Clone {
    fn text(&self) -> &str;
}

#[allow(unused)]
pub trait Generator {
    fn prepare(
        &mut self,
        filterbank: &mut FilterBank,
        env_bank: &EnvelopeBank,
        input: &dyn AudioSrc,
        export_sample_idx: Option<usize>,
    );
    fn get_gen_callback_params(&mut self, st: &AppState, live: bool, fps: usize) -> GenCbParams;
    fn draw_render_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {}
    fn draw_filtering_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {}
    fn draw_color_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {}
    fn draw_visual_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {}
}
pub trait AudioSrc {
    fn is_live(&self) -> bool;
    fn sample_rate(&self) -> u32;
    fn num_channels(&self) -> u16;
    fn audio_buffer(&self) -> &[f32];
    fn position(&self) -> Duration;
    fn peak_level(&mut self, export_sample_idx: Option<usize>) -> (f32, f32);

    fn set_volume(&mut self, volume: f32);

    /// Number of samples removed from the circular buffer per frame
    fn popped_sample_count(&self) -> usize;
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
