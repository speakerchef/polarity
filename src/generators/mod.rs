use std::sync::Arc;

use biquad::*;
use eframe::egui::{Pos2, pos2};
use rustfft::{Fft, FftPlanner, num_complex::Complex};

use crate::{
    audio::StereoFilter,
    generators::{fluidwave::ModSrc, stereometer::FilterMode},
    labeled_enum,
    traits::{AudioSrc, Labeled},
};

pub mod cymatic_field;
pub mod fluidwave;
pub mod oscilloscope;
pub mod polar_patterns;
pub mod rendering;
pub mod stereometer;

pub const TARGET_FPS: f32 = 30.0;
pub const SUBSTEP_DIV: f32 = 6.0;
pub const MIN_SUBSTEP_DIV: f32 = 3.0;
pub const TARGET_DT: f32 = 1. / TARGET_FPS / SUBSTEP_DIV;
const MAX_POINT_SIZE: f32 = 0.01;
const MIN_POINT_SIZE: f32 = 0.0005;

labeled_enum!(GenKind{
    PolarPatterns => "Polar Motion",
    Oscilloscope => "Oscillations",
    Stereometer=> "Stereometer",
    Fluidwave => "Fluidwave",
    CymaticField => "Cymatic Field",
}, PolarPatterns);

impl Labeled for GenKind {
    fn text(&self) -> &'static str {
        self.label()
    }
}

labeled_enum!(ChromaType {
    Linear => "Linear",
    Radial => "Radial",
}, Radial);
impl ChromaType {
    pub fn value(self) -> u32 {
        match self {
            ChromaType::Linear => 0,
            ChromaType::Radial => 1,
        }
    }
}
impl Labeled for ChromaType {
    fn text(&self) -> &'static str {
        self.label()
    }
}

labeled_enum!(FftWindow {
    W8192 => "8192",
    W16384 => "16384",
}, W8192);

impl Labeled for FftWindow {
    fn text(&self) -> &'static str {
        self.label()
    }
}

impl FftWindow {
    pub fn value(&self) -> usize {
        match self {
            Self::W8192 => 8192,
            Self::W16384 => 16384,
        }
    }
}

pub struct FftPass {
    pub fft: Arc<dyn Fft<f32>>,
}
impl Default for FftPass {
    fn default() -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FftWindow::default().value());
        FftPass { fft }
    }
}
impl Clone for FftPass {
    fn clone(&self) -> Self {
        Self::default()
    }
}

pub fn radial_scale(scale_factor: f32, x: f32, y: f32) -> f32 {
    let mag = (x * x + y * y).sqrt();
    let scaled = mag.powf(1.0 - scale_factor);
    if mag > 1e-6 { scaled / mag } else { 0.0 }
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FilterParams {
    pub filter_mode: FilterMode,
    pub last_freq: f32,
    pub filter_freq: f32,
}
#[derive(Default)]
pub struct FilterBank {
    pub live_fs_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    pub trace_fs_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    pub live_mb_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    pub trace_mb_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
}
#[derive(Default)]
pub struct EnvelopeBank {
    pub env_a: Option<Envelope>,
    pub env_b: Option<Envelope>,
    pub env_c: Option<Envelope>,
    pub env_d: Option<Envelope>,
}

impl EnvelopeBank {
    pub fn run_follower(&mut self, i: &dyn AudioSrc, export_sample_idx: Option<usize>) {
        let (Some(env_a), Some(env_b), Some(env_c), Some(env_d)) = (
            &mut self.env_a,
            &mut self.env_b,
            &mut self.env_c,
            &mut self.env_d,
        ) else {
            return;
        };
        env_a.run_differential_follower(i, export_sample_idx);
        env_b.run_differential_follower(i, export_sample_idx);
        env_c.run_differential_follower(i, export_sample_idx);
        env_d.run_differential_follower(i, export_sample_idx);
    }
    pub fn envelope_value_from_mod_src(&self, src: ModSrc, range: f32) -> f32 {
        let (Some(a), Some(b), Some(c), Some(d)) =
            (&self.env_a, &self.env_b, &self.env_c, &self.env_d)
        else {
            return 0.0;
        };
        match src {
            ModSrc::None => 0.0,
            ModSrc::EnvA => a.generator_envelope(range),
            ModSrc::EnvB => b.generator_envelope(range),
            ModSrc::EnvC => c.generator_envelope(range),
            ModSrc::EnvD => d.generator_envelope(range),
        }
    }
    pub fn get_envelope(&self, src: ModSrc) -> Option<&Envelope> {
        match src {
            ModSrc::None => None,
            ModSrc::EnvA => self.env_a.as_ref(),
            ModSrc::EnvB => self.env_b.as_ref(),
            ModSrc::EnvC => self.env_c.as_ref(),
            ModSrc::EnvD => self.env_d.as_ref(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Default)]
pub struct PostFx {
    //states
    pub use_bloom: bool,
    pub use_vignette: bool,
    pub use_chroma: bool,

    //mod
    pub bloom_mod_src: ModSrc,
    pub bloom_range: f32,
    pub vignette_mod_src: ModSrc,
    pub vignette_range: f32,
    pub chroma_shift_mod_src: ModSrc,
    pub chroma_shift_range: f32,

    //params
    pub bloom: f32,
    pub vignette: f32,
    pub chroma_shift: f32,
    pub chroma_blur: f32,
    pub chroma_type: ChromaType,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Envelope {
    pub attack: f32,
    pub release: f32,
    pub sensitivity: f32,

    #[serde(skip)]
    pub det_last_idx: usize,
    #[serde(skip)]
    cur_envelope: f32,
    #[serde(skip)]
    fast_envelope: f32,
    #[serde(skip)]
    slow_envelope: f32,

    #[serde(skip)]
    abs_last_idx: usize,
    #[serde(skip)]
    abs_env: f32,

    #[serde(skip)]
    hpf: Option<StereoFilter>,
}

impl Envelope {
    pub fn new(attack: f32, release: f32, sensitivity: f32, sample_rate: u32) -> Self {
        const HP_FREQ: f32 = 40.0;
        Self {
            attack,
            release,
            sensitivity,
            hpf: Some(StereoFilter::from_coeffs_butterworth(
                Type::HighPass,
                HP_FREQ,
                sample_rate,
            )),
            ..Default::default()
        }
    }

    /// Used to trigger generator effects
    pub fn generator_envelope(&self, range: f32) -> f32 {
        let env = self.cur_envelope;
        let s = range / 100.0;
        let compress = |mu: f32| -> f32 { ((1.0 + mu * env).ln() / (1.0 + mu).ln()) * s };
        let expand = |mu: f32| -> f32 { (((1.0 + mu).powf(env) - 1.0) / mu) * s };

        /* audio taper kinda */
        let raw = self.sensitivity;
        let sens = raw.abs();
        let mu = 10.0 + 0.05 * 300_f32.powf(sens);
        let base = compress(10.0);
        if env >= 1e-3 {
            if sens < 0.01 {
                base
            } else {
                if raw > 0. {
                    compress(mu)
                } else {
                    expand(mu - 10.0)
                }
            }
        } else {
            env * s
        }
    }

    /// Used for metering and such
    pub fn abs_envelope(&self) -> f32 {
        self.abs_env
    }

    pub fn run_differential_follower(
        &mut self,
        i: &dyn AudioSrc,
        export_sample_idx: Option<usize>,
    ) {
        let num_ch = i.num_channels() as usize;
        let sample_rate = i.sample_rate() as f32;
        let s = i.audio_buffer();

        const ENVELOPE_WINDOW: usize = 1024;
        let mut last_idx = self.det_last_idx;
        let start_idx = export_sample_idx.unwrap_or_else(|| {
            if i.is_live() {
                (s.len() / num_ch).saturating_sub(ENVELOPE_WINDOW)
            } else {
                (i.position().as_secs_f32() * sample_rate) as usize
            }
        });
        let end_idx = if i.is_live() {
            s.len() / num_ch
        } else {
            start_idx
        };
        if last_idx > start_idx {
            last_idx = start_idx;
        }

        let window = s
            .get(if i.is_live() { start_idx } else { last_idx } * num_ch..end_idx * num_ch)
            .unwrap_or_default();

        // DET params
        const FAST_ATT: f32 = 0.001;
        const SLOW_ATT: f32 = 0.200;
        const PEAK_FACTOR: f32
        /* r = FAST / SLOW
         * factor = r^(r / (1 - r)) - r^(1 / (1 - r))
         * cool property from the homogeneity to factor out shape from delta.
         */ = 0.96885797;

        let fast_coeff = (-1.0 / (FAST_ATT * sample_rate)).exp();
        let slow_coeff = (-1.0 / (SLOW_ATT * sample_rate)).exp();

        let att = self.attack / 1000.0;
        let att_coeff = (-1.0 / (att * sample_rate)).exp();
        let rel = self.release / 1000.;
        let rel_coeff = (-1.0 / (rel * sample_rate)).exp();

        let fast = &mut self.fast_envelope;
        let slow = &mut self.slow_envelope;
        let mut frame_max = self.cur_envelope;
        for frame in window.chunks_exact(i.num_channels() as usize) {
            let (l, r) = (frame[0], *frame.last().unwrap_or(&0.0));
            let (l, r) = self
                .hpf
                .as_mut()
                .expect("unreachable without filter")
                .run(l, r);
            let abs = (l.abs() + r.abs()) / 2.0;
            *fast = *fast * fast_coeff + abs * (1.0 - fast_coeff);
            *slow = *slow * slow_coeff + abs * (1.0 - slow_coeff);

            let delta = (*fast - *slow).max(0.0) / PEAK_FACTOR;

            if delta > frame_max {
                frame_max = frame_max * att_coeff + delta * (1.0 - att_coeff)
            } else {
                frame_max *= rel_coeff
            };
        }
        self.cur_envelope = frame_max;
        self.det_last_idx = start_idx;
    }

    pub fn run_absolute_follower(
        &mut self,
        i: &dyn AudioSrc,
        left: bool,
        export_sample_idx: Option<usize>,
    ) {
        let num_ch = i.num_channels() as usize;
        let sample_rate = i.sample_rate() as f32;
        let s = i.audio_buffer();

        const ENVELOPE_WINDOW: usize = 1024;
        let mut last_idx = self.abs_last_idx;
        let start_idx = export_sample_idx.unwrap_or_else(|| {
            if i.is_live() {
                (s.len() / num_ch).saturating_sub(ENVELOPE_WINDOW)
            } else {
                (i.position().as_secs_f32() * sample_rate) as usize
            }
        });
        let end_idx = if i.is_live() {
            s.len() / num_ch
        } else {
            start_idx
        };
        if last_idx > start_idx {
            last_idx = start_idx;
        }

        let window = s
            .get(if i.is_live() { start_idx } else { last_idx } * num_ch..end_idx * num_ch)
            .unwrap_or_default();

        let rel_s = self.release / 1000.0;
        let rel_coeff = (-1.0 / (rel_s * sample_rate)).exp();

        let mut level = self.abs_env;
        window.chunks_exact(num_ch).for_each(|frame| {
            let (l, r) = (frame[0], *frame.last().unwrap_or(&frame[0]));
            let src = if left { l.abs() } else { r.abs() };
            level = src.max(level * rel_coeff + (1.0 - rel_coeff) * src);
        });

        self.abs_env = level;
        self.abs_last_idx = start_idx;
    }
}

fn positional_interp_upsampling(src: &[Pos2], dst: &mut Vec<Pos2>, factor: usize) {
    let Some(mut prev) = src.first() else {
        return;
    };
    dst.reserve(1 + src.len().saturating_sub(1).saturating_mul(factor));
    dst.push(*prev);
    for pos in src {
        let dx = pos.x - prev.x;
        let dy = pos.y - prev.y;
        let ix = dx / factor as f32;
        let iy = dy / factor as f32;

        let (mut cx, mut cy) = (prev.x, prev.y);
        (0..factor).for_each(|_| {
            cx += ix;
            cy += iy;
            dst.push(pos2(cx, cy));
        });
        prev = pos;
    }
}

#[derive(Copy, Clone)]
pub struct FftBin {
    pub frequency: f32,
    pub amplitude: f32,
}

pub fn fft_spectrum(
    fft: Arc<dyn Fft<f32>>,
    buffer: &[f32],
    num_ch: usize,
    sample_rate: usize,
) -> Vec<FftBin> {
    let mut fft_buf: Vec<Complex<f32>> = buffer
        .chunks_exact(num_ch)
        .map(|s| Complex::new(s[0], 0.0))
        .collect();
    let n = fft_buf.len();
    fft.process(&mut fft_buf);

    fft_buf
        .get(..=n / 2)
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(bin, s)| {
            let frequency = (bin * sample_rate) as f32 / n as f32;

            let boundary =
                frequency.round() as usize == sample_rate / 2 || frequency.round() == 0.0;
            let scale = if boundary { 1.0 } else { 2.0 };
            let amplitude = 2.0 * s.norm() * scale / n as f32;
            FftBin {
                frequency,
                amplitude,
            }
        })
        .collect::<Vec<FftBin>>()
}

pub fn fft_max_frequency_bin(
    fft: Arc<dyn Fft<f32>>,
    buffer: &[f32],
    num_ch: usize,
    sample_rate: usize,
) -> FftBin {
    let spectrum = fft_spectrum(fft, buffer, num_ch, sample_rate);
    let mut frequency = 0.0;
    let amplitude = spectrum.iter().fold(0.0, |acc, s| {
        if s.amplitude > acc {
            frequency = s.frequency;
            s.amplitude
        } else {
            acc
        }
    });
    FftBin {
        frequency,
        amplitude,
    }
}
