#![allow(dead_code, unused_variables, non_snake_case)]
use eframe::egui::Color32;

pub const BLACK: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);

pub fn BG(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0x0d, 0x0d, 0x0d)
    } else {
        // Color32::from_rgb(0x1a, 0x1a, 0x1a)
        Color32::from_rgb(0x22, 0x22, 0x22) // #222222 #1f1f1f
    }
}

pub fn BG_MED(_dark: bool) -> Color32 {
    Color32::from_rgb(0x0c, 0x0c, 0x0c)
}

pub fn BG_DARK(_dark: bool) -> Color32 {
    Color32::from_rgb(0x0a, 0x0a, 0x0a)
}

pub fn VOID() -> Color32 {
    Color32::from_rgb(0x00, 0x00, 0x00)
}

pub fn SURFACE(_dark: bool) -> Color32 {
    Color32::from_rgb(0x15, 0x15, 0x15)
}

pub fn SURFACE_HOVER(_dark: bool) -> Color32 {
    Color32::from_rgb(0x25, 0x25, 0x25)
}

pub const GRAY: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);

pub fn WAVEFORM_BG(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0x3d, 0x3d, 0x3d)
    } else {
        Color32::from_rgb(0, 0, 0)
    }
}

pub fn BORDER(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0x3d, 0x3d, 0x3d)
    } else {
        // Color32::from_rgb(0x51, 0x51, 0x51)
        Color32::from_rgb(0x62, 0x62, 0x62)
    }
}

pub const DIM: Color32 = Color32::from_rgb(0x8a, 0x8a, 0x8a);
pub const TEXT: Color32 = Color32::from_rgb(0xcc, 0xcc, 0xcc);
pub const BRIGHT: Color32 = Color32::from_rgb(0xe8, 0xe8, 0xe8);
pub const LIVE: Color32 = Color32::from_rgb(0x4f, 0xda, 0x6e);
pub const GREEN: Color32 = Color32::from_rgb(72, 173, 102);
pub const YELLO: Color32 = Color32::from_rgb(255, 158, 51);
pub const DANGER: Color32 = Color32::from_rgb(0xd9, 0x34, 0x2a);
pub const WARN: Color32 = Color32::from_rgb(0xff, 0x8a, 0x4c);
pub const INK: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);

pub mod font_size {
    pub const TINY: f32 = 10.0;
    pub const META: f32 = 11.0;
    pub const BODY: f32 = 12.0;
    pub const MED: f32 = 13.0;
    pub const BIG: f32 = 14.0;
    pub const ICON: f32 = 16.0;
}

pub mod letter_spacing {
    pub const MINIMAL: f32 = 0.5;
    pub const BASE: f32 = 1.0;
    pub const SPACED: f32 = 2.0;
    pub const ULTRAWIDE: f32 = 3.0;
}

pub mod width {
    pub const SLIDER: f32 = 140.0;
}

pub mod height {
    pub const ROWHEAD: f32 = 44.0;
    pub const DROPDOWN_ITEM: f32 = 37.0;
    pub const MENU_ITEM: f32 = 40.0;
    pub const SLIDER_ROW_ITEM: f32 = 30.0;
    pub const SEG: f32 = 28.0;
    pub const INNER: f32 = 26.0;
    pub const TRANSPORT: f32 = 22.0;
    pub const TOGGLE: f32 = 14.0;
}

pub const FRAME_WIDTH: f32 = 1.0;
pub const APP_PADDING: f32 = 12.0;
