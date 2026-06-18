use eframe::egui_wgpu;
use egui::{Pos2, Vec2, vec2};
use pollster::FutureExt;

use crate::{
    LinearRgba,
    generators::stereometer::{MAX_TRACE_POINT_DENSITY, RenderMode, VERTICES_PER_QUAD},
};

const CANVAS_BG: wgpu::Color = wgpu::Color::BLACK;
pub struct StereometerRenderResources {
    pub target_format: wgpu::TextureFormat,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub params_buffer: wgpu::Buffer,
    pub alpha_buffer: wgpu::Buffer,
    pub tex: Option<wgpu::Texture>,
}
#[allow(clippy::too_many_arguments)]
impl StereometerRenderResources {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        pos: Vec<Pos2>,
        fs_color: LinearRgba,
        lb_color: LinearRgba,
        mb_color: LinearRgba,
        hb_color: LinearRgba,
        is_mb: bool,
        live_len: u32,
        trace_len: u32,
        live_mb_len: u32,
        trace_mb_len: u32,
    ) {
        let alphas: Vec<f32> = if is_mb {
            (0..trace_mb_len)
                .map(|i| {
                    (i as f32 / (MAX_TRACE_POINT_DENSITY * VERTICES_PER_QUAD) as f32).powf(1.75)
                })
                .collect()
        } else {
            (0..trace_len)
                .map(|i| {
                    (i as f32 / (MAX_TRACE_POINT_DENSITY * VERTICES_PER_QUAD) as f32).powf(1.75)
                })
                .collect()
        };

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
        queue.write_buffer(&self.params_buffer, 128, &[is_mb as u8, 0, 0, 0]);

        queue.write_buffer(&self.alpha_buffer, 0, bytemuck::cast_slice(&alphas));
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&pos));
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>, num_points: u32) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[0]);
        render_pass.draw(0..num_points, 0..1);
    }
}
pub struct OutputResources {
    pub output_buffer: wgpu::Buffer,
    pub target_format: wgpu::TextureFormat,
    // pub pipeline: wgpu::RenderPipeline,
    // pub bind_group: wgpu::BindGroup,
    pub tex: Option<wgpu::Texture>,
}

async fn read_output_buffer(buffer_slice: wgpu::BufferSlice<'_>, device: &wgpu::Device, ts: Vec2) {
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

        let padded_w = w.next_multiple_of(256);

        let start = padded_w * stride;
        for i in 0..h {
            let offset = start * i;
            display_data.extend(&data[offset..offset + w * stride]);
        }

        let buffer =
            image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(w as u32, h as u32, display_data)
                .unwrap();
        buffer.save("image.png").unwrap();
    }
}

pub struct RendererCallback {
    // Stereometer fields
    pub render_mode: RenderMode,
    pub live_pos: Vec<Pos2>,
    pub trace_pos: Vec<Pos2>,

    pub live_low_pos: Vec<Pos2>,
    pub live_mid_pos: Vec<Pos2>,
    pub live_high_pos: Vec<Pos2>,
    pub trace_low_pos: Vec<Pos2>,
    pub trace_mid_pos: Vec<Pos2>,
    pub trace_high_pos: Vec<Pos2>,

    pub fs_color: LinearRgba,
    pub lb_color: LinearRgba,
    pub mb_color: LinearRgba,
    pub hb_color: LinearRgba,
    pub canvas_size: Vec2,
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
        let meter_res: &mut StereometerRenderResources = resources.get_mut().unwrap();
        let pos = match self.render_mode {
            RenderMode::FullSpectrum => {
                let mut pos = self.live_pos.clone();
                pos.extend(&self.trace_pos);
                pos
            }
            RenderMode::MultiBand => {
                let mut pos = self.live_low_pos.clone();
                pos.extend(&self.live_mid_pos);
                pos.extend(&self.live_high_pos);
                pos.extend(&self.trace_low_pos);
                pos.extend(&self.trace_mid_pos);
                pos.extend(&self.trace_high_pos);
                pos
            }
        };
        let (live_len, trace_len) = (self.live_pos.len(), self.trace_pos.len());
        let (live_mb_len, trace_mb_len) = (self.live_low_pos.len(), self.trace_low_pos.len());
        meter_res.prepare(
            device,
            queue,
            pos,
            self.fs_color,
            self.lb_color,
            self.mb_color,
            self.hb_color,
            matches!(self.render_mode, RenderMode::MultiBand),
            live_len as u32,
            trace_len as u32,
            live_mb_len as u32,
            trace_mb_len as u32,
        );

        let ppp = screen_descriptor.pixels_per_point;
        let (w, h) = (
            (self.canvas_size.x * ppp) as u32,
            (self.canvas_size.y * ppp) as u32,
        );
        let resized = meter_res
            .tex
            .as_ref()
            .map(|t| t.width() != w || t.height() != h)
            .unwrap_or(true);
        if resized {
            // mip mapping
            // Here we compute the number of mip levels using the smaller of width and height
            let mip_level_count = w.min(h).ilog2() + 1;

            // Now we create the texture
            let diffuse_blit_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("blit texture"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count, // This is the important bit
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: meter_res.target_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            meter_res.tex = Some(diffuse_blit_texture);
        }
        let src_view = meter_res
            .tex
            .as_ref()
            .unwrap()
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });

        let mut stereometer_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("stereometer pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &src_view,
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

        let n = match self.render_mode {
            RenderMode::FullSpectrum => (self.live_pos.len() + self.trace_pos.len()) as u32,
            RenderMode::MultiBand => {
                let live =
                    self.live_low_pos.len() + self.live_mid_pos.len() + self.live_high_pos.len();
                let trace =
                    self.trace_low_pos.len() + self.trace_mid_pos.len() + self.trace_high_pos.len();
                (live + trace) as u32
            }
        };
        meter_res.paint(&mut stereometer_pass, n);
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

pub struct BlitRenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
}
pub struct BloomRenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub bind_group: Option<wgpu::BindGroup>,
    pub params_buffer: wgpu::Buffer,
}
impl BloomRenderResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, top_left: Pos2, bloom_amt: f32) {
        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[top_left]));
        queue.write_buffer(&self.params_buffer, 16, bytemuck::cast_slice(&[bloom_amt]));
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

pub struct EffectsCallback {
    pub top_left: Pos2,
    pub bloom_amt: f32,
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
        let meter_res: &StereometerRenderResources = resources.get().unwrap();
        let blit_res: &BlitRenderResources = resources.get().unwrap();
        let bloom_res: &BloomRenderResources = resources.get().unwrap();

        let tex = meter_res.tex.as_ref().unwrap();
        let full_mip_view = tex.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_mip_level: 0,
            mip_level_count: None,
            ..Default::default()
        });
        let mut src_view = tex.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let bloom_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom"),
            layout: &bloom_res.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&full_mip_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&bloom_res.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: bloom_res.params_buffer.as_entire_binding(),
                },
            ],
        });

        for mip in 1..tex.mip_level_count() {
            let dst_view = tex.create_view(&wgpu::TextureViewDescriptor {
                format: Some(meter_res.target_format),
                base_mip_level: mip,
                mip_level_count: Some(1),
                ..Default::default()
            });

            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &blit_res.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&blit_res.sampler),
                    },
                ],
            });

            let mut pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&blit_res.pipeline);
            pass.set_bind_group(0, &texture_bind_group, &[]);
            pass.draw(0..6, 0..1);

            // make sure that current mip is src in next iteration.
            src_view = dst_view;
        }
        let bloom_res = resources.get_mut::<BloomRenderResources>().unwrap();
        bloom_res.bind_group = Some(bloom_bind_group);
        bloom_res.prepare(
            device,
            queue,
            self.top_left * screen_descriptor.pixels_per_point,
            self.bloom_amt,
        );
        let meter_res: &StereometerRenderResources = resources.get().unwrap();
        let meter_tex = meter_res.tex.as_ref().unwrap();
        let (w, h) = (meter_tex.width(), meter_tex.height());

        let output_res: &mut OutputResources = resources.get_mut().unwrap();
        let resized = output_res
            .tex
            .as_ref()
            .map(|t| t.width() != w || t.height() != h)
            .unwrap_or(true);
        if resized {
            let output_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("output_texture"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: output_res.target_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            output_res.tex = Some(output_texture);
        }
        let src_view = output_res
            .tex
            .as_ref()
            .unwrap()
            .create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_mip_level: 0,
                mip_level_count: None,
                ..Default::default()
            });

        let mut output_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("output pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &src_view,
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
        let bloom_res = resources.get::<BloomRenderResources>().unwrap();

        output_pass.set_pipeline(&bloom_res.pipeline);
        output_pass.set_bind_group(0, &bloom_res.bind_group, &[]);
        output_pass.draw(0..6, 0..1);

        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &BloomRenderResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}
pub struct OutputCallback;
impl egui_wgpu::CallbackTrait for OutputCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        command_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let output_res: &OutputResources = resources.get().unwrap();

        let tex = output_res.tex.as_ref().unwrap();
        let tex_img_copy = tex.as_image_copy();

        let out_res: &OutputResources = resources.get().unwrap();
        command_encoder.copy_texture_to_buffer(
            tex_img_copy,
            wgpu::TexelCopyBufferInfo {
                buffer: &out_res.output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        size_of::<u32>() as u32 * tex.width().next_multiple_of(256),
                    ),
                    rows_per_image: Some(tex.height()),
                },
            },
            tex.size(),
        );
        let fut = read_output_buffer(
            out_res.output_buffer.slice(..),
            device,
            vec2(tex.width() as f32, tex.height() as f32),
        );
        fut.block_on();
        out_res.output_buffer.unmap();
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
