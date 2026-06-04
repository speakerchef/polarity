#![allow(unused_variables, dead_code)]
macro_rules! labeled_enum {
    ($name:ident { $($variant:ident => $label:literal),+ $(,)?}, $def:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
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

labeled_enum!(StereometerKind {
    LinearBipolar  => "Linear Bipolar",
    ScaledBipolar  => "Scaled Bipolar",
    LinearLissajous => "Linear Lissajous",
    ScaledLissajous => "Scaled Lissajous",
}, ScaledLissajous);

labeled_enum!(RenderMode {
    FullSpectrum => "Full Spectrum",
    MultiBand    => "Multi-Band",
}, MultiBand);

labeled_enum!(FilterMode {
    Off => "Off",
    Lpf => "Lpf",
    Bpf => "Bpf", 
    Hpf => "Hpf",
}, Off);

pub trait Labeled: Copy + PartialEq {
    fn text(self) -> &'static str;
}
impl Labeled for crate::state::RenderMode {
    fn text(self) -> &'static str {
        self.label()
    }
}
impl Labeled for crate::state::FilterMode {
    fn text(self) -> &'static str {
        self.label()
    }
}
impl Labeled for crate::state::StereometerKind {
    fn text(self) -> &'static str {
        self.label()
    }
}
pub type Hsl = (f32, f32, f32);

pub struct PanelState {
    pub render_mode: RenderMode,
    pub filter_mode: FilterMode,
    pub stereo_kind: StereometerKind,

    pub gen_open: bool,
    pub render_open: bool,
    pub render_mode_options_open: bool,
    pub stereo_kind_options_open: bool,
    pub filtering_open: bool,
    pub filter_mode_options_open: bool,
    pub mode_open: bool,
    pub color_open: bool,
    pub visual_open: bool,

    pub postfx_open: bool,
    pub sparkle_open: bool,

    pub filter_freq: f32,
    pub hsl_color_bands: [Hsl; 3],
    pub bloom: f32,
    pub file_name: String,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            render_mode: RenderMode::default(),
            filter_mode: FilterMode::default(),
            stereo_kind: StereometerKind::default(),
            gen_open: false,
            render_open: false,
            render_mode_options_open: false,
            stereo_kind_options_open: false,
            filtering_open: false,
            filter_mode_options_open: false,
            mode_open: false,
            color_open: false,
            visual_open: false,

            postfx_open: false,
            sparkle_open: false,
            filter_freq: 1.0,
            hsl_color_bands: [(0.0, 1.0, 0.50), (120.0, 1.0, 0.50), (240.0, 1.0, 0.50)],
            bloom: 0.4,
            file_name: "—".into(),
        }
    }
}
