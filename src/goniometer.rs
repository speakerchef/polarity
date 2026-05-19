use crate::{
    ANIM_SCALE_FACTOR, AudioFileContents, DrawableCursor, HISTORY_WINDOW_SIZE, HistoryMesh,
    LIVE_WINDOW_SIZE, LiveMesh, PlayingAudio, PreviewCanvas,
};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component, Debug, Clone)]
pub struct Goniometer {
    pub live_buffer: VecDeque<Vec2>,
    pub history_buffer: VecDeque<Vec2>,
    pub id: Entity,
    pub last_sample_idx: usize,
}

#[derive(Component, Debug, Clone)]
pub struct Oscilloscope(pub VecDeque<Vec2>, pub Entity);

pub fn update(
    playing_audio: Single<&AudioSink, With<PlayingAudio>>,
    audio: Single<&AudioFileContents>,
    mut goniometer: Single<&mut Goniometer, With<DrawableCursor>>,
    canvas: Single<&UiGlobalTransform, With<PreviewCanvas>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let canvas_2d = canvas.translation;
    let (camera, camera_xform) = *camera;
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_xform, canvas_2d * 0.5) else {
        return;
    };

    let pos = playing_audio.position().as_secs_f64();
    let mut last_idx = goniometer.last_sample_idx;
    let cur_idx = std::cmp::min(
        (audio.sample_rate as f64 * pos) as usize,
        audio.samples.len() - 1,
    );

    if cur_idx > last_idx {
        let frame_size = (cur_idx - last_idx) * audio.num_channels;
        last_idx *= audio.num_channels;
        let history_window = &audio.samples[last_idx..last_idx + frame_size];
        goniometer.last_sample_idx = cur_idx;
        for frame in history_window.chunks_exact(audio.num_channels) {
            let first = frame[0];
            let last = *frame.last().unwrap_or(&first);
            goniometer.history_buffer.push_back(Vec2 {
                x: world_pos.x + first * ANIM_SCALE_FACTOR,
                y: world_pos.y + last * ANIM_SCALE_FACTOR,
            });
        }
    }

    let live_window = &audio.samples
        [cur_idx * audio.num_channels..(cur_idx + LIVE_WINDOW_SIZE) * audio.num_channels];
    goniometer.live_buffer = live_window
        .windows(audio.num_channels)
        .map(|frame| {
            let first = frame[0];
            let last = *frame.last().unwrap_or(&first);
            Vec2 {
                x: world_pos.x + first * ANIM_SCALE_FACTOR,
                y: world_pos.y + last * ANIM_SCALE_FACTOR,
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

pub fn recolor(
    _q_cursor: Single<&Goniometer, With<DrawableCursor>>,
    live_mesh: Query<&MeshMaterial2d<ColorMaterial>, With<LiveMesh>>,
    history_mesh: Query<&MeshMaterial2d<ColorMaterial>, With<HistoryMesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (i, entity) in live_mesh.iter().enumerate() {
        if let Some(mat) = materials.get_mut(&entity.0) {
            let alpha = (i as f32 / LIVE_WINDOW_SIZE as f32) + 0.25;
            mat.color = Color::srgba(2.0, 0.3, 1.4, alpha);
            mat.alpha_mode = bevy::sprite_render::AlphaMode2d::Blend
        }
    }
}

pub fn draw(
    goniometer: Single<&Goniometer, With<DrawableCursor>>,
    live_mesh: Single<&Mesh2d, With<LiveMesh>>,
    history_mesh: Single<&Mesh2d, With<HistoryMesh>>,
    mut mesh: ResMut<Assets<Mesh>>,
) {
    if let Some(live_mesh) = mesh.get_mut(live_mesh.id()) {
        let pos: Vec<_> = goniometer
            .live_buffer
            .iter()
            .map(|v| [v.x, v.y, 0.0])
            .collect();
        live_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    }
    if let Some(history_mesh) = mesh.get_mut(history_mesh.id()) {
        let pos: Vec<_> = goniometer
            .history_buffer
            .iter()
            .map(|v| [v.x, v.y, 0.0])
            .collect();
        history_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    }
}
