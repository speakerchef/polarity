use biquad::*;
use eframe::egui_wgpu::{self, wgpu};
use egui_winit::winit::dpi::LogicalSize;
use std::{
    io::Write,
    num::NonZeroU64,
    ops::Add,
    path::PathBuf,
    process::Stdio,
    time::{Duration, Instant},
};
use wgpu::{Device, util::DeviceExt};

use crate::{
    audio::{StereoFilter, audio_player::*},
    generators::{
        rendering::{
            BloomRenderResources, OutputResources, RendererCallback, StereometerRenderResources,
            effects_render_pipeline, get_gpu_frame, main_render_pipeline, output_render_pipeline,
            prep_bloom_resources_for_effects, prep_meter_resources_for_effects,
            prep_output_resources_for_effects,
        },
        stereometer::{
            FilterMode, MAX_LIVE_POINT_DENSITY, MAX_TRACE_POINT_DENSITY, VERTICES_PER_QUAD,
        },
    },
    state::PlaybackMode,
    ui::{
        app_widgets::export_modal,
        canvas, control_panel,
        control_panel_widgets::menu_bar_option,
        custom_text, timeline,
        timeline_widgets::{SHARP, border},
    },
};
use eframe::egui;
use eframe::egui::{Align, FontId, Key, StrokeKind, pos2, vec2};

use crate::{state::AppState, ui::palette as plt, ui::theme};

#[derive(Default)]
pub struct PolarityApp {
    st: AppState,
    player: Option<AudioPlayer>,
}
const MB_H: f32 = 20.0;
const MB_GAP: f32 = 12.0;

const BATCH_SIZE: usize = 200;

fn export_batched_frames(
    st: &mut AppState,
    p: &AudioPlayer,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let canvas_size = st.export_config.resolution.resolution();
    let (w, h) = (canvas_size.0, canvas_size.1);
    let fps = st.export_config.frame_rate.fps();
    let quality = st.export_config.quality.quality();

    let bloom_amt = st.bloom;
    let device = &wgpu_render_state.device;
    let queue = &wgpu_render_state.queue;

    let meter_res = st.stereometer_render_resources.as_mut().unwrap();
    let bloom_res = st.bloom_render_resources.as_mut().unwrap();
    let out_res = st.output_render_resources.as_mut().unwrap();
    if st.join_handle.is_none() {
        let mut cmd = std::process::Command::new("ffmpeg");
        let mut cmd = cmd
            .args([
                "-f",
                "rawvideo",
                "-pixel_format",
                "bgra",
                "-video_size",
                &format!("{w}x{h}"),
                "-framerate",
                &fps.to_string(),
                "-i",
                "-",
                "-i",
                p.contents.path.to_str().unwrap(),
                "-shortest",
                "-c:v",
                "libx264",
                "-crf",
                &quality.to_string(),
                "-preset",
                "veryfast",
                "-pix_fmt",
                "yuv444p",
                "-c:a",
                "aac",
                "-b:a",
                "320k",
                "-y",
                "out.mp4",
            ])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = cmd.stdin.take().expect("No stdin");
        let (tx, rx) = flume::bounded::<Vec<u8>>(4);
        let handle = std::thread::spawn(move || {
            rx.iter().for_each(|frame| {
                stdin.write_all(&frame).unwrap();
            });
            drop(stdin);
            cmd.wait().unwrap();
        });
        st.join_handle = Some(handle);
        st.export_tx = Some(tx);
    }

    let total_frames = p.contents.duration.as_secs_f32() * fps as f32;

    for _ in 0..BATCH_SIZE {
        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("export command encoder"),
        });

        st.cur_frame_idx += 1;
        if st.cur_frame_idx >= total_frames as usize {
            drop(std::mem::take(&mut st.export_tx));
            st.join_handle.take().unwrap().join().unwrap();
            st.rendering = false;
            st.cur_frame_idx = 0;
            println!("Finished");
            break;
        }
        let frac = st.cur_frame_idx as f32 / fps as f32;
        let export_sample_idx = (frac * p.contents.sample_rate as f32) as usize;

        st.stereo.draw(p, Some(export_sample_idx));

        let render_data = RendererCallback {
            render_mode: st.stereo.render_mode,
            live_pos: std::mem::take(&mut st.stereo.live_buffer),
            trace_pos: st.stereo.trace_buffer.clone().into(),

            live_low_pos: std::mem::take(&mut st.stereo.live_low_buffer),
            live_mid_pos: std::mem::take(&mut st.stereo.live_mid_buffer),
            live_high_pos: std::mem::take(&mut st.stereo.live_high_buffer),
            trace_low_pos: st.stereo.trace_low_buffer.clone().into(),
            trace_mid_pos: st.stereo.trace_mid_buffer.clone().into(),
            trace_high_pos: st.stereo.trace_high_buffer.clone().into(),

            fs_color: st.stereo.fs_color.into(),
            lb_color: st.stereo.mb_color[0].into(),
            mb_color: st.stereo.mb_color[1].into(),
            hb_color: st.stereo.mb_color[2].into(),
            canvas_size: vec2(w as f32, h as f32),
        };

        // Main pipeline
        main_render_pipeline(
            &render_data,
            device,
            queue,
            (w, h),
            &mut command_encoder,
            meter_res,
        );

        // Effects pipeline
        let (tex_size, bloom_bind_group) =
            prep_meter_resources_for_effects(device, meter_res, bloom_res);

        let dst_view = prep_output_resources_for_effects(device, tex_size, out_res);
        prep_bloom_resources_for_effects(device, bloom_res, queue, bloom_bind_group, bloom_amt);
        effects_render_pipeline(device, &mut command_encoder, dst_view, bloom_res);

        // Output
        output_render_pipeline(&mut command_encoder, out_res);
        queue.submit(Some(command_encoder.finish()));
        let frame = get_gpu_frame(device, out_res);
        st.export_tx.as_ref().unwrap().send(frame).unwrap();
    }
}

impl eframe::App for PolarityApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if !self.st.fullscreen {
            self.render_menu_bar(ui);
        }

        let mut resp = egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
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
                                || i.modifiers.matches_logically(egui::Modifiers::SHIFT)
                                || i.modifiers.matches_logically(egui::Modifiers::ALT))
                    }) {
                        frame
                            .winit_window()
                            .unwrap()
                            .drag_window()
                            .unwrap_or_default();
                    }
                }

                if self.st.show_export_modal {
                    export_modal(ui, &mut self.st);
                }

                if self.st.rendering {
                    let p = self.player.as_ref().unwrap();
                    p.pause();
                } else {
                    canvas::draw(ui, &mut self.st, &self.player);
                }
            })
            .response;

        if !self.st.fullscreen {
            resp.rect = resp.rect.translate(vec2(0., -6.));
            resp.rect = resp.rect.shrink2(vec2(12.0, 6.0));
            ui.painter()
                .rect_stroke(resp.rect, SHARP, border(), StrokeKind::Inside);
        }
    }
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(p) = &self.player
            && !p.is_paused()
        {
            ctx.request_repaint_after_secs(Duration::from_millis(16).as_secs_f32());
        }
        //EXPORT
        if let Some(p) = &self.player
            && (self.st.start_render || self.st.rendering)
        {
            self.st.start_render = false;
            self.st.rendering = true;
            let wgpu_render_state = frame.wgpu_render_state().unwrap();
            export_batched_frames(&mut self.st, p, wgpu_render_state);
            ctx.request_repaint();
            self.st.show_export_modal = false;
        }

        // Open file dialog
        if self.st.import_open {
            self.st.file_dialog.pick_file();
            self.st.import_open = false;
            self.st.start_render = false;
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

fn build_stereometer_render_resources(
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) -> StereometerRenderResources {
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
    StereometerRenderResources {
        target_format: wgpu_render_state.target_format,
        pipeline,
        bind_group: stereometer_bind_group,
        vertex_buffer,
        params_buffer,
        alpha_buffer,
        tex: None,
    }
}

fn init_stereometer_render_resources(
    st: &mut AppState,
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let live_res = build_stereometer_render_resources(device, wgpu_render_state);
    let export_res = build_stereometer_render_resources(device, wgpu_render_state);
    st.stereometer_render_resources = Some(export_res);
    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(live_res);
}

fn build_bloom_render_resources(
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) -> BloomRenderResources {
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
                        min_binding_size: NonZeroU64::new(16),
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
        contents: bytemuck::cast_slice(&[0f32; 4]), // 16 bytes aligned
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    });

    BloomRenderResources {
        pipeline,
        bind_group_layout: bloom_bind_group_layout,
        sampler: bloom_sampler,
        bind_group: None,
        params_buffer,
    }
}

fn init_bloom_render_resources(
    st: &mut AppState,
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let live_res = build_bloom_render_resources(device, wgpu_render_state);
    let export_res = build_bloom_render_resources(device, wgpu_render_state);
    st.bloom_render_resources = Some(export_res);
    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(live_res);
}

fn build_output_render_resources(
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) -> OutputResources {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("output"),
        source: wgpu::ShaderSource::Wgsl(include_str!("./output_shader.wgsl").into()),
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("output"),
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
                    min_binding_size: NonZeroU64::new(16),
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("output"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("output"),
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

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("output sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });
    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("uniform buffer"),
        contents: bytemuck::cast_slice(&[0f32; 4]), // 16 bytes aligned
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
    });
    let output_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("output buffer"),
        contents: bytemuck::cast_slice(&[0u32; 4096 * 4096]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    });
    OutputResources {
        pipeline,
        tex: None,
        sampler,
        bind_group: None,
        bind_group_layout: bgl,
        params_buffer,
        output_buffer,
        target_format: wgpu_render_state.target_format,
    }
}

fn init_output_render_resources(
    st: &mut AppState,
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let live_res = build_output_render_resources(device, wgpu_render_state);
    let export_res = build_output_render_resources(device, wgpu_render_state);
    st.output_render_resources = Some(export_res);
    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(live_res);
}

fn setup_wgpu(st: &mut AppState, cc: &eframe::CreationContext<'_>) {
    let wgpu_render_state = cc
        .wgpu_render_state
        .as_ref()
        .expect("not using wgpu backend");
    let device = &wgpu_render_state.device;

    init_stereometer_render_resources(st, device, wgpu_render_state);
    init_bloom_render_resources(st, device, wgpu_render_state);
    init_output_render_resources(st, device, wgpu_render_state);
}

impl PolarityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::apply_theme(&cc.egui_ctx);
        let mut st = AppState::default();
        setup_wgpu(&mut st, cc);
        Self {
            st,
            ..Default::default()
        }
    }
    fn render_menu_bar(&mut self, ui: &mut egui::Ui) {
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
                            &mut [&mut self.st.import_open, &mut self.st.show_export_modal],
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
