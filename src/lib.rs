#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod audio;
mod generators;
mod state;
mod ui;
mod wgpu_init;

use std::time::Instant;

pub use app::PolarityApp;
use eframe::egui::{Pos2, pos2};

use crate::{
    audio::audio_player::AudioPlayer,
    state::{AppState, Labeled},
};

#[macro_export]
macro_rules! labeled_enum {
    ($name:ident { $($variant:ident => $label:literal),+ $(,)?}, $def:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug,serde::Serialize, serde::Deserialize)]
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

labeled_enum!(GeneratorKind {
    Stereometer => "Stereometer",
    Fluidwave => "Fluidwave"
}, Stereometer);

impl Labeled for GeneratorKind {
    fn text(self) -> &'static str {
        self.label()
    }
}

#[derive(Default, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
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

pub fn points_to_quad_vertices(s: f32, l: f32, r: f32) -> [Pos2; 6] {
    [
        pos2(l + s, r + s),
        pos2(l + s, r - s),
        pos2(l - s, r - s),
        pos2(l + s, r + s),
        pos2(l - s, r + s),
        pos2(l - s, r - s),
    ]
}

pub fn envelope_follower(pl: &AudioPlayer, st: &mut AppState, live: bool, fps: usize) {
    let num_channels = pl.contents.num_channels as usize;
    const ATT: f32 = 0.75;
    const REL: f32 = 0.90;

    let start = if live {
        (pl.position()
            .saturating_sub(Instant::now().duration_since(st.fwave.last_frame))
            .as_secs_f64()
            * pl.contents.sample_rate as f64) as usize
    } else {
        st.export_sample_idx
    };
    let end = if live {
        (pl.position().as_secs_f64() * pl.contents.sample_rate as f64) as usize
    } else {
        start + (1. / fps as f64 * pl.contents.sample_rate as f64) as usize
    };
    let ef_window = &pl.contents.samples[start * num_channels..end * num_channels];
    let mut ls = st.fwave.envelope_last_sample;

    for s in ef_window.chunks_exact(2) {
        let l = s.first().unwrap_or(&0.0);
        let abs = l.abs();
        if abs > ls {
            ls = ls * ATT + (1.0 - ATT) * abs;
        } else {
            ls = ls * REL + (1.0 - REL) * abs;
        }
    }
    st.fwave.envelope_last_sample = ls;
}
