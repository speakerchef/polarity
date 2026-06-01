use crate::{
    AudioFileContents, CustomMaterial, DrawableCursor, FilteringMode, LiveDensity, LiveMesh,
    MAX_WINDOW_SIZE, NUM_VERTICES, PlayingAudio, PreviewCanvas, RADIAL_SCALE_FACTOR, TraceDensity,
    TraceMesh,
};
use bevy::{asset::RenderAssetUsages, math::ops::sqrt, prelude::*, sprite_render::AlphaMode2d};
use biquad::*;
use std::collections::VecDeque;
#[derive(Resource, Debug, Clone, Default, Hash, Eq, PartialEq)]
pub enum StereometerKind {
    LinearBipolar,
    ScaledBipolar,
    LinearLissajous,
    #[default]
    ScaledLissajous,
}

#[derive(Resource, Default, Debug, PartialEq, Clone)]
pub enum StereometerRenderMode {
    FullSpectrum,
    #[default]
    MultiBand,
}

impl From<StereometerRenderMode> for String {
    fn from(value: StereometerRenderMode) -> Self {
        match value {
            StereometerRenderMode::FullSpectrum => "Full Spectrum".to_string(),
            StereometerRenderMode::MultiBand => "Multi-Band".to_string(),
        }
    }
}
impl std::fmt::Display for StereometerRenderMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StereometerRenderMode::FullSpectrum => write!(f, "Full Spectrum"),
            StereometerRenderMode::MultiBand => write!(f, "Multi-Band"),
        }
    }
}

#[derive(Resource, Default, Debug, PartialEq, Clone)]
pub struct StereometerParams {
    pub kind: StereometerKind,
    pub render_mode: StereometerRenderMode,
    pub filtering_mode: FilteringMode,
    pub live_density: LiveDensity,
    pub trace_density: TraceDensity,

    pub color: Hsla,
    pub multiband_color: (Hsla, Hsla, Hsla),

    pub freq: f32,

    pub scale_factor: f32,
    pub dot_size: f32,
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

#[derive(Component, Debug, Clone)]
pub struct Stereometer {
    // Full Spectrum
    pub live_buffer: VecDeque<Vec2>,
    pub trace_buffer: VecDeque<Vec2>,
    // Multiband
    pub live_lf_buffer: VecDeque<Vec2>,
    pub live_mf_buffer: VecDeque<Vec2>,
    pub live_hf_buffer: VecDeque<Vec2>,
    pub trace_lf_buffer: VecDeque<Vec2>,
    pub trace_mf_buffer: VecDeque<Vec2>,
    pub trace_hf_buffer: VecDeque<Vec2>,

    pub id: Entity,
    pub last_sample_idx: usize,

    pub live_filterbank: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    pub trace_filterbank: Option<(StereoFilter, StereoFilter, StereoFilter)>,

    pub mb_filterbank: Option<(StereoFilter, StereoFilter, StereoFilter)>,
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

fn render_with_mode(
    stereometer: &mut Stereometer,
    params: &StereometerParams,
    window_buf: &&[f32],
    chunk: usize,
    world_pos: Vec2,
    is_trace: bool,
) {
    match params.render_mode {
        StereometerRenderMode::FullSpectrum => {
            let buf = window_buf
                .chunks_exact(chunk)
                .map(|frame| {
                    let mut left = frame[0];
                    let mut right = *frame.last().unwrap_or(&frame[0]);

                    // Filtering
                    if let Some(fb) = if is_trace {
                        stereometer.trace_filterbank.as_mut()
                    } else {
                        stereometer.live_filterbank.as_mut()
                    } {
                        (left, right) = match params.filtering_mode {
                            FilteringMode::Off => (left, right),
                            FilteringMode::Lpf => fb.0.run(left, right),
                            FilteringMode::Bpf => fb.1.run(left, right),
                            FilteringMode::Hpf => fb.2.run(left, right),
                        };
                    }

                    let (x_sample, y_sample) = get_xy_from_meterkind(&params.kind, left, right);
                    Vec2 {
                        x: world_pos.x + x_sample * params.scale_factor,
                        y: world_pos.y + y_sample * params.scale_factor,
                    }
                })
                .collect();

            if is_trace {
                stereometer.trace_buffer.extend(buf);
            } else {
                stereometer.live_buffer = buf;
            }
        }
        StereometerRenderMode::MultiBand => {
            let mut lf_buf: VecDeque<Vec2> = VecDeque::new();
            let mut mf_buf: VecDeque<Vec2> = VecDeque::new();
            let mut hf_buf: VecDeque<Vec2> = VecDeque::new();
            window_buf.chunks_exact(chunk).for_each(|frame| {
                let l = frame[0];
                let r = *frame.last().unwrap_or(&frame[0]);

                // MV Filtering
                if let Some(fb) = stereometer.mb_filterbank.as_mut() {
                    let (lf_l, lf_r) = fb.0.run(l, r);
                    let (x, y) = get_xy_from_meterkind(&params.kind, lf_l, lf_r);
                    lf_buf.push_back(Vec2 {
                        x: world_pos.x + x * params.scale_factor,
                        y: world_pos.y + y * params.scale_factor,
                    });
                    let (mf_l, mf_r) = fb.1.run(l, r);
                    let (x, y) = get_xy_from_meterkind(&params.kind, mf_l, mf_r);
                    mf_buf.push_back(Vec2 {
                        x: world_pos.x + x * params.scale_factor,
                        y: world_pos.y + y * params.scale_factor,
                    });
                    let (hf_l, hf_r) = fb.2.run(l, r);
                    let (x, y) = get_xy_from_meterkind(&params.kind, hf_l, hf_r);
                    hf_buf.push_back(Vec2 {
                        x: world_pos.x + x * params.scale_factor,
                        y: world_pos.y + y * params.scale_factor,
                    });
                }
            });
            if is_trace {
                stereometer.trace_lf_buffer.extend(&lf_buf);
                stereometer.trace_mf_buffer.extend(&mf_buf);
                stereometer.trace_hf_buffer.extend(&hf_buf);
            } else {
                if !lf_buf.is_empty() && !mf_buf.is_empty() && !hf_buf.is_empty() {
                    stereometer.live_lf_buffer = lf_buf;
                    stereometer.live_mf_buffer = mf_buf;
                    stereometer.live_hf_buffer = hf_buf;
                }
            }
        }
    }
}

pub fn update(
    playing_audio: Single<&AudioSink, With<PlayingAudio>>,
    audio: Single<&AudioFileContents>,
    mut meter: Single<&mut Stereometer, With<DrawableCursor>>,
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
    let mut last_idx = meter.last_sample_idx;
    let cur_idx = (audio.sample_rate as f64 * pos) as usize;

    if cur_idx > last_idx {
        let frame_size = (cur_idx - last_idx) * audio.num_channels;
        last_idx *= audio.num_channels;
        let trace_window = &audio
            .samples
            .get(last_idx..last_idx + frame_size)
            .unwrap_or_else(|| {
                meter.last_sample_idx = 0;
                &[0.]
            });
        meter.last_sample_idx = cur_idx;
        render_with_mode(
            &mut meter,
            &params,
            trace_window,
            audio.num_channels,
            world_pos,
            true,
        );
    } else {
        meter.last_sample_idx = cur_idx;
    }

    let live_window = &audio
        .samples
        .get(
            cur_idx * audio.num_channels
                ..(cur_idx + params.live_density.count()) * audio.num_channels,
        )
        .unwrap_or_else(|| {
            meter.last_sample_idx = 0;
            &[0.]
        });
    render_with_mode(
        &mut meter,
        &params,
        live_window,
        audio.num_channels,
        world_pos,
        false,
    );

    while meter.live_buffer.len() > params.live_density.count() {
        meter.live_buffer.pop_front();
    }
    while meter.trace_buffer.len() > params.trace_density.count() {
        meter.trace_buffer.pop_front();
    }
    // MB
    while meter.live_lf_buffer.len() > params.live_density.count() {
        meter.live_lf_buffer.pop_front();
        meter.live_mf_buffer.pop_front();
        meter.live_hf_buffer.pop_front();
    }
    while meter.trace_lf_buffer.len() > params.trace_density.count() {
        meter.trace_lf_buffer.pop_front();
    }
    while meter.trace_mf_buffer.len() > params.trace_density.count() {
        meter.trace_mf_buffer.pop_front();
    }
    while meter.trace_hf_buffer.len() > params.trace_density.count() {
        meter.trace_hf_buffer.pop_front();
    }
}

/// Converts 2D vertex into 2 triangles forming a rectangle
fn point_to_quad_vertices(v: Vec2, dot_sz: f32) -> [[f32; 3]; NUM_VERTICES] {
    [
        [v.x - dot_sz, v.y - dot_sz, 0.0],
        [v.x + dot_sz, v.y - dot_sz, 0.0],
        [v.x + dot_sz, v.y + dot_sz, 0.0],
        [v.x - dot_sz, v.y - dot_sz, 0.0],
        [v.x + dot_sz, v.y + dot_sz, 0.0],
        [v.x - dot_sz, v.y + dot_sz, 0.0],
    ]
}

pub fn draw(
    stereometer: Single<&Stereometer, With<DrawableCursor>>,
    live_mesh: Single<&Mesh2d, With<LiveMesh>>,
    trace_mesh: Single<&Mesh2d, With<TraceMesh>>,
    mut mesh: ResMut<Assets<Mesh>>,
    params: Res<StereometerParams>,
) {
    if let Some(mut trace_mesh) = mesh.get_mut(trace_mesh.id()) {
        match params.render_mode {
            StereometerRenderMode::FullSpectrum => {
                let pos: Vec<_> = stereometer
                    .trace_buffer
                    .iter()
                    .flat_map(|&v| point_to_quad_vertices(v, params.dot_size))
                    .collect();
                trace_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
                trace_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, {
                    let color: Vec<_> = (0..stereometer.trace_buffer.len())
                        .flat_map(|i| {
                            let alpha = (i as f32 / MAX_WINDOW_SIZE as f32).powf(2.5);
                            let c = LinearRgba::from(params.color.with_alpha(alpha)).to_f32_array();
                            std::iter::repeat_n(c, NUM_VERTICES)
                        })
                        .collect();
                    color
                });
            }
            StereometerRenderMode::MultiBand => {
                let pos_lf: Vec<_> = stereometer
                    .trace_lf_buffer
                    .iter()
                    .flat_map(|&v| point_to_quad_vertices(v, params.dot_size))
                    .collect();
                let pos_mf: Vec<_> = stereometer
                    .trace_mf_buffer
                    .iter()
                    .flat_map(|&v| point_to_quad_vertices(v, params.dot_size))
                    .collect();
                let pos_hf: Vec<_> = stereometer
                    .trace_hf_buffer
                    .iter()
                    .flat_map(|&v| point_to_quad_vertices(v, params.dot_size))
                    .collect();
                let pos = [pos_lf, pos_mf, pos_hf].concat();
                trace_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);

                let color_lf: Vec<_> = (0..stereometer.trace_lf_buffer.len())
                    .flat_map(|i| {
                        let alpha = (i as f32 / MAX_WINDOW_SIZE as f32).powf(2.5);
                        // let c = params.multiband_color.0.with_alpha(alpha).to_f32_array();
                        let c = LinearRgba::RED.with_alpha(alpha).to_f32_array();
                        std::iter::repeat_n(c, NUM_VERTICES)
                    })
                    .collect();
                let color_mf: Vec<_> = (0..stereometer.trace_mf_buffer.len())
                    .flat_map(|i| {
                        let alpha = (i as f32 / MAX_WINDOW_SIZE as f32).powf(2.5);
                        // let c = params.multiband_color.0.with_alpha(alpha).to_f32_array();
                        let c = LinearRgba::GREEN.with_alpha(alpha).to_f32_array();
                        std::iter::repeat_n(c, NUM_VERTICES)
                    })
                    .collect();
                let color_hf: Vec<_> = (0..stereometer.trace_hf_buffer.len())
                    .flat_map(|i| {
                        let alpha = (i as f32 / MAX_WINDOW_SIZE as f32).powf(2.5);
                        // let c = params.multiband_color.0.with_alpha(alpha).to_f32_array();
                        let c = LinearRgba::BLUE.with_alpha(alpha).to_f32_array();
                        std::iter::repeat_n(c, NUM_VERTICES)
                    })
                    .collect();
                let colors = [color_lf, color_mf, color_hf].concat();
                trace_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            }
        }
    }
    if let Some(mut live_mesh) = mesh.get_mut(live_mesh.id()) {
        match params.render_mode {
            StereometerRenderMode::FullSpectrum => {
                let pos: Vec<_> = stereometer
                    .live_buffer
                    .iter()
                    .flat_map(|&v| point_to_quad_vertices(v, params.dot_size))
                    .collect();
                live_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
                live_mesh.insert_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    vec![
                        LinearRgba::from(params.color).to_f32_array();
                        NUM_VERTICES * stereometer.live_buffer.len()
                    ],
                );
            }
            StereometerRenderMode::MultiBand => {
                let pos_lf: Vec<_> = stereometer
                    .live_lf_buffer
                    .iter()
                    .flat_map(|&v| point_to_quad_vertices(v, params.dot_size))
                    .collect();
                let pos_mf: Vec<_> = stereometer
                    .live_mf_buffer
                    .iter()
                    .flat_map(|&v| point_to_quad_vertices(v, params.dot_size))
                    .collect();
                let pos_hf: Vec<_> = stereometer
                    .live_hf_buffer
                    .iter()
                    .flat_map(|&v| point_to_quad_vertices(v, params.dot_size))
                    .collect();
                let pos = [pos_lf, pos_mf, pos_hf].concat();
                live_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);

                let color_lf = vec![
                    LinearRgba::RED.to_f32_array();
                    NUM_VERTICES * stereometer.live_lf_buffer.len()
                ];
                let color_mf = vec![
                    LinearRgba::GREEN
                        .with_alpha(params.color.alpha)
                        .to_f32_array();
                    NUM_VERTICES * stereometer.live_mf_buffer.len()
                ];
                let color_hf = vec![
                    LinearRgba::BLUE
                        .with_alpha(params.color.alpha)
                        .to_f32_array();
                    NUM_VERTICES * stereometer.live_hf_buffer.len()
                ];
                let colors = [color_lf, color_mf, color_hf].concat();
                live_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            }
        }
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
    let mut trace_mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let live_zeros: Vec<[f32; 3]> = vec![[0., 0., 10.]; MAX_WINDOW_SIZE * NUM_VERTICES];
    let hist_zeros: Vec<[f32; 3]> = vec![[0., 0., 0.]; MAX_WINDOW_SIZE * NUM_VERTICES];
    let hist_colors: Vec<[f32; 4]> = (0..MAX_WINDOW_SIZE)
        .flat_map(|i| {
            let alpha = (i as f32 / MAX_WINDOW_SIZE as f32).powf(2.5);
            let c = params.color.with_alpha(alpha).to_f32_array();
            std::iter::repeat_n(c, NUM_VERTICES)
        })
        .collect();
    live_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, live_zeros);
    live_mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![params.color.to_f32_array(); MAX_WINDOW_SIZE * NUM_VERTICES],
    );
    trace_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, hist_zeros);
    trace_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, hist_colors);

    let goni_id = commands.spawn_empty().id();

    commands.entity(goni_id).insert((
        Stereometer {
            live_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),
            trace_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),

            live_lf_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),
            live_mf_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),
            live_hf_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),
            trace_lf_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),
            trace_mf_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),
            trace_hf_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),

            last_sample_idx: 0,
            id: goni_id,
            live_filterbank: None,
            trace_filterbank: None,
            mb_filterbank: None,
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
        TraceMesh,
        Mesh2d(meshes.add(trace_mesh)),
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
