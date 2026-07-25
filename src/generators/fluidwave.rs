use std::time::Instant;

use eframe::egui;

use crate::{
    generators::{
        ChromaType, EnvelopeBank, FilterBank, FilterParams, MIN_SUBSTEP_DIV, PostFx, SUBSTEP_DIV,
        TARGET_DT,
        rendering::{FluidCbParams, GenCbParams},
    },
    labeled_enum,
    state::{AppState, BoolStates, DAMP_FACTOR},
    traits::{ActiveGenerator, Generator, Labeled, ParamAccess},
    ui::{
        canvas::NUM_PARTICLES,
        control_panel_widgets::{
            dropdown_row, mod_slider_row, section_header_submenu, slider_row, static_label,
            toggle_button_row,
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
    EnvB => "Envelope B",
    EnvC => "Envelope C",
    EnvD => "Envelope D"
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
    pub luminance_mode_mod_open: bool,
    pub luminance_floor_mod_src: ModSrc,
    pub luminance_floor_rng: f32,

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
impl ParamAccess for Fluidwave {
    fn post_fx(&self) -> PostFx {
        self.efx
    }
    fn post_fx_mut(&mut self) -> &mut PostFx {
        &mut self.efx
    }
    fn filter_params(&mut self) -> Option<&mut FilterParams> {
        None
    }
}

fn instant_default() -> Instant {
    Instant::now()
}

impl Generator for Fluidwave {
    fn prepare(
        &mut self,
        _f: &mut FilterBank,
        _env: &EnvelopeBank,
        _pl: &crate::audio::audio_player::AudioPlayer,
        _export_sample_idx: Option<usize>,
    ) {
    }
    fn get_gen_callback_params(
        &mut self,
        st: &AppState,
        live: bool,
        fps: usize,
    ) -> super::rendering::GenCbParams {
        const MAX_FRAME_TIME: f32 = 1. / 12. / SUBSTEP_DIV;
        const MAX_LUMINANCE_FLOOR: f32 = 100.0;

        let f = self;
        let env = |src: ModSrc, range: f32| st.env_bank.envelope_value_from_mod_src(src, range);
        let luminance_floor = f.luminance_floor
            + env(f.luminance_floor_mod_src, f.luminance_floor_rng) * MAX_LUMINANCE_FLOOR;

        let pressure_multiplier = f.pressure_multiplier
            - if f.envelope_pressure_link {
                400.0 * env(ModSrc::EnvA, f.env_range).powf(DAMP_FACTOR)
            } else {
                0.0
            };

        let sim_speed_scale = 100.0 / f.sim_speed.max(1.0);
        let sim_speed = (sim_speed_scale * SUBSTEP_DIV) /* higher == slower */
            .clamp(MIN_SUBSTEP_DIV, 100.0)
            .round();

        let now = Instant::now();
        let frame_time = if live {
            now.duration_since(f.last_frame).as_secs_f32() / sim_speed
        } else {
            1. / fps as f32 / sim_speed
        };

        f.frame_time_accumulator += frame_time.min(MAX_FRAME_TIME);

        let params = GenCbParams::Fwave(FluidCbParams {
            color_mode: f.color_mode,
            uniform_color: f.uniform_color,
            //fwave will use env A as its driver
            particle_pos: env(ModSrc::EnvA, f.env_range),
            frame_time_accumulator: f.frame_time_accumulator,
            gravity: f.gravity,
            pressure_multiplier,
            target_density: f.target_density,
            smoothing_radius: f.smoothing_radius,
            edge_damping_factor: f.edge_damping_factor,
            near_pressure_multiplier: f.near_pressure_multiplier,
            viscosity_amount: f.viscosity_amount,
            point_size: f.point_size,
            energy_transfer_mode: f.energy_transfer_mode,
            force_direction: f.force_direction,
            color_arrangement: f.color_arrangement,
            color_invert: f.color_invert,
            luminance_mode: f.luminance_mode,
            luminance_floor,
            substeps: sim_speed,
        });
        f.last_frame = now;
        f.frame_time_accumulator %= TARGET_DT; // leftover frametime
        params
    }

    fn draw_render_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {
        section_header_submenu(ui, "RENDER", &mut open.render_open);

        if open.render_open {
            dropdown_row(
                ui,
                "ENERGY TRANSFER",
                &mut self.energy_transfer_mode,
                EnergyTransferMode::ALL,
                &mut open.energy_transfer_mode_options_open,
                false,
            );
            if matches!(self.energy_transfer_mode, EnergyTransferMode::ForceField) {
                dropdown_row(
                    ui,
                    "FORCE DIRECTION",
                    &mut self.force_direction,
                    ForceDirection::ALL,
                    &mut open.force_direction_options_open,
                    false,
                );
            }
        }
    }

    fn draw_color_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {
        section_header_submenu(ui, "COLOR", &mut open.color_open);
        if open.color_open {
            dropdown_row(
                ui,
                "COLOR MODE",
                &mut self.color_mode,
                ColorMode::ALL,
                &mut open.color_mode_options_open,
                false,
            );
            match self.color_mode {
                ColorMode::VelocityGradient => {
                    dropdown_row(
                        ui,
                        "COLOR ORDER",
                        &mut self.color_arrangement,
                        ColorArrangement::ALL,
                        &mut open.color_arrangement_options_open,
                        false,
                    );
                    toggle_button_row(ui, "INVERT COLOR", &mut self.color_invert, false);
                    toggle_button_row(ui, "LUMINANCE MODE", &mut self.luminance_mode, false);
                    if self.luminance_mode {
                        mod_slider_row(
                            ui,
                            "LUM FLOOR",
                            &mut self.luminance_floor,
                            0.0,
                            100.0,
                            0,
                            &mut self.luminance_floor_mod_src,
                            &mut self.luminance_mode_mod_open,
                            &mut open.mod_src_open,
                            &mut self.luminance_floor_rng,
                            false,
                        );
                    }
                }
                ColorMode::Uniform => {
                    slider_row(ui, "RED", &mut self.uniform_color.r, 0.0, 255.0, 0, false);
                    slider_row(ui, "GREEN", &mut self.uniform_color.g, 0.0, 255.0, 0, false);
                    slider_row(ui, "BLUE", &mut self.uniform_color.b, 0.0, 255.0, 0, false);
                }
            }
        }
    }

    fn draw_visual_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {
        section_header_submenu(ui, "VISUAL", &mut open.visual_open);
        if open.visual_open {
            slider_row(ui, "SIM SPEED", &mut self.sim_speed, 1.0, 200.0, 1, false);
            slider_row(
                ui,
                "POINT SIZE",
                &mut self.point_size,
                0.0005,
                0.02,
                4,
                false,
            );
            if open.advanced_mode {
                static_label(ui, "ADVANCED SETTINGS");
                slider_row(ui, "MAX FORCE", &mut self.env_range, 0.0, 100.0, 1, false);
                slider_row(
                    ui,
                    "VISCOSITY",
                    &mut self.viscosity_amount,
                    0.0,
                    0.05,
                    3,
                    false,
                );
                slider_row(
                    ui,
                    "DENSITY",
                    &mut self.target_density,
                    0.0,
                    (86.0 * NUM_PARTICLES as f32).round(),
                    0,
                    false,
                );
                toggle_button_row(
                    ui,
                    "ENVELOPE-PRESSURE LINK",
                    &mut self.envelope_pressure_link,
                    false,
                );
                slider_row(
                    ui,
                    "PRESSURE",
                    &mut self.pressure_multiplier,
                    0.0,
                    400.0,
                    0,
                    false,
                );
                slider_row(
                    ui,
                    "DAMPING",
                    &mut self.edge_damping_factor,
                    0.0,
                    1.0,
                    2,
                    false,
                );
                slider_row(
                    ui,
                    "F-BOUNDS",
                    &mut self.smoothing_radius,
                    0.05,
                    0.25,
                    2,
                    false,
                );
            }
        }
    }
}

impl Default for Fluidwave {
    fn default() -> Self {
        Self {
            sim_speed: 120.0,
            last_frame: Instant::now(),
            frame_time_accumulator: 0.0,
            color_mode: ColorMode::VelocityGradient,
            color_invert: false,
            luminance_mode: true,

            luminance_floor: 5.0,
            luminance_mode_mod_open: false,
            luminance_floor_mod_src: ModSrc::None,
            luminance_floor_rng: 0.0,

            color_arrangement: ColorArrangement::Grb,
            uniform_color: crate::Rgba::new(255, 255, 255, 255),
            last_idx: 0,
            energy_transfer_mode: EnergyTransferMode::ForceField,
            force_direction: ForceDirection::Out,
            gravity: 0.0,
            pressure_multiplier: 150.0,
            envelope_pressure_link: true,
            target_density: (NUM_PARTICLES as f32 * 78.57).round(),
            // smoothing_radius: 0.10,
            smoothing_radius: 0.20,
            edge_damping_factor: 0.75,
            near_pressure_multiplier: 7.0,
            viscosity_amount: 0.007,
            // point_size: 0.0045,
            point_size: 0.0020,
            env_range: 77.0,
            efx: PostFx {
                bloom_mod_src: ModSrc::EnvC,
                vignette_mod_src: ModSrc::EnvC,
                chroma_shift_mod_src: ModSrc::EnvB,

                use_bloom: true,
                bloom: 3.0,
                bloom_range: 40.0,
                use_vignette: true,
                vignette: 0.20,
                vignette_range: -100.0,
                use_chroma: true,
                chroma_shift: 0.0,
                chroma_shift_range: 100.0,
                chroma_blur: 4.0,
                chroma_type: ChromaType::Radial,
            },
        }
    }
}
