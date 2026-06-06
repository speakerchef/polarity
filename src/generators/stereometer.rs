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
    pub live_mb_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    pub trace_mb_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,

    pub fs_color: Rgba,
    pub mb_color: [Rgba; 3],

    pub last_sample_idx: usize,
    pub trace_buffer: VecDeque<Pos2>,
    pub trace_low_buffer: VecDeque<Pos2>,
    pub trace_mid_buffer: VecDeque<Pos2>,
    pub trace_high_buffer: VecDeque<Pos2>,

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

enum FilterBand {
    Low,
    Mid,
    High,
}

fn filter_mb(st: &mut Stereometer, is_live: bool, band: FilterBand, l: f32, r: f32) -> (f32, f32) {
    if is_live {
        if let Some(live) = &mut st.live_mb_filters {
            match band {
                FilterBand::Low => live.0.run(l, r),
                FilterBand::Mid => live.1.run(l, r),
                FilterBand::High => live.2.run(l, r),
            }
        } else {
            println!("NO FILTER");
            (l, r)
        }
    } else {
        if let Some(trace) = &mut st.trace_mb_filters {
            match band {
                FilterBand::Low => trace.0.run(l, r),
                FilterBand::Mid => trace.1.run(l, r),
                FilterBand::High => trace.2.run(l, r),
            }
        } else {
            println!("NO FILTER");
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
    let (mblowr, mblowg, mblowb, _) = st.stereo.mb_color[0].as_tuple();
    let (mbmidr, mbmidg, mbmidb, _) = st.stereo.mb_color[1].as_tuple();
    let (mbhighr, mbhighg, mbhighb, _) = st.stereo.mb_color[2].as_tuple();
    let last_idx = st.stereo.last_sample_idx;

    if sample_idx < last_idx {
        st.stereo.trace_buffer.clear();
    }

    let trace_window = p
        .contents
        .samples
        .get(last_idx * num_channels..sample_idx * num_channels)
        .unwrap_or_default();

    trace_window.chunks_exact(2).for_each(|s| {
        let l = s.first().unwrap();
        let r = s.last().unwrap_or(l);
        match st.stereo.render_mode {
            RenderMode::FullSpectrum => {
                let (l, r) = filter_fs(&mut st.stereo, false, *l, *r);
                let (l, r) = get_coord_from_meterkind(st, l, r);
                let (sl, sr) = (l * st.stereo.scale_factor, r * st.stereo.scale_factor);
                let pos = pos2(center.x + sl, center.y + sr.neg());
                st.stereo.trace_buffer.push_back(pos);
            }
            RenderMode::MultiBand => {
                let (lowl, lowr) = filter_mb(&mut st.stereo, false, FilterBand::Low, *l, *r);
                let (midl, midr) = filter_mb(&mut st.stereo, false, FilterBand::Mid, *l, *r);
                let (highl, highr) = filter_mb(&mut st.stereo, false, FilterBand::High, *l, *r);
                let (lowl, lowr) = get_coord_from_meterkind(st, lowl, lowr);
                let (midl, midr) = get_coord_from_meterkind(st, midl, midr);
                let (highl, highr) = get_coord_from_meterkind(st, highl, highr);
                let (lowl, lowr) = (lowl * st.stereo.scale_factor, lowr * st.stereo.scale_factor);
                let (midl, midr) = (midl * st.stereo.scale_factor, midr * st.stereo.scale_factor);
                let (highl, highr) = (
                    highl * st.stereo.scale_factor,
                    highr * st.stereo.scale_factor,
                );
                let posl = pos2(center.x + lowl, center.y + lowr.neg());
                let posm = pos2(center.x + midl, center.y + midr.neg());
                let posh = pos2(center.x + highl, center.y + highr.neg());
                st.stereo.trace_low_buffer.push_back(posl);
                st.stereo.trace_mid_buffer.push_back(posm);
                st.stereo.trace_high_buffer.push_back(posh);
            }
        }
    });
    st.stereo.last_sample_idx = sample_idx;
    while st.stereo.trace_buffer.len() > st.stereo.trace_density.count() {
        st.stereo.trace_buffer.pop_front();
    }
    while st.stereo.trace_low_buffer.len() > st.stereo.trace_density.count() {
        st.stereo.trace_low_buffer.pop_front();
    }
    while st.stereo.trace_mid_buffer.len() > st.stereo.trace_density.count() {
        st.stereo.trace_mid_buffer.pop_front();
    }
    while st.stereo.trace_high_buffer.len() > st.stereo.trace_density.count() {
        st.stereo.trace_high_buffer.pop_front();
    }
    match st.stereo.render_mode {
        RenderMode::FullSpectrum => {
            st.stereo
                .trace_buffer
                .iter()
                .enumerate()
                .for_each(|(i, &pos)| {
                    let alpha =
                        ((i as f32 / TraceDensity::Max.count() as f32) * u8::MAX as f32) as u8;
                    trace_mesh.add_colored_rect(
                        Rect::from_min_size(pos, vec2(st.stereo.point_size, st.stereo.point_size)),
                        Color32::from_rgba_unmultiplied(fsr, fsg, fsb, alpha),
                    );
                });
        }
        RenderMode::MultiBand => {
            st.stereo
                .trace_low_buffer
                .iter()
                .enumerate()
                .for_each(|(i, &pos)| {
                    let alpha =
                        ((i as f32 / TraceDensity::Max.count() as f32) * u8::MAX as f32) as u8;
                    trace_mesh.add_colored_rect(
                        Rect::from_min_size(pos, vec2(st.stereo.point_size, st.stereo.point_size)),
                        Color32::from_rgba_unmultiplied(mblowr, mblowg, mblowb, alpha),
                    );
                });
            st.stereo
                .trace_mid_buffer
                .iter()
                .enumerate()
                .for_each(|(i, &pos)| {
                    let alpha =
                        ((i as f32 / TraceDensity::Max.count() as f32) * u8::MAX as f32) as u8;
                    trace_mesh.add_colored_rect(
                        Rect::from_min_size(pos, vec2(st.stereo.point_size, st.stereo.point_size)),
                        Color32::from_rgba_unmultiplied(mbmidr, mbmidg, mbmidb, alpha),
                    );
                });
            st.stereo
                .trace_high_buffer
                .iter()
                .enumerate()
                .for_each(|(i, &pos)| {
                    let alpha =
                        ((i as f32 / TraceDensity::Max.count() as f32) * u8::MAX as f32) as u8;
                    trace_mesh.add_colored_rect(
                        Rect::from_min_size(pos, vec2(st.stereo.point_size, st.stereo.point_size)),
                        Color32::from_rgba_unmultiplied(mbhighr, mbhighg, mbhighb, alpha),
                    );
                });
        }
    }

    let live_window = p
        .contents
        .samples
        .get(sample_idx * num_channels..sample_idx * num_channels + st.stereo.live_density.count())
        .unwrap_or_default();

    match st.stereo.render_mode {
        RenderMode::FullSpectrum => {
            live_window.chunks_exact(2).for_each(|s| {
                let l = s.first().unwrap();
                let r = s.last().unwrap_or(l);
                let (l, r) = filter_fs(&mut st.stereo, true, *l, *r);
                let (l, r) = get_coord_from_meterkind(st, l, r);
                let (sl, sr) = (l * st.stereo.scale_factor, r * st.stereo.scale_factor);
                let pos = pos2(center.x + sl, center.y + sr.neg());
                live_mesh.add_colored_rect(
                    Rect::from_min_size(pos, vec2(st.stereo.point_size, st.stereo.point_size)),
                    Color32::from_rgb(fsr, fsg, fsb),
                );
            });
        }
        RenderMode::MultiBand => {
            live_window.chunks_exact(2).for_each(|s| {
                let l = s.first().unwrap();
                let r = s.last().unwrap_or(l);

                let (lowl, lowr) = filter_mb(&mut st.stereo, true, FilterBand::Low, *l, *r);
                let (midl, midr) = filter_mb(&mut st.stereo, true, FilterBand::Mid, *l, *r);
                let (highl, highr) = filter_mb(&mut st.stereo, true, FilterBand::High, *l, *r);

                let (lowl, lowr) = get_coord_from_meterkind(st, lowl, lowr);
                let (midl, midr) = get_coord_from_meterkind(st, midl, midr);
                let (highl, highr) = get_coord_from_meterkind(st, highl, highr);

                let (lowl, lowr) = (lowl * st.stereo.scale_factor, lowr * st.stereo.scale_factor);
                let (midl, midr) = (midl * st.stereo.scale_factor, midr * st.stereo.scale_factor);
                let (highl, highr) = (
                    highl * st.stereo.scale_factor,
                    highr * st.stereo.scale_factor,
                );
                let posl = pos2(center.x + lowl, center.y + lowr.neg());
                let posm = pos2(center.x + midl, center.y + midr.neg());
                let posh = pos2(center.x + highl, center.y + highr.neg());

                live_mesh.add_colored_rect(
                    Rect::from_min_size(posl, vec2(st.stereo.point_size, st.stereo.point_size)),
                    Color32::from_rgb(mblowr, mblowg, mblowb),
                );
                live_mesh.add_colored_rect(
                    Rect::from_min_size(posm, vec2(st.stereo.point_size, st.stereo.point_size)),
                    Color32::from_rgb(mbmidr, mbmidg, mbmidb),
                );
                live_mesh.add_colored_rect(
                    Rect::from_min_size(posh, vec2(st.stereo.point_size, st.stereo.point_size)),
                    Color32::from_rgb(mbhighr, mbhighg, mbhighb),
                );
            });
        }
    }

    live_mesh.append(trace_mesh);
    live_mesh
}
