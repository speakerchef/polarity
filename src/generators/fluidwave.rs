use std::time::Instant;

use eframe::egui;

use crate::{
    generators::{ChromaType, PostFx},
    labeled_enum,
    state::{BoolStates, MAX_BLOOM, MAX_CHROMA_SHIFT, MAX_VIGNETTE},
    traits::{ActiveGenerator, Generator, Labeled, PostFxParams},
    ui::{
        canvas::NUM_PARTICLES,
        control_panel_widgets::{
            dropdown_row, mod_slider_row, section_header_submenu, slider_row, static_label,
            subheader_toggle_button, toggle_button_row,
        },
    },
};

labeled_enum!(ColorMode {
    Uniform => "Uniform",
    VelocityGradient => "Velocity Gradient",
}, VelocityGradient);

impl Labeled for ColorMode {
    fn text(self) -> &'static str {
        self.label()
    }
}

labeled_enum!(EnergyTransferMode {
    ForceField =>"Force Field",
    Obstacle => "Obstacle"
}, ForceField);

impl Labeled for EnergyTransferMode {
    fn text(self) -> &'static str {
        self.label()
    }
}
labeled_enum!(ForceDirection {
    Out =>"Outward",
    In => "Inward"
}, Out);

impl Labeled for ForceDirection {
    fn text(self) -> &'static str {
        self.label()
    }
}

labeled_enum!(ColorArrangement {
    Rgb => "RGB",
    Grb => "GRB",
    Gbr => "GBR",
    Bgr => "BGR",
    Brg => "BRG",
    Rbg => "RBG",
}, Rgb);

impl Labeled for ColorArrangement {
    fn text(self) -> &'static str {
        self.label()
    }
}
impl ColorArrangement {
    pub fn to_value(self) -> u32 {
        match self {
            ColorArrangement::Rgb => 0,
            ColorArrangement::Grb => 1,
            ColorArrangement::Gbr => 2,
            ColorArrangement::Bgr => 3,
            ColorArrangement::Brg => 4,
            ColorArrangement::Rbg => 5,
        }
    }
}

labeled_enum!(ModSrc {
    None => "None",
    EnvA => "Envelope A",
    EnvB => "Envelope B"
}, None);

impl Labeled for ModSrc {
    fn text(self) -> &'static str {
        self.label()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Fluidwave {
    pub color_mode: ColorMode,
    pub color_invert: bool,
    pub sim_speed: f32,
    pub luminance_mode: bool,
    pub luminance_floor: f32,
    pub color_arrangement: ColorArrangement,
    pub energy_transfer_mode: EnergyTransferMode,
    pub force_direction: ForceDirection,
    pub gravity: f32,
    pub envelope_pressure_link: bool,
    pub pressure_multiplier: f32,
    pub target_density: f32,
    pub smoothing_radius: f32,
    pub edge_damping_factor: f32,
    pub near_pressure_multiplier: f32,
    pub viscosity_amount: f32,
    pub point_size: f32,
    pub uniform_color: crate::Rgba,
    pub env_range: f32,
    pub efx: PostFx,

    #[serde(skip, default = "instant_default")]
    pub last_frame: Instant,
    #[serde(skip)]
    pub frame_time_accumulator: f32,
    #[serde(skip)]
    pub last_idx: usize,
}

impl ActiveGenerator for Fluidwave {}
impl PostFxParams for Fluidwave {
    fn post_fx(&self) -> PostFx {
        self.efx
    }
}

fn instant_default() -> Instant {
    Instant::now()
}

impl Generator for Fluidwave {
    fn prepare(
        &mut self,
        _pl: &crate::audio::audio_player::AudioPlayer,
        _export_sample_idx: Option<usize>,
    ) {
    }

    fn draw_post_fx(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {
        let rect = section_header_submenu(ui, "BLOOM", &mut open.bloom_open).rect;
        subheader_toggle_button(ui, &rect, &mut self.efx.use_bloom);
        if open.bloom_open {
            slider_row(ui, "BLOOM", &mut self.efx.bloom, 0.0, MAX_BLOOM, 1);
        }
        let rect = section_header_submenu(ui, "VIGNETTE", &mut open.vignette_open).rect;
        subheader_toggle_button(ui, &rect, &mut self.efx.use_vignette);
        if open.vignette_open {
            slider_row(ui, "VIGNETTE", &mut self.efx.vignette, 0.0, MAX_VIGNETTE, 2);
        }
        let rect = section_header_submenu(ui, "CHROMA", &mut open.chroma_open).rect;
        subheader_toggle_button(ui, &rect, &mut self.efx.use_chroma);
        if open.chroma_open {
            mod_slider_row(
                ui,
                "CHROMA",
                &mut self.efx.chroma_shift,
                0.0,
                MAX_CHROMA_SHIFT,
                3,
                &mut self.efx.chroma_shift_mod_src,
                ModSrc::ALL,
                &mut open.chroma_mod_open,
                &mut open.mod_src_open,
                &mut self.efx.chroma_shift_range,
            );
            slider_row(ui, "BLUR", &mut self.efx.chroma_blur, 0.0, 20.0, 0);
            dropdown_row(
                ui,
                "TYPE",
                &mut self.efx.chroma_type,
                ChromaType::ALL,
                &mut open.chroma_type_open,
            );
        }
    }

    fn draw_visual_menu(&mut self, ui: &mut egui::Ui, st: &mut BoolStates) {
        slider_row(ui, "SIM SPEED", &mut self.sim_speed, 1.0, 200.0, 1);
        slider_row(ui, "POINT SIZE", &mut self.point_size, 0.0005, 0.02, 4);
        if st.advanced_mode {
            static_label(ui, "ADVANCED SETTINGS");
            slider_row(ui, "MAX FORCE", &mut self.env_range, 0.0, 100.0, 1);
            slider_row(ui, "VISCOSITY", &mut self.viscosity_amount, 0.0, 0.05, 3);
            slider_row(
                ui,
                "DENSITY",
                &mut self.target_density,
                0.0,
                (86.0 * NUM_PARTICLES as f32).round(),
                0,
            );
            toggle_button_row(
                ui,
                "ENVELOPE-PRESSURE LINK",
                &mut self.envelope_pressure_link,
            );
            slider_row(ui, "PRESSURE", &mut self.pressure_multiplier, 0.0, 400.0, 0);
            slider_row(ui, "DAMPING", &mut self.edge_damping_factor, 0.0, 1.0, 2);
            slider_row(ui, "F-BOUNDS", &mut self.smoothing_radius, 0.05, 0.25, 2);
        }
    }
}

impl Default for Fluidwave {
    fn default() -> Self {
        Self {
            sim_speed: 100.0,
            last_frame: Instant::now(),
            frame_time_accumulator: 0.0,
            color_mode: ColorMode::VelocityGradient,
            color_invert: false,
            luminance_mode: true,
            luminance_floor: 2.0,
            color_arrangement: ColorArrangement::default(),
            uniform_color: crate::Rgba::new(255, 25, 255, 255),
            last_idx: 0,
            energy_transfer_mode: EnergyTransferMode::ForceField,
            force_direction: ForceDirection::Out,
            gravity: 0.0,
            pressure_multiplier: 150.0,
            envelope_pressure_link: true,
            target_density: (NUM_PARTICLES as f32 * 78.57).round(),
            smoothing_radius: 0.10,
            edge_damping_factor: 0.45,
            near_pressure_multiplier: 7.0,
            viscosity_amount: 0.007,
            point_size: 0.0045,
            env_range: 80.0,
            efx: PostFx {
                bloom_mod_src: ModSrc::None,
                vignette_mod_src: ModSrc::None,
                chroma_shift_mod_src: ModSrc::EnvB,

                use_bloom: true,
                bloom: 0.5,
                use_vignette: true,
                vignette: 0.10,
                use_chroma: true,
                chroma_shift: 0.0,
                chroma_shift_range: 100.0,
                chroma_blur: 4.0,
                chroma_type: ChromaType::Radial,
                ..Default::default()
            },
        }
    }
}
