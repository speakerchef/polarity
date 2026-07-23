use crate::generators::fluidwave::ModSrc;
use crate::generators::rendering::{GenCbParams, Particle2DCbParams};
use crate::generators::{ChromaType, FilterBank, FilterParams, PostFx, radial_scale};
use crate::state::{AppState, BoolStates};
use crate::traits::{ActiveGenerator, Generator, Labeled, ParamAccess};
use crate::ui::control_panel_widgets::{
    dropdown_row, mod_slider_row, section_header_submenu, slider_row, static_label,
};
use crate::{Rgba, labeled_enum};
use eframe::egui::{self, Pos2, pos2};
use std::collections::VecDeque;

use crate::audio::audio_player::AudioPlayer;
labeled_enum!(StereometerKind {
    LinearBipolar  => "Linear Bipolar",
    ScaledBipolar  => "Scaled Bipolar",
    LinearLissajous => "Linear Lissajous",
    ScaledLissajous => "Scaled Lissajous",
}, ScaledLissajous);

labeled_enum!(ParticleRenderMode {
    FullSpectrum => "Full Spectrum",
    MultiBand    => "Multi-Band",
}, MultiBand);

labeled_enum!(FilterMode {
    Off => "Off",
    Lpf => "Lpf",
    Bpf => "Bpf",
    Hpf => "Hpf",
}, Off);

labeled_enum!(LiveDensity {
    Low => "Low",
    Med => "Med",
    High => "High",
    Ultra => "Ultra",
    Extreme => "Extreme",
    PleaseDont => "Please Dont",
}, Ultra);

labeled_enum!(TraceDensity {
    Off => "Off",
    Low => "Low",
    Med => "Med",
    High => "High",
    Max => "Max",
}, Med);

impl LiveDensity {
    pub fn count(self) -> usize {
        match self {
            Self::Low => 512,
            Self::Med => 1536,
            Self::High => 2048,
            Self::Ultra => 4096,
            Self::Extreme => 8192,
            Self::PleaseDont => 16384,
        }
    }
}

impl TraceDensity {
    pub fn count(self) -> usize {
        match self {
            Self::Off => 1,
            Self::Low => 10420,
            Self::Med => 15696,
            Self::High => 24576,
            Self::Max => 32768,
        }
    }
}

impl Labeled for ParticleRenderMode {
    fn text(self) -> &'static str {
        self.label()
    }
}
impl Labeled for FilterMode {
    fn text(self) -> &'static str {
        self.label()
    }
}
impl Labeled for StereometerKind {
    fn text(self) -> &'static str {
        self.label()
    }
}

impl Labeled for LiveDensity {
    fn text(self) -> &'static str {
        self.label()
    }
}

impl Labeled for TraceDensity {
    fn text(self) -> &'static str {
        self.label()
    }
}

pub const MAX_LIVE_POINT_DENSITY: usize = 16384;
pub const MAX_TRACE_POINT_DENSITY: usize = 32768;
pub const VERTICES_PER_QUAD: usize = 6;
const SQRT_3: f32 = 1.7320508;
const LINEAR_BIPOLAR_SF: f32 = 0.5;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Stereometer {
    pub kind: StereometerKind,
    pub render_mode: ParticleRenderMode,

    pub live_density: LiveDensity,
    pub trace_density: TraceDensity,

    pub filter_params: Option<FilterParams>,

    pub fs_color: Rgba,
    pub mb_color: [Rgba; 3],

    pub point_size: f32,
    pub point_size_mod_src: ModSrc,
    pub point_size_mod_open: bool,
    pub point_size_rng: f32,

    pub radial_scale_factor: f32,

    pub last_sample_idx: usize,
    pub efx: PostFx,

    #[serde(skip)]
    pub live_buffer: Vec<Pos2>,
    #[serde(skip)]
    pub live_low_buffer: Vec<Pos2>,
    #[serde(skip)]
    pub live_mid_buffer: Vec<Pos2>,
    #[serde(skip)]
    pub live_high_buffer: Vec<Pos2>,
    #[serde(skip)]
    pub trace_buffer: VecDeque<Pos2>,
    #[serde(skip)]
    pub trace_low_buffer: VecDeque<Pos2>,
    #[serde(skip)]
    pub trace_mid_buffer: VecDeque<Pos2>,
    #[serde(skip)]
    pub trace_high_buffer: VecDeque<Pos2>,
}

impl ActiveGenerator for Stereometer {}
impl ParamAccess for Stereometer {
    fn post_fx(&self) -> PostFx {
        self.efx
    }
    fn post_fx_mut(&mut self) -> &mut PostFx {
        &mut self.efx
    }
    fn filter_params(&mut self) -> Option<&mut FilterParams> {
        self.filter_params.as_mut()
    }
}

impl Generator for Stereometer {
    fn prepare(&mut self, f: &mut FilterBank, pl: &AudioPlayer, export_sample_idx: Option<usize>) {
        self.draw(f, pl, export_sample_idx);
    }

    fn into_gen_callback_params(&mut self, st: &AppState, _live: bool, _fps: usize) -> GenCbParams {
        const MAX_POINT_SIZE: f32 = 0.01;
        let s = self;
        let env = |src: ModSrc, range: f32| st.envelope_value_from_mod_src(src, range);
        let point_size =
            s.point_size + env(s.point_size_mod_src, s.point_size_rng) * MAX_POINT_SIZE;

        GenCbParams::Particle2D(Particle2DCbParams {
            render_mode: s.render_mode,
            point_size,
            add_point_border: true,
            live_pos: std::mem::take(&mut s.live_buffer),
            trace_pos: s.trace_buffer.clone(),

            live_low: std::mem::take(&mut s.live_low_buffer),
            live_mid: std::mem::take(&mut s.live_mid_buffer),
            live_high: std::mem::take(&mut s.live_high_buffer),
            trace_low: s.trace_low_buffer.clone(),
            trace_mid: s.trace_mid_buffer.clone(),
            trace_high: s.trace_high_buffer.clone(),

            fs_color: s.fs_color.into(),
            lb_color: s.mb_color[0].into(),
            mb_color: s.mb_color[1].into(),
            hb_color: s.mb_color[2].into(),
        })
    }

    fn draw_render_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {
        section_header_submenu(ui, "RENDER", &mut open.render_open);
        if open.render_open {
            dropdown_row(
                ui,
                "MODE",
                &mut self.render_mode,
                ParticleRenderMode::ALL,
                &mut open.render_mode_options_open,
                false,
            );
            dropdown_row(
                ui,
                "STYLE",
                &mut self.kind,
                StereometerKind::ALL,
                &mut open.stereo_kind_options_open,
                false,
            );
        }
    }

    fn draw_filtering_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {
        if matches!(self.render_mode, ParticleRenderMode::FullSpectrum) {
            section_header_submenu(ui, "FILTERING", &mut open.filtering_open);
            if matches!(self.render_mode, ParticleRenderMode::FullSpectrum) && open.filtering_open {
                let fp = self.filter_params.as_mut().expect("created at constructor");
                dropdown_row(
                    ui,
                    "FILTER",
                    &mut fp.filter_mode,
                    FilterMode::ALL,
                    &mut open.filter_mode_options_open,
                    false,
                );
                if open.set_default_freqs {
                    let f = match fp.filter_mode {
                        FilterMode::Off => 1.0,
                        FilterMode::Lpf => 200.,
                        FilterMode::Bpf => 1000.,
                        FilterMode::Hpf => 5000.,
                    };
                    fp.filter_freq = f;
                    fp.last_freq = f;
                }
                slider_row(ui, "FREQ", &mut fp.filter_freq, 1.0, 20000.0, 0, false);
            }
        }
    }

    fn draw_color_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {
        section_header_submenu(ui, "COLOR", &mut open.color_open);
        if open.color_open {
            match self.render_mode {
                ParticleRenderMode::FullSpectrum => {
                    slider_row(ui, "RED", &mut self.fs_color.r, 0.0, 255.0, 0, false);
                    slider_row(ui, "GREEN", &mut self.fs_color.g, 0.0, 255.0, 0, false);
                    slider_row(ui, "BLUE", &mut self.fs_color.b, 0.0, 255.0, 0, false);
                }
                ParticleRenderMode::MultiBand => {
                    for (band, name) in ["LOW BAND", "MID BAND", "HIGH BAND"].iter().enumerate() {
                        static_label(ui, name);
                        slider_row(ui, "RED", &mut self.mb_color[band].r, 0.0, 255.0, 0, false);
                        slider_row(
                            ui,
                            "GREEN",
                            &mut self.mb_color[band].g,
                            0.0,
                            255.0,
                            0,
                            false,
                        );
                        slider_row(ui, "BLUE", &mut self.mb_color[band].b, 0.0, 255.0, 0, false);
                    }
                }
            }
        }
    }

    fn draw_visual_menu(&mut self, ui: &mut egui::Ui, open: &mut BoolStates) {
        section_header_submenu(ui, "VISUAL", &mut open.visual_open);
        if open.visual_open {
            dropdown_row(
                ui,
                "DENSITY",
                &mut self.live_density,
                LiveDensity::ALL,
                &mut open.density_open,
                false,
            );
            dropdown_row(
                ui,
                "TRACE",
                &mut self.trace_density,
                TraceDensity::ALL,
                &mut open.trace_open,
                false,
            );
            if matches!(
                self.kind,
                StereometerKind::ScaledBipolar | StereometerKind::ScaledLissajous
            ) {
                slider_row(
                    ui,
                    "RADIUS",
                    &mut self.radial_scale_factor,
                    0.0,
                    1.0,
                    3,
                    false,
                );
            }
            mod_slider_row(
                ui,
                "POINT SIZE",
                &mut self.point_size,
                0.0005,
                0.01,
                4,
                &mut self.point_size_mod_src,
                &mut self.point_size_mod_open,
                &mut open.mod_src_open,
                &mut self.point_size_rng,
                false,
            );
        }
    }
}

impl Default for Stereometer {
    fn default() -> Self {
        // synthwave preset
        Self {
            efx: PostFx {
                bloom_mod_src: ModSrc::EnvB,
                vignette_mod_src: ModSrc::EnvC,
                chroma_shift_mod_src: ModSrc::EnvB,

                use_bloom: true,
                bloom: 3.0,
                use_vignette: true,
                vignette: 0.0,
                use_chroma: true,
                chroma_shift: 0.0,
                chroma_shift_range: 50.0,
                chroma_blur: 4.0,
                chroma_type: ChromaType::Radial,

                ..Default::default()
            },
            radial_scale_factor: 0.6,
            fs_color: Rgba::new(0, 255, 170, 255),
            mb_color: [
                Rgba::new(110, 0, 255, 255),
                Rgba::new(0, 155, 255, 255),
                Rgba::new(230, 0, 140, 255),
            ],
            point_size: 0.0025,
            point_size_mod_src: ModSrc::None,
            point_size_mod_open: false,
            point_size_rng: 0.0,
            kind: StereometerKind::default(),
            render_mode: ParticleRenderMode::MultiBand,
            live_density: LiveDensity::Ultra,
            trace_density: TraceDensity::High,
            filter_params: Some(FilterParams {
                filter_freq: 1.0,
                last_freq: 1.0,
                filter_mode: FilterMode::Off,
            }),
            live_buffer: Default::default(),
            live_low_buffer: Default::default(),
            live_mid_buffer: Default::default(),
            live_high_buffer: Default::default(),
            last_sample_idx: Default::default(),
            trace_buffer: Default::default(),
            trace_low_buffer: Default::default(),
            trace_mid_buffer: Default::default(),
            trace_high_buffer: Default::default(),
        }
    }
}

enum FilterBand {
    Low,
    Mid,
    High,
}

impl Stereometer {
    fn filter_fs(&mut self, f: &mut FilterBank, is_live: bool, l: f32, r: f32) -> (f32, f32) {
        let fp = &mut self.filter_params.expect("safe");
        if is_live {
            if let Some(live_fs) = &mut f.live_fs_filters {
                match fp.filter_mode {
                    FilterMode::Off => (l, r),
                    FilterMode::Lpf => live_fs.0.run(l, r),
                    FilterMode::Bpf => live_fs.1.run(l, r),
                    FilterMode::Hpf => live_fs.2.run(l, r),
                }
            } else {
                (l, r)
            }
        } else {
            if let Some(trace_fs) = &mut f.trace_fs_filters {
                match fp.filter_mode {
                    FilterMode::Off => (l, r),
                    FilterMode::Lpf => trace_fs.0.run(l, r),
                    FilterMode::Bpf => trace_fs.1.run(l, r),
                    FilterMode::Hpf => trace_fs.2.run(l, r),
                }
            } else {
                (l, r)
            }
        }
    }

    fn filter_mb(
        &mut self,
        f: &mut FilterBank,
        is_live: bool,
        band: FilterBand,
        l: f32,
        r: f32,
    ) -> (f32, f32) {
        if is_live {
            if let Some(live) = &mut f.live_mb_filters {
                match band {
                    FilterBand::Low => live.0.run(l, r),
                    FilterBand::Mid => live.1.run(l, r),
                    FilterBand::High => live.2.run(l, r),
                }
            } else {
                (l, r)
            }
        } else {
            if let Some(trace) = &mut f.trace_mb_filters {
                match band {
                    FilterBand::Low => trace.0.run(l, r),
                    FilterBand::Mid => trace.1.run(l, r),
                    FilterBand::High => trace.2.run(l, r),
                }
            } else {
                (l, r)
            }
        }
    }

    fn get_coord_from_meterkind(&self, l: f32, r: f32) -> (f32, f32) {
        let res = match self.kind {
            StereometerKind::LinearBipolar => {
                ((l - r) * LINEAR_BIPOLAR_SF, (l + r) * LINEAR_BIPOLAR_SF)
            }
            StereometerKind::ScaledBipolar => {
                let rscale = radial_scale(self.radial_scale_factor, l, r);
                ((l - r) * rscale / SQRT_3, (l + r) * rscale / SQRT_3)
            }
            StereometerKind::LinearLissajous => (l, r),
            StereometerKind::ScaledLissajous => {
                let rscale = radial_scale(self.radial_scale_factor, l, r);
                (l * rscale, r * rscale)
            }
        };
        (res.0.min(1.0), res.1.min(1.0))
    }

    fn set_positions(&mut self, f: &mut FilterBank, is_live: bool, l: f32, r: f32) {
        match self.render_mode {
            ParticleRenderMode::FullSpectrum => {
                let (l, r) = self.filter_fs(f, is_live, l, r);
                let (l, r) = self.get_coord_from_meterkind(l, r);
                if is_live {
                    self.live_buffer.push(pos2(l, r));
                } else {
                    self.trace_buffer.push_back(pos2(l, r));
                }
            }
            ParticleRenderMode::MultiBand => {
                let (lowl, lowr) = self.filter_mb(f, is_live, FilterBand::Low, l, r);
                let (midl, midr) = self.filter_mb(f, is_live, FilterBand::Mid, l, r);
                let (highl, highr) = self.filter_mb(f, is_live, FilterBand::High, l, r);
                let (lowl, lowr) = self.get_coord_from_meterkind(lowl, lowr);
                let (midl, midr) = self.get_coord_from_meterkind(midl, midr);
                let (highl, highr) = self.get_coord_from_meterkind(highl, highr);
                if is_live {
                    self.live_low_buffer.push(pos2(lowl, lowr));
                    self.live_mid_buffer.push(pos2(midl, midr));
                    self.live_high_buffer.push(pos2(highl, highr));
                } else {
                    self.trace_low_buffer.push_back(pos2(lowl, lowr));
                    self.trace_mid_buffer.push_back(pos2(midl, midr));
                    self.trace_high_buffer.push_back(pos2(highl, highr));
                }
            }
        }
    }

    fn limit_trace_buffers(&mut self) {
        let cap = self.trace_density.count();
        while self.trace_buffer.len() > cap {
            self.trace_buffer.pop_front();
        }
        while self.trace_low_buffer.len() > cap {
            self.trace_low_buffer.pop_front();
        }
        while self.trace_mid_buffer.len() > cap {
            self.trace_mid_buffer.pop_front();
        }
        while self.trace_high_buffer.len() > cap {
            self.trace_high_buffer.pop_front();
        }
    }

    pub fn clear_live_buffers(&mut self) {
        self.live_buffer.clear();
        self.live_low_buffer.clear();
        self.live_mid_buffer.clear();
        self.live_high_buffer.clear();
    }
    pub fn clear_trace_buffers(&mut self) {
        self.trace_buffer.clear();
        self.trace_low_buffer.clear();
        self.trace_mid_buffer.clear();
        self.trace_high_buffer.clear();
    }

    pub fn draw(&mut self, f: &mut FilterBank, p: &AudioPlayer, export_sample_idx: Option<usize>) {
        let num_ch = p.contents.num_channels as usize;

        let sample_pos = p.position().as_secs_f64();
        let sample_idx =
            export_sample_idx.unwrap_or((sample_pos * p.contents.sample_rate as f64) as usize);
        let last_idx = self.last_sample_idx;
        if sample_idx < last_idx {
            self.trace_buffer.clear();
        }

        let mut is_live = true;
        let live_window = p
            .contents
            .samples
            .get(sample_idx * num_ch..(sample_idx + self.live_density.count() + 1) * num_ch)
            .unwrap_or_default();

        self.clear_live_buffers();
        live_window.chunks_exact(2).for_each(|s| {
            let l = s.first().unwrap();
            let r = s.last().unwrap_or(l);
            self.set_positions(f, is_live, *l, *r);
        });

        is_live = false;
        let trace_window = p
            .contents
            .samples
            .get(last_idx * num_ch..sample_idx * num_ch)
            .unwrap_or_default();

        trace_window.chunks_exact(2).for_each(|s| {
            let l = s.first().unwrap();
            let r = s.last().unwrap_or(l);
            self.set_positions(f, is_live, *l, *r);
        });
        self.limit_trace_buffers();
        self.last_sample_idx = sample_idx;
    }
}
