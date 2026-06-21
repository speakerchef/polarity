#![allow(dead_code)]
use biquad::*;
use std::{fs, io::Read, path::PathBuf};

pub mod audio_player;

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
