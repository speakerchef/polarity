use crate::{
    ANIM_SCALE_FACTOR, AudioFileContents, CustomMaterial, DOT_HALF_SIZE, DrawableCursor,
    HISTORY_MAGENTA, HistoryDensity, HistoryMesh, LIVE_MAGENTA, LiveDensity, LiveMesh,
    MAX_WINDOW_SIZE, NUM_VERTICES, PlayingAudio, PreviewCanvas, RADIAL_SCALE_FACTOR,
};
use bevy::{asset::RenderAssetUsages, math::ops::sqrt, prelude::*, sprite_render::AlphaMode2d};
use biquad::{Biquad, Coefficients, DirectForm1};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
#[derive(Resource, Debug, Clone, Default, Hash, Eq, PartialEq)]
pub enum StereometerKind {
    #[default]
    LinearBipolar,
    ScaledBipolar,
    LinearLissajous,
    ScaledLissajous,
}
#[derive(Resource, Default, Debug, PartialEq, Clone)]
pub struct StereometerParams {
    pub kind: StereometerKind,
    pub live_density: LiveDensity,
    pub history_density: HistoryDensity,
    pub color: LinearRgba,
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
    params: Res<StereometerParams>,
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
            let (x_sample, y_sample) = get_xy_from_meterkind(&params.kind, left, right);
            goniometer.history_buffer.push_back(Vec2 {
                x: world_pos.x + x_sample * ANIM_SCALE_FACTOR,
                y: world_pos.y + y_sample * ANIM_SCALE_FACTOR,
            });
        }
    }

    let live_window = &audio
        .samples
        .get(
            cur_idx * audio.num_channels
                ..(cur_idx + params.live_density.count()) * audio.num_channels,
        )
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
            let (x_sample, y_sample) = get_xy_from_meterkind(&params.kind, left, right);
            Vec2 {
                x: world_pos.x + x_sample * ANIM_SCALE_FACTOR,
                y: world_pos.y + y_sample * ANIM_SCALE_FACTOR,
            }
        })
        .collect();

    while goniometer.live_buffer.len() > params.live_density.count() {
        goniometer.live_buffer.pop_front();
    }
    while goniometer.history_buffer.len() > params.history_density.count() {
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

pub fn draw(
    goniometer: Single<&Stereometer, With<DrawableCursor>>,
    live_mesh: Single<&Mesh2d, With<LiveMesh>>,
    history_mesh: Single<&Mesh2d, With<HistoryMesh>>,
    mut mesh: ResMut<Assets<Mesh>>,
    params: Res<StereometerParams>,
) {
    if let Some(mut history_mesh) = mesh.get_mut(history_mesh.id()) {
        history_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, {
            let color: Vec<_> = (0..MAX_WINDOW_SIZE)
                .flat_map(|i| {
                    let alpha = (i as f32 / MAX_WINDOW_SIZE as f32).powf(2.5);
                    let c = params.color.with_alpha(alpha).to_f32_array();
                    std::iter::repeat_n(c, 6)
                })
                .collect();
            color
        });
        let pos: Vec<_> = goniometer
            .history_buffer
            .iter()
            .flat_map(|&v| point_to_quad_vertices(v))
            .collect();
        history_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    }
    if let Some(mut live_mesh) = mesh.get_mut(live_mesh.id()) {
        live_mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![params.color.to_f32_array(); NUM_VERTICES * MAX_WINDOW_SIZE],
        );
        let pos: Vec<_> = goniometer
            .live_buffer
            .iter()
            .flat_map(|&v| point_to_quad_vertices(v))
            .collect();
        live_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    }
}

pub fn spawn_stereometer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    params: Res<StereometerParams>,
) {
    let mut live_mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let mut history_mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let live_zeros: Vec<[f32; 3]> = vec![[0., 0., 10.]; MAX_WINDOW_SIZE * NUM_VERTICES];
    let hist_zeros: Vec<[f32; 3]> = vec![[0., 0., 0.]; MAX_WINDOW_SIZE * NUM_VERTICES];
    let hist_colors: Vec<[f32; 4]> = (0..MAX_WINDOW_SIZE)
        .flat_map(|i| {
            let alpha = (i as f32 / MAX_WINDOW_SIZE as f32).powf(2.5);
            let c = HISTORY_MAGENTA.with_alpha(alpha).to_f32_array();
            std::iter::repeat_n(c, 6)
        })
        .collect();
    live_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, live_zeros);
    live_mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        // vec![LIVE_MAGENTA.to_f32_array(); MAX_WINDOW_SIZE * NUM_VERTICES],
        vec![params.color.to_f32_array(); MAX_WINDOW_SIZE * NUM_VERTICES],
    );
    history_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, hist_zeros);
    history_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, hist_colors);

    let goni_id = commands.spawn_empty().id();

    commands.entity(goni_id).insert((
        Stereometer {
            live_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),
            history_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),
            last_sample_idx: 0,
            id: goni_id,
            filterbank: None,
        },
        DrawableCursor,
    ));
    commands.spawn((
        LiveMesh,
        Mesh2d(meshes.add(live_mesh)),
        MeshMaterial2d(materials.add(CustomMaterial {
            color: LinearRgba::default(),
            alpha_mode: AlphaMode2d::Blend,
        })),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
    commands.spawn((
        HistoryMesh,
        Mesh2d(meshes.add(history_mesh)),
        MeshMaterial2d(materials.add(CustomMaterial {
            color: LinearRgba::default(),
            alpha_mode: AlphaMode2d::Blend,
        })),
    ));
    info!("Spawned Goniometer");
}

pub fn despawn_stereometer(
    mut commands: Commands,
    q_goniometer: Single<&Stereometer, With<DrawableCursor>>,
) {
    commands.entity(q_goniometer.id).despawn();
}
