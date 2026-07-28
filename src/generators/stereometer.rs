use crate::generators::fluidwave::ModSrc;
use crate::generators::rendering::{GenCbParams, Particle2DCbParams};
use crate::generators::{
    ChromaType, EnvelopeBank, FilterBank, FilterParams, MAX_POINT_SIZE, MIN_POINT_SIZE, PostFx,
    radial_scale,
};
use crate::state::{AppState, BoolStates};
use crate::traits::{ActiveGenerator, AudioSrc, Generator, Labeled, ParamAccess};
use crate::ui::control_panel_widgets::{
    dropdown_row, mod_slider_row, section_header_submenu, slider_row, static_label,
};
use crate::{Rgba, labeled_enum};
use eframe::egui::{self, Pos2, pos2};
use std::collections::VecDeque;

enum FilterBand {
    Low,
    Mid,
    High,
}

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
    fn text(&self) -> &'static str {
        self.label()
    }
}
impl Labeled for FilterMode {
    fn text(&self) -> &'static str {
        self.label()
    }
}
impl Labeled for StereometerKind {
    fn text(&self) -> &'static str {
        self.label()
    }
}

impl Labeled for LiveDensity {
    fn text(&self) -> &'static str {
        self.label()
    }
}

impl Labeled for TraceDensity {
    fn text(&self) -> &'static str {
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
    kind: StereometerKind,
    render_mode: ParticleRenderMode,

    live_density: LiveDensity,
    trace_density: TraceDensity,

    filter_params: FilterParams,

    fs_color: Rgba,
    mb_color: [Rgba; 3],

    point_size: f32,
    point_size_mod_src: ModSrc,
    point_size_mod_open: bool,
    point_size_rng: f32,

    radial_scale_factor: f32,

    last_sample_idx: usize,
    efx: PostFx,

    #[serde(skip)]
    live_buffer: Vec<Pos2>,
    #[serde(skip)]
    live_low_buffer: Vec<Pos2>,
    #[serde(skip)]
    live_mid_buffer: Vec<Pos2>,
    #[serde(skip)]
    live_high_buffer: Vec<Pos2>,
    #[serde(skip)]
    trace_buffer: VecDeque<Pos2>,
    #[serde(skip)]
    trace_low_buffer: VecDeque<Pos2>,
    #[serde(skip)]
    trace_mid_buffer: VecDeque<Pos2>,
    #[serde(skip)]
    trace_high_buffer: VecDeque<Pos2>,
}

impl Default for Stereometer {
    fn default() -> Self {
        // synthwave preset
        Self {
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
            filter_params: FilterParams {
                filter_freq: 1.0,
                last_freq: 0.0,
                filter_mode: FilterMode::Off,
            },
            live_buffer: Default::default(),
            live_low_buffer: Default::default(),
            live_mid_buffer: Default::default(),
            live_high_buffer: Default::default(),
            last_sample_idx: Default::default(),
            trace_buffer: Default::default(),
            trace_low_buffer: Default::default(),
            trace_mid_buffer: Default::default(),
            trace_high_buffer: Default::default(),

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
        }
    }
}

impl Stereometer {
    pub fn draw(
        &mut self,
        f: &mut FilterBank,
        input: &dyn AudioSrc,
        export_sample_idx: Option<usize>,
    ) {
        let num_ch = input.num_channels() as usize;
        let sr = input.sample_rate();
        let s = input.audio_buffer();
        let gap = self.live_density.count() + 1;

        let start_idx = export_sample_idx.unwrap_or(if input.is_live() {
            (s.len() / num_ch).saturating_sub(gap)
        } else {
            (input.position().as_secs_f32() * sr as f32) as usize
        });
        let end_idx = if input.is_live() {
            s.len() / num_ch
        } else {
            start_idx + gap
        };

        let mut last_idx = self.last_sample_idx;

        if input.is_live() {
            /* correct for shifted indices from buffer resizing */
            last_idx = last_idx.saturating_sub(input.popped_sample_count() / num_ch);
        }

        if last_idx > start_idx {
            last_idx = start_idx;
            self.clear_trace_buffers();
        }

        let mut is_live = true;
        let live_window = input
            .audio_buffer()
            .get(start_idx * num_ch..end_idx * num_ch)
            .unwrap_or_default();

        self.clear_live_buffers();
        live_window.chunks_exact(2).for_each(|s| {
            let l = s.first().unwrap();
            let r = s.last().unwrap_or(l);
            self.set_positions(f, is_live, *l, *r);
        });

        is_live = false;
        let trace_window = input
            .audio_buffer()
            .get(last_idx * num_ch..start_idx * num_ch)
            .unwrap_or_default();

        trace_window.chunks_exact(2).for_each(|frame| {
            let l = frame[0];
            let r = *frame.last().unwrap_or(&l);
            self.set_positions(f, is_live, l, r);
        });
        self.limit_trace_buffers();
        self.last_sample_idx = start_idx;
    }

    fn filter_fs(&mut self, f: &mut FilterBank, is_live: bool, l: f32, r: f32) -> (f32, f32) {
        let fp = &mut self.filter_params;
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
        Some(&mut self.filter_params)
    }
}

impl Generator for Stereometer {
    fn prepare(
        &mut self,
        f: &mut FilterBank,
        _env: &EnvelopeBank,
        input: &dyn AudioSrc,
        export_sample_idx: Option<usize>,
    ) {
        self.draw(f, input, export_sample_idx);
    }

    fn get_gen_callback_params(&mut self, st: &AppState, _live: bool, _fps: usize) -> GenCbParams {
        let s = self;
        let env = |src: ModSrc, range: f32| st.env_bank.envelope_value_from_mod_src(src, range);
        let point_size = (s.point_size
            + env(s.point_size_mod_src, s.point_size_rng) * MAX_POINT_SIZE)
            .clamp(MIN_POINT_SIZE, MAX_POINT_SIZE);

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
                let fp = &mut self.filter_params;
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
                fp.filter_freq = fp.filter_freq.round();
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
