use crate::{audio::audio_player::AudioPlayer, generators::ChromaType};

pub trait Labeled: PartialEq + Copy {
    fn text(self) -> &'static str;
}
pub trait Generator {
    fn prepare(&mut self, pl: &AudioPlayer, export_sample_idx: Option<usize>);
}
pub trait Textured {
    fn texture(&self) -> Option<&wgpu::Texture>;
    fn target_format(&self) -> wgpu::TextureFormat;
    fn set_texture(&mut self, tex: wgpu::Texture);
}
pub trait PostFxParams {
    fn post_fx(&self) -> (f32, f32, f32, ChromaType, f32);
    fn post_fx_state(&self) -> (bool, bool, bool);
}
pub trait ActiveGenerator: Generator + PostFxParams {}
