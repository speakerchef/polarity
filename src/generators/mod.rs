use eframe::egui::{Pos2, pos2};

use crate::{audio::audio_player::AudioPlayer, state::AppState};

pub mod fluidwave;
pub mod rendering;
pub mod stereometer;

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

pub fn envelope_follower(pl: &AudioPlayer, st: &mut AppState, live: bool) {
    let num_channels = pl.contents.num_channels as usize;
    let mut last_idx = st.fwave.last_idx;
    let sample_idx = if live {
        (pl.position().as_secs_f64() * pl.contents.sample_rate as f64) as usize
    } else {
        st.export_sample_idx
    };
    if last_idx > sample_idx {
        last_idx = sample_idx;
    }

    let ef_window = &pl
        .contents
        .samples
        .get(last_idx * num_channels..sample_idx * num_channels)
        .unwrap_or_default();
    let mut ls = st.fwave.envelope_last_sample;
    for s in ef_window.chunks_exact(2) {
        let l = s.first().unwrap_or(&0.0);
        let r = s.last().unwrap_or(l);
        let absl = l.abs();
        let absr = r.abs();
        let (left, right) = if (absl + absr) / 2.0 > ls {
            (
                ls * st.fwave.attack + (1.0 - st.fwave.attack) * absl,
                ls * st.fwave.attack + (1.0 - st.fwave.attack) * absr,
            )
        } else {
            (
                ls * st.fwave.release + (1.0 - st.fwave.release) * absl,
                ls * st.fwave.release + (1.0 - st.fwave.release) * absr,
            )
        };
        ls = (left + right) / 2.0;
    }
    st.fwave.envelope_last_sample = ls;
    st.fwave.last_idx = sample_idx;
}
