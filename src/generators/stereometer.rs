use crate::{Rgba, audio::StereoFilter, labeled_enum, state::Labeled};
use eframe::egui::{Pos2, pos2};
use std::collections::VecDeque;

use crate::audio::audio_player::AudioPlayer;
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

impl Labeled for RenderMode {
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Stereometer {
    pub kind: StereometerKind,
    pub render_mode: RenderMode,

    pub live_density: LiveDensity,
    pub trace_density: TraceDensity,

    pub filter_mode: FilterMode,
    pub filter_freq: f32,
    pub last_freq: f32,

    pub fs_color: Rgba,
    pub mb_color: [Rgba; 3],

    pub bloom: f32,
    pub point_size: f32,

    pub last_sample_idx: usize,

    #[serde(skip)]
    pub live_fs_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    #[serde(skip)]
    pub trace_fs_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    #[serde(skip)]
    pub live_mb_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    #[serde(skip)]
    pub trace_mb_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,

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

impl Default for Stereometer {
    fn default() -> Self {
        // default preset
        let fstr = std::fs::read_to_string("presets/synthwave.json").unwrap();
        serde_json::from_str(&fstr).unwrap()
    }
}

enum FilterBand {
    Low,
    Mid,
    High,
}

impl Stereometer {
    fn points_to_quad_vertices(&self, l: f32, r: f32) -> [Pos2; 6] {
        let s = self.point_size;
        [
            pos2(l + s, r + s),
            pos2(l + s, r - s),
            pos2(l - s, r - s),
            pos2(l + s, r + s),
            pos2(l - s, r + s),
            pos2(l - s, r - s),
        ]
    }
    fn filter_fs(&mut self, is_live: bool, l: f32, r: f32) -> (f32, f32) {
        if is_live {
            if let Some(live_fs) = &mut self.live_fs_filters {
                match self.filter_mode {
                    FilterMode::Off => (l, r),
                    FilterMode::Lpf => live_fs.0.run(l, r),
                    FilterMode::Bpf => live_fs.1.run(l, r),
                    FilterMode::Hpf => live_fs.2.run(l, r),
                }
            } else {
                (l, r)
            }
        } else {
            if let Some(trace_fs) = &mut self.trace_fs_filters {
                match self.filter_mode {
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

    fn filter_mb(&mut self, is_live: bool, band: FilterBand, l: f32, r: f32) -> (f32, f32) {
        if is_live {
            if let Some(live) = &mut self.live_mb_filters {
                match band {
                    FilterBand::Low => live.0.run(l, r),
                    FilterBand::Mid => live.1.run(l, r),
                    FilterBand::High => live.2.run(l, r),
                }
            } else {
                (l, r)
            }
        } else {
            if let Some(trace) = &mut self.trace_mb_filters {
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

    fn radial_scale(&self, x: f32, y: f32) -> f32 {
        let sf = match self.render_mode {
            RenderMode::FullSpectrum => 0.325,
            RenderMode::MultiBand => 0.275,
        };
        let mag = (x * x + y * y).sqrt();
        let scaled = mag.powf(sf);
        if mag > 1e-6 { scaled / mag } else { 0.0 }
    }

    fn get_coord_from_meterkind(&self, l: f32, r: f32) -> (f32, f32) {
        let res = match self.kind {
            StereometerKind::LinearBipolar => {
                ((l - r) * LINEAR_BIPOLAR_SF, (l + r) * LINEAR_BIPOLAR_SF)
            }
            StereometerKind::ScaledBipolar => {
                let rscale = self.radial_scale(l, r);
                ((l - r) * rscale / SQRT_3, (l + r) * rscale / SQRT_3)
            }
            StereometerKind::LinearLissajous => (l, r),
            StereometerKind::ScaledLissajous => {
                let rscale = self.radial_scale(l, r)
                    * if self.render_mode == RenderMode::MultiBand {
                        1.1
                    } else {
                        1.0
                    };
                (l * rscale, r * rscale)
            }
        };
        (res.0.min(1.0), res.1.min(1.0))
    }

    fn set_positions(&mut self, is_live: bool, l: f32, r: f32) {
        match self.render_mode {
            RenderMode::FullSpectrum => {
                let (l, r) = self.filter_fs(is_live, l, r);
                let (l, r) = self.get_coord_from_meterkind(l, r);
                let pos = self.points_to_quad_vertices(l, r);
                if is_live {
                    self.live_buffer.extend(pos);
                } else {
                    self.trace_buffer.extend(pos);
                }
            }
            RenderMode::MultiBand => {
                let (lowl, lowr) = self.filter_mb(is_live, FilterBand::Low, l, r);
                let (midl, midr) = self.filter_mb(is_live, FilterBand::Mid, l, r);
                let (highl, highr) = self.filter_mb(is_live, FilterBand::High, l, r);
                let (lowl, lowr) = self.get_coord_from_meterkind(lowl, lowr);
                let (midl, midr) = self.get_coord_from_meterkind(midl, midr);
                let (highl, highr) = self.get_coord_from_meterkind(highl, highr);
                let posl = self.points_to_quad_vertices(lowl, lowr);
                let posm = self.points_to_quad_vertices(midl, midr);
                let posh = self.points_to_quad_vertices(highl, highr);
                if is_live {
                    self.live_low_buffer.extend(posl);
                    self.live_mid_buffer.extend(posm);
                    self.live_high_buffer.extend(posh);
                } else {
                    self.trace_low_buffer.extend(posl);
                    self.trace_mid_buffer.extend(posm);
                    self.trace_high_buffer.extend(posh);
                }
            }
        }
    }

    fn limit_trace_buffers(&mut self) {
        let cap = self.trace_density.count() * VERTICES_PER_QUAD;
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

    pub fn draw(&mut self, p: &AudioPlayer, export_sample_idx: Option<usize>) {
        let num_channels = p.contents.num_channels as usize;

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
            .get(sample_idx * num_channels..sample_idx * num_channels + self.live_density.count())
            .unwrap_or_default();

        self.clear_live_buffers();
        live_window.chunks_exact(2).for_each(|s| {
            let l = s.first().unwrap();
            let r = s.last().unwrap_or(l);
            self.set_positions(is_live, *l, *r);
        });

        is_live = false;
        let trace_window = p
            .contents
            .samples
            .get(last_idx * num_channels..sample_idx * num_channels)
            .unwrap_or_default();

        trace_window.chunks_exact(2).for_each(|s| {
            let l = s.first().unwrap();
            let r = s.last().unwrap_or(l);
            self.set_positions(is_live, *l, *r);
        });
        self.limit_trace_buffers();
        self.last_sample_idx = sample_idx;
    }
}
