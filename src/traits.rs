use crate::audio::audio_player::AudioPlayer;

pub trait Labeled: PartialEq + Copy {
    fn text(self) -> &'static str;
}
pub trait Generator {
    fn prepare(&mut self, pl: &AudioPlayer, export_sample_idx: Option<usize>);
}
