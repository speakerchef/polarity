use biquad::*;
use eframe::egui_wgpu::{self, wgpu};
use std::{num::NonZeroU64, path::PathBuf, time::Duration};
use wgpu::util::DeviceExt;

use crate::{
    LinearRgba,
    audio::{StereoFilter, audio_player::*},
    generators::stereometer::MAX_LIVE_POINT_DENSITY,
    state::{FilterMode, LiveDensity, PlaybackMode},
    ui::{control_panel, timeline},
};
use egui::{CornerRadius, Key, Pos2, Stroke, StrokeKind, emath::GuiRounding, pos2};

use crate::{state::AppState, ui::palette as plt, ui::theme};

#[derive(Default)]
pub struct PolarityApp {
    st: AppState,
    player: Option<AudioPlayer>,
}

struct StereometerRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
}
impl StereometerRenderResources {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        pos: Vec<Pos2>,
        color: LinearRgba,
        live_density: LiveDensity,
    ) {
        let pos: Vec<[f32; 4]> = pos.iter().map(|pos| [pos.x, pos.y, 0.0, 1.0]).collect();
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[live_density.count()]),
        );
        queue.write_buffer(
            &self.uniform_buffer,
            16,
            bytemuck::cast_slice(&[color.r, color.g, color.b, color.a]),
        );
        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&[0f32, 0.0, 0.0, 0.0]),
        );
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&pos));
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[0]);
        render_pass.draw(0..(MAX_LIVE_POINT_DENSITY * 6) as u32, 0..1);
    }
}
struct CustomStereometerCallback {
    pos: Vec<Pos2>,
    color: LinearRgba,
    live_density: LiveDensity,
}
impl egui_wgpu::CallbackTrait for CustomStereometerCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &StereometerRenderResources = resources.get().unwrap();
        resources.prepare(
            device,
            queue,
            self.pos.clone(),
            self.color,
            self.live_density,
        );
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &StereometerRenderResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}

impl PolarityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::apply_theme(&cc.egui_ctx);
        let wgpu_render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("not using wgpu backend");

        let device = &wgpu_render_state.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("stereometer"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./stereometer_shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("stereometer"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(16),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: NonZeroU64::new(32),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("stereometer"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("stereometer"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu_render_state.target_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&vec![[0f32; 4]; MAX_LIVE_POINT_DENSITY * 4]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        });
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(&[0u32; 8]), // 32 bytes aligned
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("stereometer"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(StereometerRenderResources {
                pipeline,
                bind_group,
                vertex_buffer,
                uniform_buffer,
            });

        Self::default()
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let mut canvas_size = ui.available_size();
        let center = pos2(ui.available_width() / 2.0, ui.available_height() / 2.0);
        canvas_size.x = canvas_size.x.clamp(0.0, canvas_size.y);
        canvas_size.y = canvas_size.y.clamp(0.0, canvas_size.x);
        let rect = ui
            .allocate_rect(
                egui::Rect::from_min_size(
                    pos2(
                        center.x - canvas_size.x / 2.0,
                        center.y - canvas_size.y / 2.0,
                    ),
                    canvas_size,
                ),
                egui::Sense::click(),
            )
            .rect;
        let Some(player) = &self.player else {
            return;
        };

        let pos = self.st.stereo.draw(player, canvas_size);
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            CustomStereometerCallback {
                pos,
                live_density: self.st.stereo.live_density,
                color: self.st.stereo.fs_color.into(),
            },
        ));
    }

    pub fn load_file(&mut self, path: PathBuf) {
        if let Some(old_player) = &self.player {
            if *old_player.contents.path != path {
                println!("Diff");
                self.spawn_audio_player(path, true);
            }
        } else {
            self.spawn_audio_player(path, true);
        }
    }

    pub fn spawn_audio_player(&mut self, path: PathBuf, paused: bool) {
        // Clear old player
        self.player.take();

        self.player = AudioPlayer::new(path, paused)
            .inspect_err(|err| println!("{}", err))
            .ok();
    }

    pub fn handle_playback(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(Key::Space)) {
            if let Some(player) = &mut self.player {
                player.toggle_playback();
            } else {
                println!("No audio player loaded");
            }
        }
    }

    pub fn update_filters(&mut self) {
        if self.st.stereo.live_fs_filters.is_none()
            && let Some(p) = &self.player
        {
            self.st.set_default_freqs = true;
            let st = &mut self.st.stereo;
            let filters = Some((
                StereoFilter::from_coeffs_butterworth(Type::LowPass, 200., p.contents.sample_rate),
                StereoFilter::from_coeffs_butterworth(
                    Type::BandPass,
                    1000.,
                    p.contents.sample_rate,
                ),
                StereoFilter::from_coeffs_butterworth(
                    Type::HighPass,
                    5000.,
                    p.contents.sample_rate,
                ),
            ));
            st.live_fs_filters = filters.clone();
            st.trace_fs_filters = filters.clone();
            st.live_mb_filters = filters.clone();
            st.trace_mb_filters = filters;
        }

        if self.st.stereo.live_fs_filters.is_some()
            && let Some(p) = &self.player
            && self.st.stereo.last_freq != self.st.stereo.filter_freq
        {
            self.st.set_default_freqs = false;
            let st = &mut self.st.stereo;
            st.last_freq = st.filter_freq;
            let livefs = st.live_fs_filters.as_mut().unwrap();
            let tracefs = st.trace_fs_filters.as_mut().unwrap();
            match st.filter_mode {
                FilterMode::Off => (),
                FilterMode::Lpf => {
                    livefs.0 = StereoFilter::from_coeffs_butterworth(
                        Type::LowPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                    tracefs.0 = StereoFilter::from_coeffs_butterworth(
                        Type::LowPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                }
                FilterMode::Bpf => {
                    livefs.1 = StereoFilter::from_coeffs_butterworth(
                        Type::BandPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                    tracefs.1 = StereoFilter::from_coeffs_butterworth(
                        Type::BandPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                }
                FilterMode::Hpf => {
                    livefs.2 = StereoFilter::from_coeffs_butterworth(
                        Type::HighPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                    tracefs.2 = StereoFilter::from_coeffs_butterworth(
                        Type::HighPass,
                        st.filter_freq,
                        p.contents.sample_rate,
                    );
                }
            }
        }
    }
}

impl eframe::App for PolarityApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let resp = egui::CentralPanel::default()
            .frame(egui::Frame::NONE.inner_margin(12.0).fill(plt::VOID))
            .show_inside(ui, |ui| {
                timeline::draw(ui, &mut self.st, &mut self.player);
                control_panel::draw(ui, &mut self.st);
                // canvas::draw(ui, &mut self.st, &self.player);
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(plt::VOID))
                    .show_inside(ui, |ui| {
                        self.custom_painting(ui);
                    });
            });

        let border_rect = resp.response.rect.shrink(12.0).round_ui();
        ui.painter().rect_stroke(
            border_rect,
            CornerRadius::ZERO,
            Stroke {
                width: 1.0,
                color: plt::BORDER,
            },
            StrokeKind::Inside,
        );
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(p) = &self.player
            && !p.is_paused()
        {
            ctx.request_repaint_after_secs(Duration::from_millis(16).as_secs_f32());
        }

        // Open file dialog
        if self.st.import_open {
            self.st.file_dialog.pick_file();
            self.st.import_open = false;
        }

        // Check if user picked a file
        if let Some(path) = self
            .st
            .file_dialog
            .update(ctx)
            .picked()
            .map(|p| p.to_path_buf())
        {
            self.load_file(path);
        };
        self.update_filters();
        self.handle_playback(ctx);

        // Check if playback has ended
        if let Some(player) = &self.player
            && player.ended()
        {
            println!("Respawning player");
            let paused = matches!(self.st.playback_mode, PlaybackMode::Once);
            self.spawn_audio_player(player.contents.path.to_path_buf(), paused);
        }
    }
}
