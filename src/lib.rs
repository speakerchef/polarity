#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod audio;
mod generators;
mod state;
mod ui;

pub use app::PolarityApp;

#[derive(Default)]
pub struct LinearRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
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

#[derive(Default)]
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
