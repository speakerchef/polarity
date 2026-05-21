pub mod goniometer;
pub mod low_pass_filter;
pub mod palette;
use bevy::prelude::*;

pub const NUM_VERTICES: usize = 6;
pub const DOT_HALF_SIZE: f32 = 0.75;
pub const ANIM_SCALE_FACTOR: f32 = 250.0;
pub const RADIAL_SCALE_FACTOR: f32 = 0.4;
pub const LIVE_WINDOW_SIZE: usize = 2048;
// pub const HISTORY_WINDOW_SIZE: usize = 16328;
// pub const HISTORY_WINDOW_SIZE: usize = 42696;
pub const HISTORY_WINDOW_SIZE: usize = 0;
pub const LIVE_MAGENTA: LinearRgba = LinearRgba {
    red: 3.0,
    green: 0.5,
    blue: 2.1,
    alpha: 4.0,
};
pub const HISTORY_MAGENTA: LinearRgba = LinearRgba {
    red: 1.0,
    green: 0.1,
    blue: 0.7,
    alpha: 0.0,
};
pub const CRT_P1: LinearRgba = LinearRgba {
    red: 0.40,
    green: 2.00,
    blue: 0.40,
    alpha: 3.0,
};
pub const CRT_P7: LinearRgba = LinearRgba {
    red: 0.15,
    green: 0.50,
    blue: 0.35,
    alpha: 0.0,
};

#[derive(Component)]
pub struct PlayingAudio;

#[derive(Component)]
pub struct TimelineScrubber(pub Option<std::time::Duration>);

#[derive(Component)]
pub struct DrawableCursor;

#[derive(Component)]
pub struct PreviewCanvas;

#[derive(Component)]
pub struct LiveMesh;

#[derive(Component)]
pub struct HistoryMesh;

#[derive(Component, Debug)]
pub struct AudioFileContents {
    pub duration: f32,
    pub sample_rate: u32,
    pub num_channels: usize,
    pub samples: Vec<f32>,
}
