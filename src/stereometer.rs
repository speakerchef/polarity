use crate::{
    ANIM_SCALE_FACTOR, AudioFileContents, DOT_HALF_SIZE, DrawableCursor, HISTORY_WINDOW_SIZE,
    HistoryMesh, LIVE_WINDOW_SIZE, LiveMesh, NUM_VERTICES, PlayingAudio, PreviewCanvas,
    RADIAL_SCALE_FACTOR,
};
use bevy::{math::ops::sqrt, prelude::*};
use biquad::{Biquad, Coefficients, DirectForm1};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
#[derive(Resource, Debug, Clone, Default)]
pub enum StereometerKind {
    LinearBipolar,
    #[default]
    ScaledBipolar,
    LinearLissajous,
    ScaledLissajous,
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

    pub fn run(&mut self, l: f32, r: f32) -> (f32, f32) {
        (self.l.run(l), self.r.run(r))
    }
}

#[derive(Component, Debug, Clone)]
pub struct Stereometer {
    pub live_buffer: VecDeque<Vec2>,
    pub history_buffer: VecDeque<Vec2>,
    pub id: Entity,
    pub last_sample_idx: usize,

    pub filterbank: Option<HashMap<Arc<str>, StereoFilter>>,
}

#[derive(Component, Debug, Clone)]
pub struct Oscilloscope(pub VecDeque<Vec2>, pub Entity);

fn get_xy_from_meterkind(
    kind: &StereometerKind,
    left: f32,
    right: f32,
) -> (f32 /* x */, f32 /* y */) {
    match kind {
        StereometerKind::LinearBipolar => ((left - right) / 2., (left + right) / 2.),
        StereometerKind::ScaledBipolar => {
            let sf = radial_scale(left, right) / sqrt(3.);
            ((left - right) * sf, (left + right) * sf)
        }
        StereometerKind::LinearLissajous => (left / 1.1, right / 1.1),
        StereometerKind::ScaledLissajous => {
            let sf = radial_scale(left, right);
            (left * sf, right * sf)
        }
    }
}

fn radial_scale(left: f32, right: f32) -> f32 {
    let r = (left.powi(2) + right.powi(2)).sqrt();
    let r_new = r.powf(RADIAL_SCALE_FACTOR);
    if r > 1e-6 { r_new / r } else { 0.0 }
}

pub fn update(
    playing_audio: Single<&AudioSink, With<PlayingAudio>>,
    audio: Single<&AudioFileContents>,
    mut goniometer: Single<&mut Stereometer, With<DrawableCursor>>,
    canvas: Single<&UiGlobalTransform, With<PreviewCanvas>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    kind: Res<StereometerKind>,
) {
    let canvas_2d = canvas.translation;
    let (camera, camera_xform) = *camera;
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_xform, canvas_2d * 0.5) else {
        return;
    };

    let pos = playing_audio.position().as_secs_f64() % audio.duration;
    let mut last_idx = goniometer.last_sample_idx;
    let cur_idx = (audio.sample_rate as f64 * pos) as usize;

    if cur_idx > last_idx {
        let frame_size = (cur_idx - last_idx) * audio.num_channels;
        last_idx *= audio.num_channels;
        let history_window = &audio
            .samples
            .get(last_idx..last_idx + frame_size)
            .unwrap_or_else(|| {
                goniometer.last_sample_idx = 0;
                &[0.]
            });
        goniometer.last_sample_idx = cur_idx;
        for frame in history_window.chunks_exact(audio.num_channels) {
            let left = frame[0];
            let right = *frame.last().unwrap_or(&left);
            let (x_sample, y_sample) = get_xy_from_meterkind(&*kind, left, right);
            goniometer.history_buffer.push_back(Vec2 {
                x: world_pos.x + x_sample * ANIM_SCALE_FACTOR,
                y: world_pos.y + y_sample * ANIM_SCALE_FACTOR,
            });
        }
    }

    let live_window = &audio
        .samples
        .get(cur_idx * audio.num_channels..(cur_idx + LIVE_WINDOW_SIZE) * audio.num_channels)
        .unwrap_or_else(|| {
            goniometer.last_sample_idx = 0;
            &[0.]
        });
    goniometer.live_buffer = live_window
        .chunks_exact(audio.num_channels)
        .map(|frame| {
            let left = frame[0];
            let right = *frame.last().unwrap_or(&frame[0]);
            // let (left, right) = goniometer.filterbank.0.run(left, right); // filtered
            let (x_sample, y_sample) = get_xy_from_meterkind(&*kind, left, right);
            Vec2 {
                x: world_pos.x + x_sample * ANIM_SCALE_FACTOR,
                y: world_pos.y + y_sample * ANIM_SCALE_FACTOR,
            }
        })
        .collect();

    while goniometer.live_buffer.len() > LIVE_WINDOW_SIZE {
        goniometer.live_buffer.pop_front();
    }
    while goniometer.history_buffer.len() > HISTORY_WINDOW_SIZE {
        goniometer.history_buffer.pop_front();
    }
}

/// Converts 2D vertex into 2 triangles forming a rectangle
fn point_to_quad_vertices(v: Vec2) -> [[f32; 3]; NUM_VERTICES] {
    let s = DOT_HALF_SIZE;
    [
        [v.x - s, v.y - s, 0.0],
        [v.x + s, v.y - s, 0.0],
        [v.x + s, v.y + s, 0.0],
        [v.x - s, v.y - s, 0.0],
        [v.x + s, v.y + s, 0.0],
        [v.x - s, v.y + s, 0.0],
    ]
}

// fn upsample(factor: usize) -> [[]; factor]

pub fn draw(
    goniometer: Single<&Stereometer, With<DrawableCursor>>,
    live_mesh: Single<&Mesh2d, With<LiveMesh>>,
    history_mesh: Single<&Mesh2d, With<HistoryMesh>>,
    mut mesh: ResMut<Assets<Mesh>>,
) {
    if let Some(mut history_mesh) = mesh.get_mut(history_mesh.id()) {
        let pos: Vec<[f32; 3]> = goniometer
            .history_buffer
            .iter()
            .flat_map(|&v| point_to_quad_vertices(v))
            .collect();
        history_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    }
    if let Some(mut live_mesh) = mesh.get_mut(live_mesh.id()) {
        let pos: Vec<_> = goniometer
            .live_buffer
            .iter()
            .flat_map(|&v| point_to_quad_vertices(v))
            .collect();
        live_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    }
}
