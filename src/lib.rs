pub mod palette;
pub mod stereometer;
pub mod ui;

use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

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

pub struct FontBlock {
    pub icon: FontSource,
    pub text: FontSource,
}
#[derive(Component, Clone)]
pub struct NullComponent;

#[derive(Component)]
pub struct DurationText;

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
pub struct TraceMesh;

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
pub enum TraceDensity {
    Off,
    Low,
    #[default]
    Med,
    High,
    Ultra,
}
impl TraceDensity {
    pub const COUNT: usize = 5;

    pub fn count(&self) -> usize {
        match self {
            TraceDensity::Off => 1,
            TraceDensity::Low => 10420,
            TraceDensity::Med => 15696,
            TraceDensity::High => 24576,
            TraceDensity::Ultra => 32768,
        }
    }

    pub fn all() -> &'static [Self; Self::COUNT] {
        &[
            TraceDensity::Off,
            TraceDensity::Low,
            TraceDensity::Med,
            TraceDensity::High,
            TraceDensity::Ultra,
        ]
    }
}

impl From<TraceDensity> for String {
    fn from(value: TraceDensity) -> Self {
        match value {
            TraceDensity::Off => "Off".to_string(),
            TraceDensity::Low => "Low".to_string(),
            TraceDensity::Med => "Med".to_string(),
            TraceDensity::High => "High".to_string(),
            TraceDensity::Ultra => "Ultra".to_string(),
        }
    }
}

const SHADER_ASSET_PATH: &str = "shaders/custom_material.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    pub alpha_mode: AlphaMode2d,
}

impl Material2d for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        self.alpha_mode
    }
}

#[derive(Resource, Default, Debug, PartialEq, Clone)]
pub enum FilteringMode {
    #[default]
    Off,
    Lpf,
    Bpf,
    Hpf,
}

impl From<FilteringMode> for String {
    fn from(value: FilteringMode) -> Self {
        match value {
            FilteringMode::Off => "Off".to_string(),
            FilteringMode::Lpf => "Low-Pass".to_string(),
            FilteringMode::Bpf => "Band-Pass".to_string(),
            FilteringMode::Hpf => "High-Pass".to_string(),
        }
    }
}
impl std::fmt::Display for FilteringMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilteringMode::Off => write!(f, "Off"),
            FilteringMode::Lpf => write!(f, "Low-Pass"),
            FilteringMode::Bpf => write!(f, "Band-Pass"),
            FilteringMode::Hpf => write!(f, "High-Pass"),
        }
    }
}
