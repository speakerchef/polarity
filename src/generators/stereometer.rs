use crate::{Rgba, audio::StereoFilter, state::*};
use egui::{Color32, Mesh, Pos2, Rect, pos2, vec2};
use std::{collections::VecDeque, ops::Neg};

use crate::{
    audio::audio_player::AudioPlayer,
    state::{AppState, TraceDensity},
};

const SQRT_3: f32 = 1.7320508;
const LINEAR_BIPOLAR_SF: f32 = 0.5;

#[derive(Default)]
pub struct Stereometer {
    pub kind: StereometerKind,
    pub render_mode: RenderMode,

    pub live_density: LiveDensity,
    pub trace_density: TraceDensity,

    pub filter_mode: FilterMode,
    pub filter_freq: f32,
    pub last_freq: f32,
    pub live_fs_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    pub trace_fs_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    pub live_mb_filters: Option<[(StereoFilter, StereoFilter, StereoFilter); 3]>,
    pub trace_mb_filters: Option<[(StereoFilter, StereoFilter, StereoFilter); 3]>,

    pub fs_color: Rgba,
    pub mb_color: [Rgba; 3],

    pub last_sample_idx: usize,
    pub trace_buffer: VecDeque<Pos2>,

    pub scale_factor: f32,
    pub point_size: f32,
}

fn filter_fs(st: &mut Stereometer, is_live: bool, l: f32, r: f32) -> (f32, f32) {
    if is_live {
        if let Some(live_fs) = &mut st.live_fs_filters {
            match st.filter_mode {
                FilterMode::Off => (l, r),
                FilterMode::Lpf => live_fs.0.run(l, r),
                FilterMode::Bpf => live_fs.1.run(l, r),
                FilterMode::Hpf => live_fs.2.run(l, r),
            }
        } else {
            (l, r)
        }
    } else {
        if let Some(trace_fs) = &mut st.trace_fs_filters {
            match st.filter_mode {
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

fn radial_scale(x: f32, y: f32) -> f32 {
    let sf = 0.3;
    let mag = (x * x + y * y).sqrt();
    let scaled = mag.powf(sf);
    if mag > 1e-6 { scaled / mag } else { 0.0 }
}

fn get_coord_from_meterkind(st: &AppState, l: f32, r: f32) -> (f32, f32) {
    let rscale = radial_scale(l, r);
    match st.stereo.kind {
        StereometerKind::LinearBipolar => {
            ((l - r) * LINEAR_BIPOLAR_SF, (l + r) * LINEAR_BIPOLAR_SF)
        }
        StereometerKind::ScaledBipolar => ((l - r) * rscale / SQRT_3, (l + r) * rscale / SQRT_3),
        StereometerKind::LinearLissajous => (l, r),
        StereometerKind::ScaledLissajous => (l * rscale, r * rscale),
    }
}

pub fn draw(p: &AudioPlayer, st: &mut AppState, center: Pos2) -> Mesh {
    let num_channels = p.contents.num_channels as usize;
    let mut live_mesh = Mesh::default();
    let mut trace_mesh = Mesh::default();

    let sample_pos = p.position().as_secs_f64();
    let sample_idx = (sample_pos * p.contents.sample_rate as f64) as usize;
    let (fsr, fsg, fsb, _) = st.stereo.fs_color.as_tuple();
    let last_idx = st.stereo.last_sample_idx;

    let trace_window = p
        .contents
        .samples
        .get(last_idx * num_channels..sample_idx * num_channels)
        .unwrap_or_default();

    trace_window.chunks_exact(2).for_each(|s| {
        let l = s.first().unwrap();
        let r = s.last().unwrap_or(l);
        let (l, r) = filter_fs(&mut st.stereo, false, *l, *r);
        let (l, r) = get_coord_from_meterkind(st, l, r);
        let (sl, sr) = (l * st.stereo.scale_factor, r * st.stereo.scale_factor);
        st.stereo
            .trace_buffer
            .push_back(pos2(center.x + sl, center.y + sr.neg()));
    });
    st.stereo.last_sample_idx = sample_idx;
    while st.stereo.trace_buffer.len() > st.stereo.trace_density.count() {
        st.stereo.trace_buffer.pop_front();
    }
    st.stereo
        .trace_buffer
        .iter()
        .enumerate()
        .for_each(|(i, &pos)| {
            let alpha = ((i as f32 / TraceDensity::Max.count() as f32) * u8::MAX as f32) as u8;
            trace_mesh.add_colored_rect(
                Rect::from_min_size(pos, vec2(st.stereo.point_size, st.stereo.point_size)),
                Color32::from_rgba_unmultiplied(fsr, fsg, fsb, alpha),
            );
        });

    let live_window = p
        .contents
        .samples
        .get(sample_idx * num_channels..sample_idx * num_channels + st.stereo.live_density.count())
        .unwrap_or_default();

    live_window.chunks_exact(2).for_each(|s| {
        let l = s.first().unwrap();
        let r = s.last().unwrap_or(l);
        let (l, r) = filter_fs(&mut st.stereo, true, *l, *r);
        let (l, r) = get_coord_from_meterkind(st, l, r);
        let (sl, sr) = (l * st.stereo.scale_factor, r * st.stereo.scale_factor);
        live_mesh.add_colored_rect(
            Rect::from_min_size(
                pos2(center.x + sl, center.y + sr.neg()),
                vec2(st.stereo.point_size, st.stereo.point_size),
            ),
            Color32::from_rgb(fsr, fsg, fsb),
        );
    });
    live_mesh.append(trace_mesh);
    live_mesh
}
