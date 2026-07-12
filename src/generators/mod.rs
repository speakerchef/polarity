use biquad::*;

use crate::{
    audio::{StereoFilter, audio_player::AudioPlayer},
    generators::fluidwave::ModSrc,
    labeled_enum,
    traits::Labeled,
};

pub mod fluidwave;
pub mod oscilloscope;
pub mod rendering;
pub mod stereometer;

pub const TARGET_FPS: f32 = 30.0;
pub const SUBSTEP_DIV: f32 = 6.0;
pub const MIN_SUBSTEP_DIV: f32 = 3.0;
pub const TARGET_DT: f32 = 1. / TARGET_FPS / SUBSTEP_DIV;

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
    fn text(self) -> &'static str {
        self.label()
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Envelope {
    pub attack: f32,
    pub release: f32,
    pub sensitivity: f32,

    #[serde(skip)]
    pub last_idx: usize,
    #[serde(skip)]
    cur_envelope: f32,
    #[serde(skip)]
    fast_envelope: f32,
    #[serde(skip)]
    slow_envelope: f32,
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
            last_idx: 0,
            fast_envelope: 0.0,
            slow_envelope: 0.0,
            cur_envelope: 0.0,
            hpf: Some(StereoFilter::from_coeffs_butterworth(
                Type::HighPass,
                HP_FREQ,
                sample_rate,
            )),
        }
    }
    pub fn envelope(&self, range: f32) -> f32 {
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
    pub fn run_differential_follower(
        &mut self,
        pl: &AudioPlayer,
        export_sample_idx: Option<usize>,
    ) {
        let num_ch = pl.contents.num_channels as usize;
        let sample_rate = pl.contents.sample_rate as f32;

        let mut last_idx = self.last_idx;
        let sample_idx = export_sample_idx.unwrap_or_else(|| {
            (pl.position().as_secs_f64() * pl.contents.sample_rate as f64) as usize
        });
        if last_idx > sample_idx {
            last_idx = sample_idx;
        }

        let window = &pl
            .contents
            .samples
            .get(last_idx * num_ch..sample_idx * num_ch)
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
        for frame in window.chunks_exact(2) {
            let (l, r) = (frame.first().unwrap_or(&0.0), frame.last().unwrap_or(&0.0));
            let (l, r) = self
                .hpf
                .as_mut()
                .expect("unreachable without filter")
                .run(*l, *r);
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
        self.last_idx = sample_idx;
    }
}
