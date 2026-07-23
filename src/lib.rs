#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod audio;
mod generators;
mod state;
mod traits;
mod ui;
mod wgpu_init;

use crate::{
    generators::{oscilloscope::Oscilloscope, polar_patterns::PolarPatterns},
    traits::Labeled,
};
pub use app::PolarityApp;

use crate::generators::{fluidwave::Fluidwave, stereometer::Stereometer};

#[macro_export]
macro_rules! labeled_enum {
    ($name:ident { $($variant:ident => $label:literal),+ $(,)?}, $def:ident) => {
        #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
        pub enum $name { $($variant),+ }
        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];
            pub fn label(self) -> &'static str {
                match self { $($name::$variant => $label),+ }
            }
        }
        impl Default for $name {
            fn default() -> Self { $name::$def }
        }
    };
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Preset {
    pub stereometer: Stereometer,
    pub fluidwave: Fluidwave,
    pub oscilloscope: Oscilloscope,
    pub polar_patterns: PolarPatterns,
}

#[derive(Default, Clone, Copy)]
pub struct LinearRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Rgba> for LinearRgba {
    fn from(value: Rgba) -> Self {
        LinearRgba {
            r: value.r / u8::MAX as f32,
            g: value.g / u8::MAX as f32,
            b: value.b / u8::MAX as f32,
            a: value.a / u8::MAX as f32,
        }
    }
}

impl LinearRgba {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_u8rgb(r: u8, g: u8, b: u8, a: u8) -> Self {
        let max = u8::MAX as f32;
        let (r, g, b, a) = (
            r as f32 / max,
            g as f32 / max,
            b as f32 / max,
            a as f32 / max,
        );
        Self { r, g, b, a }
    }

    pub fn as_u8(&self) -> (u8, u8, u8, u8) {
        let max = u8::MAX as f32;

        let (r, g, b, a) = (self.r * max, self.g * max, self.b * max, self.a * max);
        (r as u8, g as u8, b as u8, a as u8)
    }
}

labeled_enum!(GenKindLabel{
    Stereometer=> "Stereometer",
    Fluidwave => "Fluidwave",
    Oscilloscope => "Oscilloscope",
    PolarPatterns => "Polar Patterns"
}, PolarPatterns);

impl Labeled for GenKindLabel {
    fn text(self) -> &'static str {
        self.label()
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Default for Rgba {
    fn default() -> Self {
        Self {
            r: 255.0,
            g: 255.0,
            b: 255.0,
            a: 255.0,
        }
    }
}

impl Rgba {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        }
    }

    pub fn as_tuple(&self) -> (u8, u8, u8, u8) {
        (self.r as u8, self.g as u8, self.b as u8, self.a as u8)
    }

    /// Returns color normalized between 0 - 1
    pub fn normalized(&self) -> (f32, f32, f32, f32) {
        let max = u8::MAX as f32;
        (self.r / max, self.g / max, self.b / max, self.a / max)
    }
}
