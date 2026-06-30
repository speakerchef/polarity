use std::ops::Div;

use eframe::egui::{Pos2, pos2};

use crate::audio::audio_player::AudioPlayer;

pub mod fluidwave;
pub mod rendering;
pub mod stereometer;
pub const DAMP_FACTOR: f32 = 1.25;
pub const MAX_RANGE: f32 = 0.95;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Envelope {
    pub attack: f32,
    pub release: f32,
    pub range: f32,
    pub sensitivity: f32,

    #[serde(skip)]
    pub last_idx: usize,
    #[serde(skip)]
    pub envelope: f32,
}
impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.01,
            release: 0.01,
            range: 50.0,
            sensitivity: 105.0,
            last_idx: 0,
            envelope: 0.0,
        }
    }
}
impl Envelope {
    pub fn new(attack: f32, release: f32, range: f32, sensitivity: f32) -> Self {
        Self {
            attack,
            release,
            range,
            sensitivity,
            last_idx: 0,
            envelope: 0.0,
        }
    }
    fn update_envelope(&mut self, new_value: f32) {
        self.envelope = new_value;
    }
    pub fn envelope(&self) -> f32 {
        let range_scale = 100.0 / self.range;
        let sens_scale = 100.0 / self.sensitivity.max(1.0);
        (self
            .envelope
            .div(range_scale)
            .powf(DAMP_FACTOR)
            .min(sens_scale * MAX_RANGE)
            + (1.0 - sens_scale * MAX_RANGE))
            .max(0.0)
    }
    pub fn run_detector(&mut self, pl: &AudioPlayer, export_sample_idx: Option<usize>) {
        let num_channels = pl.contents.num_channels as usize;
        let mut last_idx = self.last_idx;
        let sample_idx = export_sample_idx.unwrap_or_else(|| {
            (pl.position().as_secs_f64() * pl.contents.sample_rate as f64) as usize
        });
        if last_idx > sample_idx {
            last_idx = sample_idx;
        }

        let ef_window = &pl
            .contents
            .samples
            .get(last_idx * num_channels..sample_idx * num_channels)
            .unwrap_or_default();
        let mut ls = self.envelope;
        let mut frame_max = 0.0_f32;
        for s in ef_window.chunks_exact(2) {
            let l = s.first().unwrap_or(&0.0);
            let r = s.last().unwrap_or(l);
            let mag = (l.abs() + r.abs()) / 2.0;
            ls = if mag > ls {
                ls * self.attack + (1.0 - self.attack) * mag
            } else {
                ls * self.release + (1.0 - self.release) * mag
            };
            frame_max = frame_max.max(ls);
        }
        self.update_envelope(frame_max);
        self.last_idx = sample_idx;
    }
}

pub fn points_to_quad_vertices(s: f32, l: f32, r: f32) -> [Pos2; 6] {
    [
        pos2(l + s, r + s),
        pos2(l + s, r - s),
        pos2(l - s, r - s),
        pos2(l + s, r + s),
        pos2(l - s, r + s),
        pos2(l - s, r - s),
    ]
}

// pub fn envelope_follower(pl: &AudioPlayer, env: &mut Envelope, export_sample_idx: Option<usize>) {
//     let num_channels = pl.contents.num_channels as usize;
//     let mut last_idx = env.last_idx;
//     let sample_idx = export_sample_idx
//         .unwrap_or_else(|| (pl.position().as_secs_f64() * pl.contents.sample_rate as f64) as usize);
//     if last_idx > sample_idx {
//         last_idx = sample_idx;
//     }
//
//     let ef_window = &pl
//         .contents
//         .samples
//         .get(last_idx * num_channels..sample_idx * num_channels)
//         .unwrap_or_default();
//     let mut ls = env.envelope;
//     for s in ef_window.chunks_exact(2) {
//         let l = s.first().unwrap_or(&0.0);
//         let r = s.last().unwrap_or(l);
//         let absl = l.abs();
//         let absr = r.abs();
//         let (left, right) = if (absl + absr) / 2.0 > ls {
//             (
//                 ls * env.attack + (1.0 - env.attack) * absl,
//                 ls * env.attack + (1.0 - env.attack) * absr,
//             )
//         } else {
//             (
//                 ls * env.release + (1.0 - env.release) * absl,
//                 ls * env.release + (1.0 - env.release) * absr,
//             )
//         };
//         ls = (left + right) / 2.0;
//     }
//     env.update_envelope(ls);
//     env.last_idx = sample_idx;
// }
