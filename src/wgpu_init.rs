use std::num::NonZeroU64;

use eframe::egui_wgpu;
use wgpu::{Device, util::DeviceExt};

use crate::{
    generators::{
        rendering::{
            EffectsRenderResources, FluidRenderResources, OutputResources, P2DRenderResources,
            SrcRenderResources,
        },
        stereometer::{MAX_LIVE_POINT_DENSITY, MAX_TRACE_POINT_DENSITY, VERTICES_PER_QUAD},
    },
    state::AppState,
    ui::canvas::{NUM_PARTICLES, generate_particle_grid},
};

fn init_src_render_resources(st: &mut AppState, wgpu_render_state: &egui_wgpu::RenderState) {
    let live_res = SrcRenderResources {
        target_format: wgpu_render_state.target_format,
        tex: None,
    };
    let export_res = SrcRenderResources {
        target_format: wgpu_render_state.target_format,
        tex: None,
    };
    st.resources.insert(export_res);
    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(live_res);
}
fn build_particle2d_render_resources(
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) -> P2DRenderResources {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("stereometer"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particle2d_shader.wgsl").into()),
    });

    let num_params = 11;
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("particle 2d"),
        entries: &[
            // sample positions
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
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: NonZeroU64::new(16 * num_params),
                },
                count: None,
            },
            // alphas
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
        label: Some("particle 2d"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let src_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("particle 2d"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
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

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("live buffer"),
        size: ((size_of::<f32>() * 2)
            * (MAX_LIVE_POINT_DENSITY + MAX_TRACE_POINT_DENSITY)
            * (VERTICES_PER_QUAD * 32)) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let alpha_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("alpha buffer"),
        size: (size_of::<f32>() * MAX_TRACE_POINT_DENSITY * VERTICES_PER_QUAD) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniform buffer"),
        size: 16 * num_params,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("particle 2d"),
        layout: &bind_group_layout,
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
    P2DRenderResources {
        src_pipeline,
        bind_group,
        vertex_buffer,
        params_buffer,
        alpha_buffer,
    }
}

fn init_particle2d_render_resources(
    st: &mut AppState,
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let live_res = build_particle2d_render_resources(device, wgpu_render_state);
    let export_res = build_particle2d_render_resources(device, wgpu_render_state);
    st.resources.insert(export_res);
    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(live_res);
}

fn build_efx_render_resources(
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) -> EffectsRenderResources {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("efx"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/postfx_shader.wgsl").into()),
    });
    let num_params = 8;

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("efx bind group layout"),
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
                    min_binding_size: NonZeroU64::new(16 * num_params),
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("efx"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let chroma_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("chroma pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: None,
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("chromatic_aberration"),
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
    let bloom_horizontal_pipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("horizontal_bloom_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: None,
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("bloom_horizontal"),
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
    let main_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("main efx"),
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
        label: Some("efx sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });
    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniform buffer"),
        size: 16 * num_params,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });

    EffectsRenderResources {
        main_pipeline,
        chroma_pipeline,
        bloom_horizontal_pipeline,
        chroma_tex: None,
        bloom_horizontal_tex: None,
        target_format: wgpu_render_state.target_format,
        bind_group_layout,
        sampler,
        chroma_bg: None,
        bloom_horizontal_bg: None,
        main_bind_group: None,
        params_buffer,
    }
}

fn init_effects_render_resources(
    st: &mut AppState,
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let live_res = build_efx_render_resources(device, wgpu_render_state);
    let export_res = build_efx_render_resources(device, wgpu_render_state);
    st.resources.insert(export_res);
    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(live_res);
}

fn build_output_render_resources(
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
    full_output_buf: bool,
) -> OutputResources {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("output"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/output_shader.wgsl").into()),
    });

    let num_params = 6;
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
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(16 * num_params),
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

    let out_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("output pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
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
    let meter_pipe = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("meter pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("render_meter"),
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
    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniform buffer"),
        size: 16 * num_params, // 16 bytes aligned
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("output buffer"),
        size: if full_output_buf {
            (4320 * 4320 * size_of::<u32>()) as u64
        } else {
            16
        },
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    OutputResources {
        output_pipeline: out_pipe,
        meter_pipeline: meter_pipe,
        tex: None,
        sampler,
        bind_group: None,
        bind_group_layout: bgl,
        params_buffer,
        output_buffer,
        target_format: wgpu_render_state.target_format,
    }
}

fn build_fluid_render_resources(
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) -> FluidRenderResources {
    let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fluid render shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/fluid_render.wgsl").into()),
    });
    let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fluid compute shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/fluid_compute.wgsl").into()),
    });

    let num_params = 20;

    let render_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fluid render bgl"),
        entries: &[
            // positions
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(8),
                },
                count: None,
            },
            // velocities
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(16),
                },
                count: None,
            },
            // Params
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(16 * num_params),
                },
                count: None,
            },
        ],
    });
    let compute_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("fluid compute bgl"),
        entries: &[
            // positions
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(8),
                },
                count: None,
            },
            // velocities
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(16),
                },
                count: None,
            },
            // Params
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: NonZeroU64::new(16 * num_params),
                },
                count: None,
            },
            // Densities
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(16),
                },
                count: None,
            },
            // Predicted positions
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(16),
                },
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("fluid render pipeline"),
        bind_group_layouts: &[Some(&render_bgl)],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("fluid"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &render_shader,
            entry_point: None,
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &render_shader,
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

    let compute_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("fluid compute pipeline"),
        bind_group_layouts: &[Some(&compute_bgl)],
        immediate_size: 0,
    });
    let compute = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("fluid compute"),
        layout: Some(&compute_layout),
        module: &compute_shader,
        entry_point: Some("cs_main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let density = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("fluid density pipeline"),
        layout: Some(&compute_layout),
        module: &compute_shader,
        entry_point: Some("cs_calculate_densities"),
        compilation_options: Default::default(),
        cache: None,
    });
    let pressure = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("fluid pressure pipeline"),
        layout: Some(&compute_layout),
        module: &compute_shader,
        entry_point: Some("cs_calculate_pressure"),
        compilation_options: Default::default(),
        cache: None,
    });
    let positions = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("fluid positions pipeline"),
        layout: Some(&compute_layout),
        module: &compute_shader,
        entry_point: Some("cs_calculate_predicted_positions"),
        compilation_options: Default::default(),
        cache: None,
    });
    let viscosity = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("fluid viscosity pipeline"),
        layout: Some(&compute_layout),
        module: &compute_shader,
        entry_point: Some("cs_calculate_viscosity"),
        compilation_options: Default::default(),
        cache: None,
    });
    static POS: [[f32; 8]; (NUM_PARTICLES * NUM_PARTICLES) as usize] = generate_particle_grid();
    let total_num_particles = (NUM_PARTICLES.pow(2) * 4) as usize;
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fluid vertex buffer"),
        contents: bytemuck::cast_slice(POS.as_flattened()),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
    });
    let density_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fluid density buffer"),
        contents: bytemuck::cast_slice(POS.as_flattened()),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
    });
    let pred_pos_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("fluid predicted positions buffer"),
        contents: bytemuck::cast_slice(POS.as_flattened()),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
    });
    let velocity_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fluid velocity buffer"),
        size: (size_of::<[f32; 2]>() * total_num_particles) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniform buffer"),
        size: num_params * 16, // 16 bytes aligned
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });
    let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fluid render bg"),
        layout: &render_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: vertex_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: velocity_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buffer.as_entire_binding(),
            },
        ],
    });
    let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fluid compute bg"),
        layout: &compute_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: vertex_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: velocity_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: density_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: pred_pos_buffer.as_entire_binding(),
            },
        ],
    });
    FluidRenderResources {
        pipeline,
        compute_pipeline: compute,
        density_pipeline: density,
        pressure_pipeline: pressure,
        viscosity_pipeline: viscosity,
        positions_pipeline: positions,
        render_bind_group,
        compute_bind_group,
        params_buffer,
    }
}

fn init_output_render_resources(
    st: &mut AppState,
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let live_res = build_output_render_resources(device, wgpu_render_state, false);
    let export_res = build_output_render_resources(device, wgpu_render_state, true);
    st.resources.insert(export_res);
    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(live_res);
}

fn init_fluid_render_resources(
    st: &mut AppState,
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let live_res = build_fluid_render_resources(device, wgpu_render_state);
    let export_res = build_fluid_render_resources(device, wgpu_render_state);
    st.resources.insert(export_res);
    wgpu_render_state
        .renderer
        .write()
        .callback_resources
        .insert(live_res);
}

pub fn setup_wgpu(st: &mut AppState, cc: &eframe::CreationContext<'_>) {
    let wgpu_render_state = cc
        .wgpu_render_state
        .as_ref()
        .expect("not using wgpu backend");
    let device = &wgpu_render_state.device;

    init_src_render_resources(st, wgpu_render_state);
    init_particle2d_render_resources(st, device, wgpu_render_state);
    init_effects_render_resources(st, device, wgpu_render_state);
    init_output_render_resources(st, device, wgpu_render_state);
    init_fluid_render_resources(st, device, wgpu_render_state);
}
