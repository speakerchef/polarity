use crate::{
    ANIM_SCALE_FACTOR, AudioFileContents, DrawableCursor, PlayingAudio, PointArray, PreviewCanvas,
    WINDOW_SIZE,
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
    q_playing_audio: Single<&AudioSink, With<PlayingAudio>>,
    q_audio_data: Single<&AudioFileContents>,
    mut goniometer: Single<&mut Goniometer, With<DrawableCursor>>,
    q_canvas: Single<&UiGlobalTransform, With<PreviewCanvas>>,
    q_camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let canvas_2d = q_canvas.translation;
    let (camera, camera_xform) = *q_camera;

    let pos = q_playing_audio.position().as_secs_f64();
    let sample_idx = std::cmp::min(
        (q_audio_data.sample_rate as f64 * pos) as usize,
        q_audio_data.samples.len() - q_audio_data.sample_rate as usize,
    );
    let frame_idx = sample_idx * q_audio_data.num_channels;
    let window = &q_audio_data.samples[frame_idx..frame_idx + WINDOW_SIZE * 2];
    let mut counter: usize = 0;
    goniometer.window_buffer = window
        .windows(2)
        .map(|frame| {
            counter += 2;
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_xform, canvas_2d * 0.5) {
                let first = *frame.first().unwrap_or(&0.);
                let last = *frame.last().unwrap_or(&0.);
                Vec2 {
                    x: world_pos.x + first * ANIM_SCALE_FACTOR,
                    y: world_pos.y + last * ANIM_SCALE_FACTOR,
                }
            } else {
                Vec2 { x: 0., y: 0. }
            }
        })
        .collect();

    while goniometer.window_buffer.len() > WINDOW_SIZE {
        goniometer.window_buffer.pop_front();
    }
}

pub fn draw(
    q_cursor: Single<(&Goniometer, &mut PointArray), With<DrawableCursor>>,
    mut q_points: Query<&mut Transform>,
) {
    let goniometer = q_cursor.0;

    for (i, entity) in q_cursor.1.0.iter().enumerate() {
        let coord = goniometer.window_buffer[i];
        if let Ok(mut transform) = q_points.get_mut(*entity) {
            transform.translation.x = coord.x;
            transform.translation.y = coord.y;
        }
    }
}
