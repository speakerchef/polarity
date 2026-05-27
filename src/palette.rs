use bevy::prelude::Color;

pub const VOID: Color = Color::srgb_u8(0, 0, 0);
pub const BG: Color = Color::srgb_u8(0x0d, 0x0d, 0x0d);
pub const BG_MED: Color = Color::srgb_u8(0x0c, 0x0c, 0x0c);
pub const BG_DARK: Color = Color::srgb_u8(0x0a, 0x0a, 0x0a);
pub const SURFACE: Color = Color::srgb_u8(0x17, 0x17, 0x17);
pub const SURFACE_HOVER: Color = Color::srgb_u8(0x1c, 0x1c, 0x1c);
pub const BORDER: Color = Color::srgb_u8(0x3a, 0x3a, 0x3a);
pub const TRACK: Color = Color::srgb_u8(0x1A, 0x1A, 0x1A);

pub const DIM: Color = Color::srgb_u8(0x8a, 0x8a, 0x8a);
pub const TEXT: Color = Color::srgb_u8(0xCC, 0xCC, 0xCC);
pub const BRIGHT: Color = Color::srgb_u8(0xE8, 0xE8, 0xE8);

pub const LIVE: Color = Color::srgb_u8(0x5E, 0xC5, 0x7b);
pub const DANGER: Color = Color::srgb_u8(0xD9, 0x34, 0x2A);
pub const WARN: Color = Color::srgb_u8(0xFF, 0x8A, 0x4C);

/// Text-on-`TEXT` (e.g. active rowhead inverts to TEXT bg with this fg).
pub const INK: Color = Color::srgb_u8(0x11, 0x11, 0x11);

pub mod font_size {
    pub const TINY: f32 = 10.0;
    pub const META: f32 = 11.0;
    pub const BODY: f32 = 12.0;
    pub const MED: f32 = 13.0;
    pub const BIG: f32 = 14.0;
    pub const ICON: f32 = 16.0;
}

pub mod font_weight {
    pub const BODY: u16 = 400;
    pub const MED: u16 = 500;
    pub const HEAVY: u16 = 700;
}

pub mod letter_spacing {
    pub const BASE: f32 = 1.;
    pub const SPACED: f32 = 2.;
    pub const ULTRAWIDE: f32 = 3.;
}

pub mod spacing {
    pub const S1: f32 = 4.0;
    pub const S2: f32 = 6.0;
    pub const S3: f32 = 8.0;
    pub const S4: f32 = 10.0;
    pub const S5: f32 = 12.0;
    pub const S6: f32 = 16.0;
    pub const S7: f32 = 24.0;
}

pub mod width {
    pub const SMALL_SELECTOR_MENU: f32 = 100.;
    pub const MED_SELECTOR_MENU: f32 = 120.;
    pub const LARGE_SELECTOR_MENU: f32 = 240.;
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

pub mod motion {
    use std::time::Duration;
    pub const FAST: Duration = Duration::from_millis(80);
}

pub const FRAME_WIDTH: f32 = 1.0;
pub const APP_PADDING: f32 = 12.0;
