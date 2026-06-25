use std::num::NonZeroU64;

use eframe::egui_wgpu;
use wgpu::{Device, util::DeviceExt};

use crate::{
    generators::{
        rendering::{
            BloomRenderResources, FluidRenderResources, OutputResources, StereometerRenderResources,
        },
        stereometer::{MAX_LIVE_POINT_DENSITY, MAX_TRACE_POINT_DENSITY, VERTICES_PER_QUAD},
    },
    state::AppState,
    ui::canvas::{NUM_PARTICLES, generate_particle_grid, generate_rand_particles},
};

fn build_stereometer_render_resources(
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) -> StereometerRenderResources {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("stereometer"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/stereometer_shader.wgsl").into()),
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

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("live buffer"),
        size: ((size_of::<f32>() * 2)
            * (MAX_LIVE_POINT_DENSITY + MAX_TRACE_POINT_DENSITY)
            * (VERTICES_PER_QUAD * 3)) as u64,
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
        size: (size_of::<f32>() * 36) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: false,
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
    st.resources.insert(export_res);
    // st.stereometer_render_resources = Some(export_res);
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
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/bloom_shader.wgsl").into()),
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
    st.resources.insert(export_res);
    // st.bloom_render_resources = Some(export_res);
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
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/output_shader.wgsl").into()),
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
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("output buffer"),
        size: (2160 * 2160 * size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
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

    let num_params = 7;

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
                visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX,
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
                visibility: wgpu::ShaderStages::COMPUTE,
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
            // Debug
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(16),
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
    // println!("{:?}", POS.as_flattened());
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
        size: (size_of::<[f32; 2]>() * (4096) * (4096)) as u64,
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
    let debug_storage = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("debug storage"),
        size: 64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let debug_staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("debug staging"),
        size: 64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
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
                binding: 3,
                resource: debug_storage.as_entire_binding(),
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
        tex: None,
        render_bind_group,
        compute_bind_group,
        vertex_buffer,
        params_buffer,
        debug_storage,
        debug_staging,
    }
}

fn init_output_render_resources(
    st: &mut AppState,
    device: &Device,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let live_res = build_output_render_resources(device, wgpu_render_state);
    let export_res = build_output_render_resources(device, wgpu_render_state);
    st.resources.insert(export_res);
    // st.output_render_resources = Some(export_res);
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

    init_stereometer_render_resources(st, device, wgpu_render_state);
    init_bloom_render_resources(st, device, wgpu_render_state);
    init_output_render_resources(st, device, wgpu_render_state);
    init_fluid_render_resources(st, device, wgpu_render_state);
}
