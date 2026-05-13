//! Polarity design tokens — Bevy `Color` constants + sizes/fonts.
//!
//! Source: `scratch/polarity/v21-review/index.html` design audit.
//! Drop into your Bevy project at `src/ui/palette.rs` (or wherever).
//!
//! Bevy 0.14+ — `Color::srgb` and `Color::srgb_u8` are const-fn.
//! Older Bevy: replace `Color::srgb_u8(r,g,b)` with `Color::rgb_u8(r,g,b)`.

use bevy::prelude::Color;

// ----------------------------------------------------------------------------
// SURFACES (darkest → lightest)
// ----------------------------------------------------------------------------

/// Outside-app bg (viewport gutter around frame).
pub const VOID: Color = Color::srgb_u8(0x00, 0x00, 0x00);

/// Sunken cells: modal panels, color-strip wells, knob-val cells.
pub const DEEPEST: Color = Color::srgb_u8(0x0A, 0x0A, 0x0A);

/// Main app surface (workspace, mod-workspace, sec-body, right panel).
pub const PANEL: Color = Color::srgb_u8(0x0E, 0x0E, 0x0E);

/// Theme alias — raised cards if needed.
pub const BG: Color = Color::srgb_u8(0x11, 0x11, 0x14);

/// Header chrome (rowhead, secline, status-footer-pinned).
pub const CHROME: Color = Color::srgb_u8(0x17, 0x17, 0x17);

/// Raised pill / chip surface.
pub const RAISED: Color = Color::srgb_u8(0x1F, 0x1F, 0x1F);

/// Active-selection bg (expanded rowhead, active segmented button).
pub const INVERTED: Color = Color::srgb_u8(0xAA, 0xAA, 0xAA);

// ----------------------------------------------------------------------------
// BORDERS / DIVIDERS (subtle → strong)
// ----------------------------------------------------------------------------

/// Slider gutter, slider tracks.
pub const DIVIDER_SOFT: Color = Color::srgb_u8(0x1A, 0x1A, 0x1A);

/// Sec-body inner row separators.
pub const DIVIDER: Color = Color::srgb_u8(0x1F, 0x1F, 0x1F);

/// Primary container / control borders.
pub const BORDER: Color = Color::srgb_u8(0x2A, 0x2A, 0x2A);

/// App frame, focus rings, contrast borders.
pub const BORDER_STRONG: Color = Color::srgb_u8(0x3A, 0x3A, 0x3A);

// ----------------------------------------------------------------------------
// TEXT (dim → bright)
// ----------------------------------------------------------------------------

/// Disabled, tertiary captions.
pub const TEXT_FAINT: Color = Color::srgb_u8(0x3A, 0x3A, 0x3A);

/// Inactive labels, separators.
pub const TEXT_MUTE: Color = Color::srgb_u8(0x55, 0x55, 0x55);

/// Off-state glyphs, inactive state text.
pub const TEXT_DIM: Color = Color::srgb_u8(0x66, 0x66, 0x66);

/// Secondary labels, hints (#888).
pub const TEXT_MID: Color = Color::srgb_u8(0x88, 0x88, 0x88);

/// Secondary labels, time (#999).
pub const TEXT_MID_2: Color = Color::srgb_u8(0x99, 0x99, 0x99);

/// Default body, section title (#AAA).
pub const TEXT_BODY: Color = Color::srgb_u8(0xAA, 0xAA, 0xAA);

/// Section title alt (#B0B0B0).
pub const TEXT_BODY_2: Color = Color::srgb_u8(0xB0, 0xB0, 0xB0);

/// Standard control text.
pub const TEXT_STRONG: Color = Color::srgb_u8(0xCC, 0xCC, 0xCC);

/// Headings, brand labels, active glyphs.
pub const TEXT_HIGH: Color = Color::srgb_u8(0xE8, 0xE8, 0xE8);

/// Text on `INVERTED` surface (e.g. expanded rowhead).
pub const TEXT_ON_INVERTED: Color = Color::srgb_u8(0x11, 0x11, 0x11);

// ----------------------------------------------------------------------------
// ACCENTS
// ----------------------------------------------------------------------------

/// System "LIVE" indicator, play button. Reserved.
pub const ACCENT_LIVE: Color = Color::srgb_u8(0x7A, 0xE8, 0x9A);

/// Color preset chip / sample fill (cyan accent).
pub const ACCENT_CYAN: Color = Color::srgb_u8(0x5A, 0x8E, 0xAA);

/// Delete hover bg.
pub const DANGER: Color = Color::srgb_u8(0xD9, 0x34, 0x2A);

/// Delete hover ring.
pub const DANGER_EDGE: Color = Color::srgb_u8(0xFF, 0x5C, 0x50);

/// Danger menu item hover (orange focus-ring color).
pub const WARN: Color = Color::srgb_u8(0xFF, 0x8A, 0x4C);

// ----------------------------------------------------------------------------
// INTERACTION STATES
// ----------------------------------------------------------------------------

/// Rowhead hover bg (collapsed root header).
pub const HOVER_ROWHEAD: Color = Color::srgb_u8(0x2A, 0x2A, 0x2A);

/// Mod-row hover bg.
pub const HOVER_MOD_ROW: Color = Color::srgb_u8(0x25, 0x25, 0x25);

// ----------------------------------------------------------------------------
// SLIDER
// ----------------------------------------------------------------------------

pub const SLIDER_TRACK: Color = DIVIDER_SOFT; // #1A1A1A
pub const SLIDER_FILL: Color = TEXT_HIGH; // #E8E8E8
pub const SLIDER_THUMB: Color = INVERTED; // #AAA
pub const SLIDER_THUMB_HOVER: Color = TEXT_HIGH; // #E8
pub const SLIDER_THUMB_BORDER: Color = Color::srgb_u8(0x11, 0x11, 0x11);

// ----------------------------------------------------------------------------
// TYPOGRAPHY
// ----------------------------------------------------------------------------

pub mod font {
    pub const SANS: &str = "Geist";
    pub const MONO: &str = "SourceCodePro";
}

/// Type sizes (px → Bevy f32).
pub mod size {
    pub const BODY: f32 = 12.0; // default body / control text
    pub const META: f32 = 11.0; // section title, time, slider label
    pub const TINY: f32 = 10.0; // state label "active"/"inactive"
    pub const BRAND: f32 = 13.0; // rowhead label-cap
    pub const NUMBER: f32 = 9.0; // bypass-tok glyph
}

/// Font weights (Bevy fonts loaded per-weight as separate assets).
pub mod weight {
    pub const BODY: u16 = 440;
    pub const MED: u16 = 550;
    pub const HEAVY: u16 = 770;
}

/// Letter spacing (em values × font size to get px in Bevy).
pub mod tracking {
    pub const TIGHT: f32 = 0.02; // segmented button
    pub const NORMAL: f32 = 0.06; // state label
    pub const WIDE: f32 = 0.08; // slider label
    pub const WIDER: f32 = 0.10; // secline section title
    pub const WIDEST: f32 = 0.12; // rowhead brand label
}

// ----------------------------------------------------------------------------
// SPACING (4px base)
// ----------------------------------------------------------------------------

pub mod space {
    pub const S1: f32 = 4.0;
    pub const S2: f32 = 6.0;
    pub const S3: f32 = 8.0;
    pub const S4: f32 = 10.0;
    pub const S5: f32 = 12.0;
    pub const S6: f32 = 16.0;
    pub const S7: f32 = 24.0;
}

// ----------------------------------------------------------------------------
// COMPONENT HEIGHTS (px)
// ----------------------------------------------------------------------------

pub mod height {
    pub const ROWHEAD: f32 = 44.0; // GENERATOR / MODIFIERS root bar
    pub const MOD_ROW: f32 = 40.0; // individual modifier instance
    pub const SECLINE: f32 = 37.0; // subsection header
    pub const SLIDER_ROW: f32 = 30.0; // param row
    pub const SEG: f32 = 28.0; // segmented tab
    pub const INNER: f32 = 26.0; // number cell, color strip, picker
    pub const TRANSPORT: f32 = 22.0; // .tb thin transport button
    pub const BYPASS_TOK: f32 = 14.0; // state toggle click target
}

// ----------------------------------------------------------------------------
// MOTION
// ----------------------------------------------------------------------------

pub mod motion {
    use std::time::Duration;

    /// Fast UI transitions (hover, color flips).
    pub const FAST: Duration = Duration::from_millis(80);
}

// ----------------------------------------------------------------------------
// FRAME
// ----------------------------------------------------------------------------

/// App outer frame border width (px).
pub const FRAME_WIDTH: f32 = 1.0;

/// App outer body padding (px) — gap between viewport and frame.
pub const BODY_PADDING: f32 = 12.0;
