use crate::{
    ANIM_SCALE_FACTOR, AudioFileContents, DrawableCursor, PlayingAudio, PreviewCanvas, WINDOW_SIZE,
};
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component, Debug, Clone)]
pub struct Goniometer {
    pub window_buffer: VecDeque<Vec2>,
    pub id: Entity,
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

    let pos = playing_audio.position().as_secs_f64();
    let sample_idx = std::cmp::min(
        (audio.sample_rate as f64 * pos) as usize,
        audio.samples.len() - audio.sample_rate as usize,
    );
    let frame_idx = sample_idx * audio.num_channels;
    let window = &audio.samples[frame_idx..frame_idx + WINDOW_SIZE * audio.num_channels];
    let mut counter: usize = 0;
    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_xform, canvas_2d * 0.5) {
        goniometer.window_buffer = window
            .windows(2)
            .map(|frame| {
                counter += 2;
                let first = *frame.first().unwrap_or(&0.);
                let last = *frame.last().unwrap_or(&0.);
                Vec2 {
                    x: world_pos.x + first * ANIM_SCALE_FACTOR,
                    y: world_pos.y + last * ANIM_SCALE_FACTOR,
                }
            })
            .collect();
    }

    while goniometer.window_buffer.len() > WINDOW_SIZE {
        goniometer.window_buffer.pop_front();
    }
}

pub fn recolor(
    _q_cursor: Single<&Goniometer, With<DrawableCursor>>,
    q_points: Query<&MeshMaterial2d<ColorMaterial>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (i, entity) in q_points.iter().enumerate() {
        if let Some(mat) = materials.get_mut(&entity.0) {
            let alpha = (i as f32 / WINDOW_SIZE as f32) + 0.25;
            mat.color = Color::srgba(2.0, 0.3, 1.4, alpha);
            mat.alpha_mode = bevy::sprite_render::AlphaMode2d::Blend
        }
    }
}

pub fn draw(
    q_cursor: Single<(&Goniometer, &Mesh2d), With<DrawableCursor>>,
    mut mesh: ResMut<Assets<Mesh>>,
) {
    let (goniometer, mesh2d) = *q_cursor;
    if let Some(mesh) = mesh.get_mut(mesh2d.id()) {
        let pos: Vec<_> = goniometer
            .window_buffer
            .iter()
            .map(|v| [v.x, v.y, 0.0])
            .collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    }
}
