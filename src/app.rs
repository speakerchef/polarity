use biquad::*;
use eframe::egui_wgpu::{self, wgpu};
use egui_winit::winit::dpi::LogicalSize;
use std::{
    num::NonZeroU64,
    ops::Add,
    path::PathBuf,
    time::{Duration, Instant},
};
use wgpu::{Device, util::DeviceExt};

use crate::{
    audio::{StereoFilter, audio_player::*},
    generators::rendering::{
        BlitRenderResources, BloomRenderResources, OutputResources, StereometerRenderResources,
    },
    generators::stereometer::{
        FilterMode, MAX_LIVE_POINT_DENSITY, MAX_TRACE_POINT_DENSITY, VERTICES_PER_QUAD,
    },
    state::PlaybackMode,
    ui::{
        canvas, control_panel,
        control_panel_widgets::menu_bar_option,
        custom_text, timeline,
        timeline_widgets::{SHARP, border},
    },
};
use egui::{Align, FontId, Key, StrokeKind, pos2, vec2};

use crate::{state::AppState, ui::palette as plt, ui::theme};

#[derive(Default)]
pub struct PolarityApp {
    st: AppState,
    player: Option<AudioPlayer>,
}
const MB_H: f32 = 18.0;
const MB_GAP: f32 = 12.0;

impl eframe::App for PolarityApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if !self.st.fullscreen {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.set_min_height(MB_H + MB_GAP);
                egui::Area::new("menu_bar".into())
                    .fixed_pos(ui.viewport_rect().left_top() + vec2(0.0, MB_GAP))
                    .order(egui::Order::Foreground)
                    .movable(false)
                    .show(ui.ctx(), |ui| {
                        ui.set_max_height(MB_H);
                        ui.set_width(ui.content_rect().width());

                        ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                            let resp = ui.allocate_rect(
                                egui::Rect::from_min_size(
                                    pos2(
                                        ui.content_rect().left_top().x,
                                        ui.content_rect().left_top().y + 12.0,
                                    ),
                                    vec2(ui.available_width(), ui.available_height()),
                                ),
                                egui::Sense::focusable_noninteractive(),
                            );
                            let mut rect = resp.rect;
                            rect.min.y -= MB_GAP;
                            ui.painter().rect_filled(rect, SHARP, plt::BG);

                            ui.set_max_width(ui.available_rect_before_wrap().width());
                            ui.set_min_width(ui.available_rect_before_wrap().width());
                            ui.add_space(12.0);

                            menu_bar_option(
                                ui,
                                "file",
                                44.0,
                                FontId {
                                    family: egui::FontFamily::Name("inter_medium".into()),
                                    size: plt::font_size::TINY,
                                },
                                &mut self.st.show_file_options,
                                &["Import", "Export"],
                                &mut [&mut self.st.import_open, &mut false],
                                MB_H,
                            );
                            ui.add_space(1.0);

                            menu_bar_option(
                                ui,
                                "window",
                                70.0,
                                FontId {
                                    family: egui::FontFamily::Name("inter_medium".into()),
                                    size: plt::font_size::TINY,
                                },
                                &mut self.st.show_window_options,
                                &[""],
                                &mut [&mut self.st.window_options_open, &mut false],
                                MB_H,
                            );

                            ui.add_space(ui.available_width() - 36.0);
                        });
                    });
            });
        }
        let mut resp = egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    // .inner_margin(if !self.st.fullscreen { 12.0 } else { 0.0 })
                    .inner_margin(if !self.st.fullscreen {
                        egui::Margin {
                            bottom: 12,
                            left: 12,
                            right: 12,
                            top: 0,
                        }
                    } else {
                        0.into()
                    })
                    .fill(plt::BG),
            )
            .show_inside(ui, |ui| {
                if !self.st.fullscreen {
                    timeline::draw(ui, &mut self.st, &mut self.player);
                    control_panel::draw(ui, &mut self.st);

                    frame.winit_window().unwrap().set_decorations(true);
                    frame
                        .winit_window()
                        .unwrap()
                        .set_min_inner_size(Some(LogicalSize::new(720.0, 480.0)));
                } else {
                    ui.request_repaint_after(Duration::from_millis(16));
                    self.show_window_drag_tooltip_modal(ui);

                    // remove titlebar and corners
                    frame.winit_window().unwrap().set_decorations(false);
                    frame
                        .winit_window()
                        .unwrap()
                        .set_min_inner_size(Some(LogicalSize::new(240.0, 240.0)));

                    // allow drag anywhere if any key is down
                    if ui.ctx().input(|i| {
                        (!i.keys_down.is_empty() && !i.key_down(Key::Space))
                            || (i.modifiers.matches_logically(egui::Modifiers::COMMAND)
                                || i.modifiers.matches_logically(egui::Modifiers::SHIFT))
                    }) {
                        frame
                            .winit_window()
                            .unwrap()
                            .drag_window()
                            .unwrap_or_default();
                    }
                }
                canvas::draw(ui, &mut self.st, &self.player);
            })
            .response;

        if !self.st.fullscreen {
            resp.rect = resp.rect.translate(vec2(0., -6.));
            resp.rect = resp.rect.shrink2(vec2(12.0, 6.0));
            ui.painter()
                .rect_stroke(resp.rect, SHARP, border(), StrokeKind::Inside);
        }
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

fn init_stereometer_render_resources(device: &Device, wgpu_render_state: &egui_wgpu::RenderState) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("stereometer"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./stereometer_shader.wgsl").into()),
    });

    let stereometer_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        min_binding_size: NonZeroU64::new(144),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(16),
                    },
                    count: None,
                },
            ],
        });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("stereometer"),
        bind_group_layouts: &[Some(&stereometer_bind_group_layout)],
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
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu_render_state.target_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("live buffer"),
        contents: bytemuck::cast_slice(
            &[[0f32; 2];
                (MAX_LIVE_POINT_DENSITY + MAX_TRACE_POINT_DENSITY) * VERTICES_PER_QUAD * 3],
        ),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
    });
    let alpha_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("alpha buffer"),
        contents: bytemuck::cast_slice(&[0f32; MAX_TRACE_POINT_DENSITY * VERTICES_PER_QUAD]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
    });
    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("uniform buffer"),
        contents: bytemuck::cast_slice(&[0f32; 36]), // 144 bytes aligned
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    });

    let stereometer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("stereometer"),
        layout: &stereometer_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: vertex_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: alpha_buffer.as_entire_binding(),
            },
        ],
    });

    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(StereometerRenderResources {
            target_format: wgpu_render_state.target_format,
            pipeline,
            bind_group: stereometer_bind_group,
            vertex_buffer,
            params_buffer,
            alpha_buffer,
            tex: None,
        });
}
fn init_blit_render_resources(device: &Device, wgpu_render_state: &egui_wgpu::RenderState) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("blit"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./blit_shader.wgsl").into()),
    });

    let blit_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blit"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("blit"),
        bind_group_layouts: &[Some(&blit_bind_group_layout)],
        immediate_size: 0,
    });

    let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("blit"),
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
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu_render_state.target_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let blit_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("blit sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });

    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(BlitRenderResources {
            pipeline: blit_pipeline,
            bind_group_layout: blit_bind_group_layout,
            sampler: blit_sampler,
        });
}
fn init_bloom_render_resources(device: &Device, wgpu_render_state: &egui_wgpu::RenderState) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("bloom"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./bloom_shader.wgsl").into()),
    });

    let bloom_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(32),
                    },
                    count: None,
                },
            ],
        });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("bloom"),
        bind_group_layouts: &[Some(&bloom_bind_group_layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("bloom"),
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
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu_render_state.target_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let bloom_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("bloom sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });
    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("uniform buffer"),
        contents: bytemuck::cast_slice(&[0f32; 8]), // 32 bytes aligned
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    });

    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(BloomRenderResources {
            pipeline,
            bind_group_layout: bloom_bind_group_layout,
            sampler: bloom_sampler,
            bind_group: None,
            params_buffer,
        });
}

fn init_output_render_resources(device: &Device, wgpu_render_state: &egui_wgpu::RenderState) {
    let output_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("uniform buffer"),
        contents: bytemuck::cast_slice(&[0u32; 1440 * 1440]), // 32 bytes aligned
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    });
    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(OutputResources {
            output_buffer,
            target_format: wgpu_render_state.target_format,
            tex: None,
        });
}

fn setup_wgpu(cc: &eframe::CreationContext<'_>) {
    let wgpu_render_state = cc
        .wgpu_render_state
        .as_ref()
        .expect("not using wgpu backend");
    let device = &wgpu_render_state.device;

    init_stereometer_render_resources(device, wgpu_render_state);
    init_blit_render_resources(device, wgpu_render_state);
    init_bloom_render_resources(device, wgpu_render_state);
    init_output_render_resources(device, wgpu_render_state);
}

impl PolarityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::apply_theme(&cc.egui_ctx);
        setup_wgpu(cc);
        Self::default()
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
        self.st.stereo.clear_live_buffers();
        self.st.stereo.clear_trace_buffers();

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
                StereoFilter::from_coeffs_butterworth(Type::LowPass, 300., p.contents.sample_rate),
                StereoFilter::from_coeffs_butterworth(
                    Type::BandPass,
                    1000.,
                    p.contents.sample_rate,
                ),
                StereoFilter::from_coeffs_butterworth(
                    Type::HighPass,
                    3000.,
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

    fn show_window_drag_tooltip_modal(&mut self, ui: &mut egui::Ui) {
        if self.st.window_drag_tooltip_modal_deadline.is_none() {
            self.st.window_drag_tooltip_modal_deadline = Some(Instant::now());
            self.st.window_drag_tooltip_modal_open = true;
        }

        if self.st.window_drag_tooltip_modal_open {
            egui::Area::new("window drag tooltip".into())
                .order(egui::Order::Background)
                .show(ui.ctx(), |ui| {
                    let mut resp = ui.allocate_rect(
                        egui::Rect::from_min_size(
                            ui.content_rect().left_top().add(vec2(48., 14.)),
                            vec2(360., 30.),
                        ),
                        egui::Sense::click(),
                    );
                    resp.interact_rect.set_height(0.0);
                    resp.interact_rect.set_width(0.0);

                    custom_text(
                        ui,
                        "PRESS AND HOLD ANY KEY TO MOVE THE WINDOW",
                        FontId {
                            size: plt::font_size::META,
                            family: egui::FontFamily::Name("inter_regular".into()),
                        },
                        pos2(resp.rect.left(), resp.rect.left_center().y - 9.0),
                        plt::letter_spacing::BASE,
                        plt::BORDER,
                        Align::LEFT,
                    );
                });
        }

        if let Some(start_time) = self.st.window_drag_tooltip_modal_deadline
            && Instant::now().duration_since(start_time) >= Duration::from_secs(5)
        {
            self.st.window_drag_tooltip_modal_open = false;
        }
    }
}
