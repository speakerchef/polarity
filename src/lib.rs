pub mod palette;
pub mod stereometer;

use bevy::prelude::*;

pub const NUM_VERTICES: usize = 6;
pub const DOT_HALF_SIZE: f32 = 0.75;
pub const ANIM_SCALE_FACTOR: f32 = 250.0;
pub const RADIAL_SCALE_FACTOR: f32 = 0.4;
pub const MAX_WINDOW_SIZE: usize = 32768;
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
    pub duration: f64,
    pub sample_rate: u32,
    pub num_channels: usize,
    pub samples: Vec<f32>,
}

#[derive(Component, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum PointDensity {
    Low,
    Med,
    #[default]
    High,
    XHigh,
    Ultra,
    Extreme,
    PleaseDont,
}

impl PointDensity {
    pub const COUNT: usize = 7;

    pub fn count(&self) -> usize {
        match self {
            PointDensity::Low => 512,
            PointDensity::Med => 1536,
            PointDensity::High => 2048,
            PointDensity::XHigh => 4096,
            PointDensity::Ultra => 8192,
            PointDensity::Extreme => 16384,
            PointDensity::PleaseDont => 32768,
        }
    }

    pub fn all() -> &'static [PointDensity; Self::COUNT] {
        &[
            PointDensity::Low,
            PointDensity::Med,
            PointDensity::High,
            PointDensity::XHigh,
            PointDensity::Ultra,
            PointDensity::Extreme,
            PointDensity::PleaseDont,
        ]
    }
}

impl From<PointDensity> for String {
    fn from(value: PointDensity) -> Self {
        match value {
            PointDensity::Low => "Low".to_string(),
            PointDensity::Med => "Med".to_string(),
            PointDensity::High => "High".to_string(),
            PointDensity::XHigh => "XHigh".to_string(),
            PointDensity::Ultra => "Ultra".to_string(),
            PointDensity::Extreme => "Extreme".to_string(),
            PointDensity::PleaseDont => "Please Dont".to_string(),
        }
    }
}
