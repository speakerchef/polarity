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
pub enum LiveDensity {
    Low,
    Med,
    #[default]
    High,
    XHigh,
    Ultra,
    Extreme,
    PleaseDont,
}

impl LiveDensity {
    pub const COUNT: usize = 7;

    pub fn count(&self) -> usize {
        match self {
            LiveDensity::Low => 512,
            LiveDensity::Med => 1536,
            LiveDensity::High => 2048,
            LiveDensity::XHigh => 4096,
            LiveDensity::Ultra => 8192,
            LiveDensity::Extreme => 16384,
            LiveDensity::PleaseDont => 32768,
        }
    }

    pub fn all() -> &'static [Self; Self::COUNT] {
        &[
            LiveDensity::Low,
            LiveDensity::Med,
            LiveDensity::High,
            LiveDensity::XHigh,
            LiveDensity::Ultra,
            LiveDensity::Extreme,
            LiveDensity::PleaseDont,
        ]
    }
}

impl From<LiveDensity> for String {
    fn from(value: LiveDensity) -> Self {
        match value {
            LiveDensity::Low => "Low".to_string(),
            LiveDensity::Med => "Med".to_string(),
            LiveDensity::High => "High".to_string(),
            LiveDensity::XHigh => "XHigh".to_string(),
            LiveDensity::Ultra => "Ultra".to_string(),
            LiveDensity::Extreme => "Extreme".to_string(),
            LiveDensity::PleaseDont => "Please Dont".to_string(),
        }
    }
}

#[derive(Component, Default, Debug, Hash, Eq, PartialEq, Clone)]
pub enum HistoryDensity {
    Off,
    Low,
    #[default]
    Med,
    High,
    Ultra,
}
impl HistoryDensity {
    pub const COUNT: usize = 5;

    pub fn count(&self) -> usize {
        match self {
            HistoryDensity::Off => 1,
            HistoryDensity::Low => 10420,
            HistoryDensity::Med => 15696,
            HistoryDensity::High => 24576,
            HistoryDensity::Ultra => 32768,
        }
    }

    pub fn all() -> &'static [Self; Self::COUNT] {
        &[
            HistoryDensity::Off,
            HistoryDensity::Low,
            HistoryDensity::Med,
            HistoryDensity::High,
            HistoryDensity::Ultra,
        ]
    }
}

impl From<HistoryDensity> for String {
    fn from(value: HistoryDensity) -> Self {
        match value {
            HistoryDensity::Off => "Off".to_string(),
            HistoryDensity::Low => "Low".to_string(),
            HistoryDensity::Med => "Med".to_string(),
            HistoryDensity::High => "High".to_string(),
            HistoryDensity::Ultra => "Ultra".to_string(),
        }
    }
}
