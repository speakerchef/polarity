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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Fluidwave {
    pub color_mode: ColorMode,
    pub energy_transfer_mode: EnergyTransferMode,
    pub force_direction: ForceDirection,
    pub gravity: f32,
    pub pressure_multiplier: f32,
    pub target_density: f32,
    pub smoothing_radius: f32,
    pub near_pressure_multiplier: f32,
    pub viscosity_strength: f32,
    pub point_size: f32,
    pub bloom: f32,
    pub uniform_color: crate::Rgba,

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
        Self {
            last_frame: Instant::now(),
            color_mode: ColorMode::VelocityGradient,
            uniform_color: crate::Rgba::new(255, 25, 255, 255),
            // Reactive
            // energy_transfer_mode: EnergyTransferMode::ForceField,
            // force_direction: ForceDirection::Out,
            // envelope_last_sample: 0.0,
            // gravity: 0.0,
            // pressure_multiplier: 290.0,
            // target_density: (NUM_PARTICLES as f32 * 86.0).round(),
            // smoothing_radius: 0.08,
            // near_pressure_multiplier: 10.0,
            // viscosity_strength: 0.022,
            // point_size: 0.006,
            // bloom: 0.0,
            energy_transfer_mode: EnergyTransferMode::ForceField,
            force_direction: ForceDirection::Out,
            envelope_last_sample: 0.0,
            gravity: 0.0,
            // pressure_multiplier: 250.0,
            pressure_multiplier: 180.0,
            target_density: (NUM_PARTICLES as f32 * 86.0).round(),
            smoothing_radius: 0.08,
            near_pressure_multiplier: 7.0,
            viscosity_strength: 0.05,
            point_size: 0.01,
            bloom: 0.0,
        }
    }
}
