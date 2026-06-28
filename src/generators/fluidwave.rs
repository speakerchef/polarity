use std::time::Instant;

use crate::{labeled_enum, state::Labeled, ui::canvas::NUM_PARTICLES};

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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Fluidwave {
    pub color_mode: ColorMode,
    pub energy_transfer_mode: EnergyTransferMode,
    pub force_direction: ForceDirection,
    pub gravity: f32,
    pub pressure_multiplier: f32,
    pub target_density: f32,
    pub smoothing_radius: f32,
    pub edge_damping_factor: f32,
    pub near_pressure_multiplier: f32,
    pub viscosity_amount: f32,
    pub point_size: f32,
    pub uniform_color: crate::Rgba,
    pub attack: f32,
    pub release: f32,
    pub range: f32,
    pub envelope_sensitivity: f32,
    pub bloom: f32,
    pub vignette: f32,

    #[serde(skip, default = "instant_default")]
    pub last_frame: Instant,
    #[serde(skip)]
    pub envelope_last_sample: f32,
}

fn instant_default() -> Instant {
    Instant::now()
}

impl Default for Fluidwave {
    fn default() -> Self {
        // Kinda chaotic
        // Self {
        //     last_frame: Instant::now(),
        //     color_mode: ColorMode::VelocityGradient,
        //     uniform_color: crate::Rgba::new(255, 25, 255, 255),
        //     attack: 0.01,
        //     release: 0.20,
        //     range: 85.0,
        //     envelope_sensitivity: 120.0,
        //     energy_transfer_mode: EnergyTransferMode::ForceField,
        //     force_direction: ForceDirection::Out,
        //     envelope_last_sample: 0.0,
        //     gravity: 0.0,
        //     pressure_multiplier: 180.0,
        //     target_density: (NUM_PARTICLES as f32 * 86.0).round(),
        //     smoothing_radius: 0.08,
        //     near_pressure_multiplier: 7.0,
        //     viscosity_strength: 0.002,
        //     point_size: 0.0045,
        //     bloom: 0.5,
        // }

        // damp = 0.75
        Self {
            last_frame: Instant::now(),
            color_mode: ColorMode::VelocityGradient,
            uniform_color: crate::Rgba::new(255, 25, 255, 255),
            attack: 0.10,
            release: 0.01,
            // range: 80.0,
            range: 60.0,
            envelope_sensitivity: 110.0,
            energy_transfer_mode: EnergyTransferMode::ForceField,
            force_direction: ForceDirection::Out,
            envelope_last_sample: 0.0,
            gravity: 0.0,
            // pressure_multiplier: 400.0,
            pressure_multiplier: 150.0,
            // target_density: (NUM_PARTICLES as f32 * 86.0).round(),
            target_density: (NUM_PARTICLES as f32 * 78.57).round(),
            smoothing_radius: 0.10,
            // edge_damping_factor: 0.25,
            edge_damping_factor: 0.45,
            near_pressure_multiplier: 7.0,
            viscosity_amount: 0.007,
            point_size: 0.0045,
            bloom: 0.5,
            vignette: 0.10,
        }
    }
}
