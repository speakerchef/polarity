#![warn(clippy::all, rust_2018_idioms)]
#![allow(dead_code)]
mod app;
mod audio;
mod generators;
mod state;
mod traits;
mod ui;
mod wgpu_init;

use crate::generators::{
    GenKind, cymatic_field::CymaticField, oscilloscope::Oscilloscope, polar_patterns::PolarPatterns,
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

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn polarity_audio_capture_permission() -> i32;
}

#[derive(Debug, PartialEq, Eq)]
pub enum AudioCapturePermission {
    Granted,
    Denied,
    Unknown,
}

#[cfg(target_os = "macos")]
pub fn get_audio_capture_permission() -> AudioCapturePermission {
    match unsafe { polarity_audio_capture_permission() } {
        0 => AudioCapturePermission::Granted,
        1 => AudioCapturePermission::Denied,
        _ => AudioCapturePermission::Unknown,
    }
}
#[cfg(not(target_os = "macos"))]
pub fn get_audio_capture_permission() -> AudioCapturePermission {
    AudioCapturePermission::Unknown
}

labeled_enum!(HardwareEncoder {
   VideoToolbox =>  "h264_videotoolbox",
   Nvenc =>  "h264_nvenc",
   Amf =>  "h264_amf",
   Qsv =>  "h264_qsv",
   Vaapi =>  "h264_vaapi",
}, VideoToolbox);

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Preset {
    pub gen_kind: GenKind,
    pub stereometer: Stereometer,
    pub fluidwave: Fluidwave,
    pub oscilloscope: Oscilloscope,
    pub polar_patterns: PolarPatterns,
    pub cymatics: CymaticField,
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
    const WHITE: Self = Rgba {
        r: 255.,
        g: 255.,
        b: 255.,
        a: 255.,
    };
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

#[cfg(target_os = "macos")]
pub fn open_macos_privacy_settings() {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSString, NSURL};
    use std::{ffi::CString, ptr::NonNull};

    let app_query =
        CString::new("x-apple.systempreferences:com.apple.preference.security").unwrap();
    let app_query = NonNull::new(app_query.into_raw()).unwrap();
    let privacy_settings_url =
        unsafe { NSURL::URLWithString(&NSString::stringWithUTF8String(app_query).unwrap()) }
            .unwrap();
    NSWorkspace::sharedWorkspace().openURL(&privacy_settings_url);
}
