#![allow(dead_code)]
use biquad::*;
use std::{fs, io::Read, path::PathBuf};

use crate::state::AppState;

pub mod audio_inputs;

fn file_as_raw_bytes(path: PathBuf) -> Vec<u8> {
    let mut bytes = Vec::new();
    let _ = fs::File::open(path.clone())
        .expect("error opening file")
        .read_to_end(&mut bytes);
    bytes
}

#[derive(Debug, Clone)]
pub struct StereoFilter {
    l: DirectForm1<f32>,
    r: DirectForm1<f32>,
}
impl StereoFilter {
    pub fn new(coeffs: Coefficients<f32>) -> Self {
        StereoFilter {
            l: DirectForm1::<f32>::new(coeffs),
            r: DirectForm1::<f32>::new(coeffs),
        }
    }

    pub fn from_coeffs_butterworth(ty: Type<f32>, f0: f32, fs: u32) -> Self {
        let coeffs = Coefficients::<f32>::from_params(
            ty,
            fs.clamp(1, 192_000).hz(),
            f0.clamp(1.0, 20_000.0).hz(),
            Q_BUTTERWORTH_F32,
        )
        .unwrap();
        StereoFilter {
            l: DirectForm1::<f32>::new(coeffs),
            r: DirectForm1::<f32>::new(coeffs),
        }
    }

    pub fn run(&mut self, l: f32, r: f32) -> (f32, f32) {
        (self.l.run(l), self.r.run(r))
    }
}

pub fn level_meter(st: &mut AppState, export_sample_idx: Option<usize>) -> (f32, f32) {
    let Some(ai) = st.active_input() else {
        return (-1.0, -1.0);
    };
    let level = ai.peak_level(export_sample_idx);
    let level = (level.0.powf(0.6), level.1.powf(0.6));

    (-1.0 + level.0 * 2.0, -1.0 + level.1 * 2.0)
}
