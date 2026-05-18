pub mod goniometer;
pub mod palette;
use bevy::prelude::*;

pub const ANIM_SCALE_FACTOR: f32 = 250.;
// pub const WINDOW_SIZE: usize = 8192;
pub const WINDOW_SIZE: usize = 5632;
// pub const WINDOW_SIZE: usize = 5120;

#[derive(Component, Debug, Clone)]
pub struct PointArray(pub Vec<Entity>);

#[derive(Component)]
pub struct PlayingAudio;

#[derive(Component)]
pub struct TimelineScrubber(pub Option<std::time::Duration>);

#[derive(Component)]
pub struct DrawableCursor;

#[derive(Component)]
pub struct PreviewCanvas;

#[derive(Component, Debug)]
pub struct AudioFileContents {
    pub duration: f32,
    pub sample_rate: u32,
    pub num_channels: usize,
    pub samples: Vec<f32>,
}
