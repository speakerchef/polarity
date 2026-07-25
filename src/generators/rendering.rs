use std::collections::VecDeque;

use eframe::egui;
use eframe::egui::{Pos2, Vec2, vec2};
use eframe::egui_wgpu;
use pollster::FutureExt;
use wgpu::{BindGroupEntry, BindingResource};

use crate::generators::fluidwave::{
    ColorArrangement, ColorMode, EnergyTransferMode, ForceDirection,
};
use crate::generators::{ChromaType, TARGET_DT};
use crate::traits::Textured;
use crate::ui::canvas::NUM_PARTICLES;
use crate::{
    LinearRgba,
    generators::stereometer::{MAX_TRACE_POINT_DENSITY, ParticleRenderMode, VERTICES_PER_QUAD},
};

const CANVAS_BG: wgpu::Color = wgpu::Color::BLACK;

#[derive(Default, Clone)]
pub struct Particle2DCbParams {
    pub render_mode: ParticleRenderMode,
    pub point_size: f32,
    pub add_point_border: bool,

    pub live_pos: Vec<Pos2>,
    pub trace_pos: VecDeque<Pos2>,

    pub live_low: Vec<Pos2>,
    pub live_mid: Vec<Pos2>,
    pub live_high: Vec<Pos2>,
    pub trace_low: VecDeque<Pos2>,
    pub trace_mid: VecDeque<Pos2>,
    pub trace_high: VecDeque<Pos2>,

    pub fs_color: LinearRgba,
    pub lb_color: LinearRgba,
    pub mb_color: LinearRgba,
    pub hb_color: LinearRgba,
}

#[derive(Clone)]
pub struct FluidCbParams {
    pub color_mode: ColorMode,
    pub uniform_color: crate::Rgba,

    pub particle_pos: f32,
    pub frame_time_accumulator: f32,
    pub gravity: f32,
    pub pressure_multiplier: f32,
    pub target_density: f32,
    pub smoothing_radius: f32,
    pub edge_damping_factor: f32,
    pub near_pressure_multiplier: f32,
    pub viscosity_amount: f32,
    pub point_size: f32,
    pub energy_transfer_mode: EnergyTransferMode,
    pub force_direction: ForceDirection,
    pub color_arrangement: ColorArrangement,
    pub color_invert: bool,
    pub luminance_mode: bool,
    pub luminance_floor: f32,
    pub substeps: f32,
}

#[allow(clippy::large_enum_variant)]
pub enum GenCbParams {
    Particle2D(Particle2DCbParams),
    Fwave(FluidCbParams),
}

impl GenCbParams {
    fn prepare_resources(
        &self,
        res: &mut egui_wgpu::CallbackResources,
        queue: &wgpu::Queue,
        command_encoder: &mut wgpu::CommandEncoder,
    ) {
        let p2d_res: &P2DRenderResources = res.get().unwrap();
        let fluid_res: &FluidRenderResources = res.get().unwrap();
        match self {
            GenCbParams::Particle2D(p2d) => {
                p2d_res.prepare(queue, p2d);
            }
            GenCbParams::Fwave(fwave) => {
                fluid_res.prepare(queue, fwave);
                let mut compute_pass =
                    command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("fluid compute pass"),
                        timestamp_writes: None,
                    });

                // maintain sim behavior across diff fps
                let mut accum = fwave.frame_time_accumulator;
                while accum >= TARGET_DT {
                    fluid_res.compute(&mut compute_pass);
                    accum -= TARGET_DT;
                }
            }
        }
    }

    fn paint(
        &self,
        res: &mut egui_wgpu::CallbackResources,
        command_encoder: &mut wgpu::CommandEncoder,
        target_texture_view: wgpu::TextureView,
    ) {
        let stereo_res: &P2DRenderResources = res.get().unwrap();
        let fluid_res: &FluidRenderResources = res.get().unwrap();
        let mut pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("src pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CANVAS_BG),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        match self {
            GenCbParams::Particle2D(stereo) => {
                let n = match stereo.render_mode {
                    ParticleRenderMode::FullSpectrum => {
                        (stereo.live_pos.len() + stereo.trace_pos.len()) as u32
                    }
                    ParticleRenderMode::MultiBand => {
                        let live =
                            stereo.live_low.len() + stereo.live_mid.len() + stereo.live_high.len();
                        let trace = stereo.trace_low.len()
                            + stereo.trace_mid.len()
                            + stereo.trace_high.len();
                        (live + trace) as u32
                    }
                };
                stereo_res.paint(&mut pass, n);
            }
            GenCbParams::Fwave(_) => {
                fluid_res.paint(&mut pass, (NUM_PARTICLES * NUM_PARTICLES) as u32 * 4);
            }
        }
    }
}

pub struct SrcRenderResources {
    pub target_format: wgpu::TextureFormat,
    pub tex: Option<wgpu::Texture>,
}
impl Textured for SrcRenderResources {
    fn texture(&self) -> Option<&wgpu::Texture> {
        self.tex.as_ref()
    }
    fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }
    fn set_texture(&mut self, tex: wgpu::Texture) {
        self.tex = Some(tex);
    }
}

pub struct P2DRenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub params_buffer: wgpu::Buffer,
    pub alpha_buffer: wgpu::Buffer,
}
#[allow(clippy::too_many_arguments)]
impl P2DRenderResources {
    fn prepare(&self, queue: &wgpu::Queue, params: &Particle2DCbParams) {
        let (live_len, trace_len) = (params.live_pos.len(), params.trace_pos.len());
        let (live_mb_len, trace_mb_len) = (params.live_low.len(), params.trace_low.len());
        let pos = match params.render_mode {
            ParticleRenderMode::FullSpectrum => {
                let mut pos = Vec::<Pos2>::new();
                pos.extend(&params.live_pos);
                pos.extend(&params.trace_pos);
                pos
            }
            ParticleRenderMode::MultiBand => {
                let mut pos = Vec::<Pos2>::new();
                pos.extend(&params.live_low);
                pos.extend(&params.live_mid);
                pos.extend(&params.live_high);
                pos.extend(&params.trace_low);
                pos.extend(&params.trace_mid);
                pos.extend(&params.trace_high);
                pos
            }
        };

        let alphas: Vec<f32> = if matches!(params.render_mode, ParticleRenderMode::MultiBand) {
            (0..trace_mb_len)
                .map(|i| (i as f32 / MAX_TRACE_POINT_DENSITY as f32).powf(1.75))
                .collect()
        } else {
            (0..trace_len)
                .map(|i| (i as f32 / MAX_TRACE_POINT_DENSITY as f32).powf(1.75))
                .collect()
        };
        queue.write_buffer(&self.alpha_buffer, 0, bytemuck::cast_slice(&alphas));
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&pos));

        let fs_color = params.fs_color;
        let lb_color = params.lb_color;
        let mb_color = params.mb_color;
        let hb_color = params.hb_color;

        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[live_len]));
        queue.write_buffer(&self.params_buffer, 16, bytemuck::cast_slice(&[trace_len]));
        queue.write_buffer(
            &self.params_buffer,
            32,
            bytemuck::cast_slice(&[live_mb_len]),
        );
        queue.write_buffer(
            &self.params_buffer,
            48,
            bytemuck::cast_slice(&[trace_mb_len]),
        );
        queue.write_buffer(
            &self.params_buffer,
            64,
            bytemuck::cast_slice(&[fs_color.r, fs_color.g, fs_color.b, fs_color.a]),
        );
        queue.write_buffer(
            &self.params_buffer,
            80,
            bytemuck::cast_slice(&[lb_color.r, lb_color.g, lb_color.b, lb_color.a]),
        );
        queue.write_buffer(
            &self.params_buffer,
            96,
            bytemuck::cast_slice(&[mb_color.r, mb_color.g, mb_color.b, mb_color.a]),
        );
        queue.write_buffer(
            &self.params_buffer,
            112,
            bytemuck::cast_slice(&[hb_color.r, hb_color.g, hb_color.b, hb_color.a]),
        );
        queue.write_buffer(
            &self.params_buffer,
            128,
            bytemuck::cast_slice(&[
                matches!(params.render_mode, ParticleRenderMode::MultiBand) as u32
            ]),
        );
        queue.write_buffer(
            &self.params_buffer,
            144,
            bytemuck::cast_slice(&[params.point_size]),
        );
        queue.write_buffer(
            &self.params_buffer,
            160,
            bytemuck::cast_slice(&[params.add_point_border as u32]),
        );
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>, num_points: u32) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[0]);
        render_pass.draw(0..num_points * VERTICES_PER_QUAD as u32, 0..1);
    }
}
pub struct OutputResources {
    pub output_buffer: wgpu::Buffer,
    pub params_buffer: wgpu::Buffer,
    pub target_format: wgpu::TextureFormat,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: Option<wgpu::BindGroup>,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub tex: Option<wgpu::Texture>,
    pub sampler: wgpu::Sampler,
}
impl Textured for OutputResources {
    fn texture(&self) -> Option<&wgpu::Texture> {
        self.tex.as_ref()
    }
    fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }
    fn set_texture(&mut self, tex: wgpu::Texture) {
        self.tex = Some(tex);
    }
}
impl OutputResources {
    fn prepare(&self, queue: &wgpu::Queue, top_left: Pos2) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[top_left]));
    }
    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

fn prepare_output_resources(
    res: &mut egui_wgpu::CallbackResources,
    device: &wgpu::Device,
    tex_size: (u32, u32),
) -> wgpu::TextureView {
    let output_view = get_texture_view(
        res.get_mut::<OutputResources>().unwrap(),
        device,
        tex_size,
        true,
    );
    let out_res = res.get_mut::<OutputResources>().unwrap();
    let output_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("output bind group"),
        layout: &out_res.bind_group_layout,
        entries: &create_bg_entries(&output_view, &out_res.sampler, &out_res.params_buffer),
    });
    out_res.bind_group = Some(output_bind_group);
    output_view
}

pub struct FluidRenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub density_pipeline: wgpu::ComputePipeline,
    pub pressure_pipeline: wgpu::ComputePipeline,
    pub positions_pipeline: wgpu::ComputePipeline,
    pub viscosity_pipeline: wgpu::ComputePipeline,
    pub render_bind_group: wgpu::BindGroup,
    pub compute_bind_group: wgpu::BindGroup,
    pub params_buffer: wgpu::Buffer,
}

#[allow(clippy::too_many_arguments)]
impl FluidRenderResources {
    fn prepare(&self, queue: &wgpu::Queue, dat: &FluidCbParams) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[TARGET_DT]));
        queue.write_buffer(
            &self.params_buffer,
            16,
            bytemuck::cast_slice(&[dat.gravity]),
        );
        queue.write_buffer(
            &self.params_buffer,
            32,
            bytemuck::cast_slice(&[dat.pressure_multiplier]),
        );
        queue.write_buffer(
            &self.params_buffer,
            48,
            bytemuck::cast_slice(&[dat.target_density]),
        );
        queue.write_buffer(
            &self.params_buffer,
            64,
            bytemuck::cast_slice(&[dat.smoothing_radius]),
        );
        queue.write_buffer(
            &self.params_buffer,
            80,
            bytemuck::cast_slice(&[dat.near_pressure_multiplier]),
        );
        queue.write_buffer(
            &self.params_buffer,
            96,
            bytemuck::cast_slice(&[dat.viscosity_amount]),
        );
        queue.write_buffer(
            &self.params_buffer,
            112,
            bytemuck::cast_slice(&[dat.particle_pos]),
        );
        queue.write_buffer(
            &self.params_buffer,
            128,
            bytemuck::cast_slice(&[dat.point_size]),
        );
        queue.write_buffer(
            &self.params_buffer,
            144,
            bytemuck::cast_slice(&[matches!(dat.color_mode, ColorMode::VelocityGradient) as u32]),
        );

        let (r, g, b, a) = dat.uniform_color.as_tuple();
        let col = LinearRgba::from_u8rgb(r, g, b, a);
        queue.write_buffer(
            &self.params_buffer,
            160,
            bytemuck::cast_slice(&[col.r, col.g, col.b, col.a]),
        );
        queue.write_buffer(
            &self.params_buffer,
            176,
            bytemuck::cast_slice(&[
                matches!(dat.energy_transfer_mode, EnergyTransferMode::Obstacle) as u32,
            ]),
        );
        queue.write_buffer(
            &self.params_buffer,
            192,
            bytemuck::cast_slice(&[matches!(dat.force_direction, ForceDirection::Out) as u32]),
        );
        queue.write_buffer(
            &self.params_buffer,
            224,
            bytemuck::cast_slice(&[dat.edge_damping_factor]),
        );
        queue.write_buffer(
            &self.params_buffer,
            240,
            bytemuck::cast_slice(&[dat.color_invert as u32]),
        );
        queue.write_buffer(
            &self.params_buffer,
            256,
            bytemuck::cast_slice(&[dat.color_arrangement.to_value()]),
        );
        queue.write_buffer(
            &self.params_buffer,
            272,
            bytemuck::cast_slice(&[dat.luminance_mode as u32]),
        );
        queue.write_buffer(
            &self.params_buffer,
            288,
            bytemuck::cast_slice(&[dat.luminance_floor]),
        );
        queue.write_buffer(
            &self.params_buffer,
            304,
            bytemuck::cast_slice(&[dat.substeps]),
        );
    }
    fn compute(&self, compute_pass: &mut wgpu::ComputePass<'_>) {
        const WORKGROUP_SIZE: u32 = 256;
        let num_groups: u32 =
            ((NUM_PARTICLES * NUM_PARTICLES * 4) as f32 / WORKGROUP_SIZE as f32).ceil() as u32;
        compute_pass.set_pipeline(&self.positions_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[0]);
        compute_pass.dispatch_workgroups(num_groups, 1, 1);

        compute_pass.set_pipeline(&self.density_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[0]);
        compute_pass.dispatch_workgroups(num_groups, 1, 1);

        compute_pass.set_pipeline(&self.viscosity_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[0]);
        compute_pass.dispatch_workgroups(num_groups, 1, 1);

        compute_pass.set_pipeline(&self.pressure_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[0]);
        compute_pass.dispatch_workgroups(num_groups, 1, 1);

        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[0]);
        compute_pass.dispatch_workgroups(num_groups, 1, 1);
    }
    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>, num_points: u32) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.render_bind_group, &[]);
        render_pass.draw(0..8 * num_points, 0..1);
    }
}

pub struct RendererCallback {
    pub canvas_size: Vec2,
    pub params: GenCbParams,
}
pub fn run_source_render_pipeline(
    params: &GenCbParams,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    command_encoder: &mut wgpu::CommandEncoder,
    res: &mut egui_wgpu::CallbackResources,
    dim: (u32, u32),
) {
    params.prepare_resources(res, queue, command_encoder);
    let src_view = get_texture_view(
        res.get_mut::<SrcRenderResources>().unwrap(),
        device,
        dim,
        true,
    );
    params.paint(res, command_encoder, src_view);
}

fn get_texture_view<T: Textured>(
    res: &mut T,
    device: &wgpu::Device,
    dim: (u32, u32),
    update_or_create: bool,
) -> wgpu::TextureView {
    if update_or_create {
        let (w, h) = (dim.0, dim.1);
        let resized = res
            .texture()
            .map(|t| t.width() != w || t.height() != h)
            .unwrap_or(true);
        if resized {
            // Now we create the texture
            let main_tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("src scene texture"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: res.target_format(),
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            res.set_texture(main_tex);
        }
    }
    res.texture()
        .unwrap()
        .create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        })
}

impl egui_wgpu::CallbackTrait for RendererCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        command_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let ppp = screen_descriptor.pixels_per_point;
        let (w, h) = (
            (self.canvas_size.x * ppp) as u32,
            (self.canvas_size.y * ppp) as u32,
        );
        run_source_render_pipeline(
            &self.params,
            device,
            queue,
            command_encoder,
            resources,
            (w, h),
        );
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        _render_pass: &mut wgpu::RenderPass<'_>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
    }
}

pub struct EffectsRenderResources {
    pub main_pipeline: wgpu::RenderPipeline,
    pub chroma_pipeline: wgpu::RenderPipeline,
    pub chroma_tex: Option<wgpu::Texture>,
    pub target_format: wgpu::TextureFormat,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub chroma_bind_group: Option<wgpu::BindGroup>,
    pub main_bind_group: Option<wgpu::BindGroup>,
    pub params_buffer: wgpu::Buffer,
}
impl Textured for EffectsRenderResources {
    fn texture(&self) -> Option<&wgpu::Texture> {
        self.chroma_tex.as_ref()
    }
    fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }
    fn set_texture(&mut self, tex: wgpu::Texture) {
        self.chroma_tex = Some(tex);
    }
}
impl EffectsRenderResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, dat: &EffectsCallback) {
        queue.write_buffer(
            &self.params_buffer,
            0,
            bytemuck::cast_slice(&[dat.use_bloom as u32]),
        );
        queue.write_buffer(
            &self.params_buffer,
            16,
            bytemuck::cast_slice(&[dat.bloom_amt]),
        );
        queue.write_buffer(
            &self.params_buffer,
            32,
            bytemuck::cast_slice(&[dat.use_vignette as u32]),
        );
        queue.write_buffer(
            &self.params_buffer,
            48,
            bytemuck::cast_slice(&[dat.vignette]),
        );
        queue.write_buffer(
            &self.params_buffer,
            64,
            bytemuck::cast_slice(&[dat.use_chroma as u32]),
        );
        queue.write_buffer(
            &self.params_buffer,
            80,
            bytemuck::cast_slice(&[dat.chroma_shift]),
        );
        queue.write_buffer(
            &self.params_buffer,
            96,
            bytemuck::cast_slice(&[dat.chroma_blur]),
        );
        queue.write_buffer(
            &self.params_buffer,
            112,
            bytemuck::cast_slice(&[dat.chroma_type.value()]),
        );
    }
}

pub struct EffectsCallback {
    pub top_left: Pos2,
    pub use_bloom: bool,
    pub bloom_amt: f32,
    pub use_vignette: bool,
    pub vignette: f32,
    pub use_chroma: bool,
    pub chroma_shift: f32,
    pub chroma_blur: f32,
    pub chroma_type: ChromaType,
}

pub fn run_effects_render_pipeline(
    data: &EffectsCallback,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    command_encoder: &mut wgpu::CommandEncoder,
    res: &mut egui_wgpu::CallbackResources,
) {
    let src_tex = res.get::<SrcRenderResources>().unwrap().texture().unwrap();
    let tex_size = (src_tex.width(), src_tex.height());

    // apply chromatic aberration first so all other postfx is applied to it
    let chroma_view =
        render_chromatic_aberration(res, device, queue, command_encoder, data, tex_size);

    let efx_res = res.get_mut::<EffectsRenderResources>().unwrap();
    let main_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("main efx bind group"),
        layout: &efx_res.bind_group_layout,
        entries: &create_bg_entries(&chroma_view, &efx_res.sampler, &efx_res.params_buffer),
    });
    efx_res.main_bind_group = Some(main_bind_group);
    let output_view = prepare_output_resources(res, device, tex_size);
    let mut main_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("main pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &output_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(CANVAS_BG),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });
    let efx_res = res.get::<EffectsRenderResources>().unwrap();
    main_pass.set_pipeline(&efx_res.main_pipeline);
    main_pass.set_bind_group(0, &efx_res.main_bind_group, &[]);
    main_pass.draw(0..6, 0..1);
}

fn render_chromatic_aberration(
    res: &mut egui_wgpu::CallbackResources,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    command_encoder: &mut wgpu::CommandEncoder,
    data: &EffectsCallback,
    tex_size: (u32, u32),
) -> wgpu::TextureView {
    let src_view = get_texture_view(
        res.get_mut::<SrcRenderResources>().unwrap(),
        device,
        tex_size,
        false,
    );

    let efx_res = res.get_mut::<EffectsRenderResources>().unwrap();
    let chroma_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("chrome bind group"),
        layout: &efx_res.bind_group_layout,
        entries: &create_bg_entries(&src_view, &efx_res.sampler, &efx_res.params_buffer),
    });
    efx_res.chroma_bind_group = Some(chroma_bind_group);
    efx_res.prepare(device, queue, data);

    let chroma_view = get_texture_view(efx_res, device, tex_size, true);
    let mut chroma_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("chroma pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &chroma_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(CANVAS_BG),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });
    chroma_pass.set_pipeline(&efx_res.chroma_pipeline);
    chroma_pass.set_bind_group(0, &efx_res.chroma_bind_group, &[]);
    chroma_pass.draw(0..6, 0..1);
    drop(chroma_pass);
    chroma_view
}

fn create_bg_entries<'a>(
    view: &'a wgpu::TextureView,
    sampler: &'a wgpu::Sampler,
    buffer: &'a wgpu::Buffer,
) -> [BindGroupEntry<'a>; 3] {
    [
        BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(view),
        },
        BindGroupEntry {
            binding: 1,
            resource: BindingResource::Sampler(sampler),
        },
        BindGroupEntry {
            binding: 2,
            resource: buffer.as_entire_binding(),
        },
    ]
}
impl egui_wgpu::CallbackTrait for EffectsCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        command_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        run_effects_render_pipeline(self, device, queue, command_encoder, resources);
        let out_res = resources.get::<OutputResources>().unwrap();
        out_res.prepare(queue, self.top_left * screen_descriptor.pixels_per_point);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &OutputResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}

pub struct OutputCallback;
pub fn run_output_render_pipeline(
    command_encoder: &mut wgpu::CommandEncoder,
    res: &OutputResources,
) {
    let tex = res.tex.as_ref().unwrap();
    let tex_img_copy = tex.as_image_copy();

    let row_bytes = tex.width() * 4;
    let padded_bytes = row_bytes.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    command_encoder.copy_texture_to_buffer(
        tex_img_copy,
        wgpu::TexelCopyBufferInfo {
            buffer: &res.output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes),
                rows_per_image: Some(tex.height()),
            },
        },
        tex.size(),
    );
}

impl egui_wgpu::CallbackTrait for OutputCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        command_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let out_res = resources.get::<OutputResources>().unwrap();
        run_output_render_pipeline(command_encoder, out_res);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        _render_pass: &mut wgpu::RenderPass<'static>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
    }
}

#[allow(dead_code)]
async fn read_debug_buffer(buffer_slice: wgpu::BufferSlice<'_>, device: &wgpu::Device) {
    let (tx, rx) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap_or_else(|e| {
            println!("{e}");
            wgpu::PollStatus::QueueEmpty
        });
    rx.recv_async().await.unwrap().unwrap();
    let data = buffer_slice.get_mapped_range();
    let _val: &[f32] = bytemuck::cast_slice(&data);
    // println!("Debug: {:?}", val.first().unwrap());
}

async fn read_output_buffer(
    buffer_slice: wgpu::BufferSlice<'_>,
    device: &wgpu::Device,
    ts: Vec2,
) -> Vec<u8> {
    {
        let (tx, rx) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap_or_else(|e| {
                println!("{e}");
                wgpu::PollStatus::QueueEmpty
            });
        rx.recv_async().await.unwrap().unwrap();
        let data = buffer_slice.get_mapped_range();
        let w = ts.x as usize;
        let h = ts.y as usize;
        let stride = size_of::<u32>();
        let mut display_data: Vec<u8> = Vec::with_capacity(w * h * stride);

        let start = (w * stride).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize);
        for i in 0..h {
            let offset = start * i;
            display_data.extend(&data[offset..offset + w * stride]);
        }
        display_data
    }
}

pub fn get_gpu_frame(device: &wgpu::Device, res: &OutputResources) -> Vec<u8> {
    let tex = res.tex.as_ref().unwrap();
    let fut = read_output_buffer(
        res.output_buffer.slice(..),
        device,
        vec2(tex.width() as f32, tex.height() as f32),
    );
    let frame = fut.block_on();
    res.output_buffer.unmap();
    frame
}
