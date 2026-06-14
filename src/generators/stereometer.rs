use crate::{LinearRgba, Rgba, audio::StereoFilter, labeled_enum, state::Labeled};
use eframe::egui_wgpu;
use egui::{Color32, Mesh, Pos2, Rect, Vec2, pos2, vec2};
use rodio::queue;
use std::collections::VecDeque;

use crate::audio::audio_player::AudioPlayer;

labeled_enum!(StereometerKind {
    LinearBipolar  => "Linear Bipolar",
    ScaledBipolar  => "Scaled Bipolar",
    LinearLissajous => "Linear Lissajous",
    ScaledLissajous => "Scaled Lissajous",
}, LinearLissajous);

labeled_enum!(RenderMode {
    FullSpectrum => "Full Spectrum",
    MultiBand    => "Multi-Band",
}, FullSpectrum);

labeled_enum!(FilterMode {
    Off => "Off",
    Lpf => "Lpf",
    Bpf => "Bpf",
    Hpf => "Hpf",
}, Off);

labeled_enum!(LiveDensity {
    Low => "Low",
    Med => "Med",
    High => "High",
    Ultra => "Ultra",
    Extreme => "Extreme",
    PleaseDont => "Please Dont",
}, High);

labeled_enum!(TraceDensity {
    Off => "Off",
    Low => "Low",
    Med => "Med",
    High => "High",
    Max => "Max",
}, Med);

impl LiveDensity {
    pub fn count(self) -> usize {
        match self {
            Self::Low => 512,
            Self::Med => 1536,
            Self::High => 2048,
            Self::Ultra => 4096,
            Self::Extreme => 8192,
            Self::PleaseDont => 16384,
        }
    }
}

impl TraceDensity {
    pub fn count(self) -> usize {
        match self {
            Self::Off => 1,
            Self::Low => 10420,
            Self::Med => 15696,
            Self::High => 24576,
            Self::Max => 32768,
        }
    }
}

impl Labeled for RenderMode {
    fn text(self) -> &'static str {
        self.label()
    }
}
impl Labeled for FilterMode {
    fn text(self) -> &'static str {
        self.label()
    }
}
impl Labeled for StereometerKind {
    fn text(self) -> &'static str {
        self.label()
    }
}

impl Labeled for LiveDensity {
    fn text(self) -> &'static str {
        self.label()
    }
}

impl Labeled for TraceDensity {
    fn text(self) -> &'static str {
        self.label()
    }
}

pub const MAX_LIVE_POINT_DENSITY: usize = 16384;
pub const MAX_TRACE_POINT_DENSITY: usize = 32768;
pub const VERTICES_PER_QUAD: usize = 6;
const SQRT_3: f32 = 1.7320508;
const LINEAR_BIPOLAR_SF: f32 = 0.5;

#[derive(Default)]
pub struct Stereometer {
    pub kind: StereometerKind,
    pub render_mode: RenderMode,

    pub live_density: LiveDensity,
    pub trace_density: TraceDensity,

    pub filter_mode: FilterMode,
    pub filter_freq: f32,
    pub last_freq: f32,
    pub live_fs_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    pub trace_fs_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    pub live_mb_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,
    pub trace_mb_filters: Option<(StereoFilter, StereoFilter, StereoFilter)>,

    pub fs_color: Rgba,
    pub mb_color: [Rgba; 3],

    pub live_buffer: Vec<Pos2>,
    pub live_low_buffer: Vec<Pos2>,
    pub live_mid_buffer: Vec<Pos2>,
    pub live_high_buffer: Vec<Pos2>,

    pub last_sample_idx: usize,
    pub trace_buffer: VecDeque<Pos2>,
    pub trace_low_buffer: VecDeque<Pos2>,
    pub trace_mid_buffer: VecDeque<Pos2>,
    pub trace_high_buffer: VecDeque<Pos2>,

    pub scale_factor: f32,
    pub point_size: f32,
}

pub struct StereometerRenderResources {
    pub target_format: wgpu::TextureFormat,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub params_buffer: wgpu::Buffer,
    pub alpha_buffer: wgpu::Buffer,
    pub tex: Option<wgpu::Texture>,
}
impl StereometerRenderResources {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        pos: Vec<Pos2>,
        color: LinearRgba,
        live_len: u32,
        trace_len: u32,
    ) {
        let alphas: Vec<f32> = (0..trace_len)
            .map(|i| i as f32 / (MAX_TRACE_POINT_DENSITY * VERTICES_PER_QUAD) as f32)
            .collect();

        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[live_len]));
        queue.write_buffer(&self.params_buffer, 16, bytemuck::cast_slice(&[trace_len]));
        queue.write_buffer(
            &self.params_buffer,
            32,
            bytemuck::cast_slice(&[color.r, color.g, color.b, color.a]),
        );
        queue.write_buffer(&self.alpha_buffer, 0, bytemuck::cast_slice(&alphas));
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&pos));
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>, num_points: u32) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[0]);
        render_pass.draw(0..num_points, 0..1);
    }
}
pub struct RendererCallback {
    // Stereometer fields
    pub live_pos: Vec<Pos2>,
    pub trace_pos: Vec<Pos2>,
    pub color: LinearRgba,
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
        let mut pos = self.live_pos.clone();
        pos.extend(&self.trace_pos);
        meter_res.prepare(
            device,
            queue,
            pos,
            self.color,
            self.live_pos.len() as u32,
            self.trace_pos.len() as u32,
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
            let tex_desc = wgpu::TextureDescriptor {
                label: Some("stereometer texture"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: meter_res.target_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            };
            meter_res.tex = Some(device.create_texture(&tex_desc));
        }

        let mut stereometer_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("stereometer pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &meter_res
                    .tex
                    .as_ref()
                    .unwrap()
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let n = (self.live_pos.len() + self.trace_pos.len()) as u32;
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

pub struct BloomRenderResources {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub bind_group: Option<wgpu::BindGroup>,
    pub top_left: wgpu::Buffer,
}
impl BloomRenderResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, top_left: Pos2) {
        queue.write_buffer(&self.top_left, 0, bytemuck::cast_slice(&[top_left]));
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

pub struct EffectsCallback {
    pub top_left: Pos2,
}
impl egui_wgpu::CallbackTrait for EffectsCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _command_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let meter_res: &StereometerRenderResources = resources.get().unwrap();
        let bloom_res: &BloomRenderResources = resources.get().unwrap();
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom"),
            layout: &bloom_res.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &meter_res
                            .tex
                            .as_ref()
                            .unwrap()
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&bloom_res.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: bloom_res.top_left.as_entire_binding(),
                },
            ],
        });
        let bloom_res = resources.get_mut::<BloomRenderResources>().unwrap();
        bloom_res.bind_group = Some(bind_group);
        bloom_res.prepare(
            device,
            queue,
            self.top_left * screen_descriptor.pixels_per_point,
        );

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

enum FilterBand {
    Low,
    Mid,
    High,
}

impl Stereometer {
    fn points_to_quad_vertices(&self, l: f32, r: f32) -> [Pos2; 6] {
        let s = 0.003;
        [
            pos2(l + s, r + s),
            pos2(l + s, r - s),
            pos2(l - s, r - s),
            pos2(l + s, r + s),
            pos2(l - s, r + s),
            pos2(l - s, r - s),
        ]
    }
    fn filter_fs(&mut self, is_live: bool, l: f32, r: f32) -> (f32, f32) {
        if is_live {
            if let Some(live_fs) = &mut self.live_fs_filters {
                match self.filter_mode {
                    FilterMode::Off => (l, r),
                    FilterMode::Lpf => live_fs.0.run(l, r),
                    FilterMode::Bpf => live_fs.1.run(l, r),
                    FilterMode::Hpf => live_fs.2.run(l, r),
                }
            } else {
                (l, r)
            }
        } else {
            if let Some(trace_fs) = &mut self.trace_fs_filters {
                match self.filter_mode {
                    FilterMode::Off => (l, r),
                    FilterMode::Lpf => trace_fs.0.run(l, r),
                    FilterMode::Bpf => trace_fs.1.run(l, r),
                    FilterMode::Hpf => trace_fs.2.run(l, r),
                }
            } else {
                (l, r)
            }
        }
    }

    fn filter_mb(&mut self, is_live: bool, band: FilterBand, l: f32, r: f32) -> (f32, f32) {
        if is_live {
            if let Some(live) = &mut self.live_mb_filters {
                match band {
                    FilterBand::Low => live.0.run(l, r),
                    FilterBand::Mid => live.1.run(l, r),
                    FilterBand::High => live.2.run(l, r),
                }
            } else {
                println!("NO FILTER");
                (l, r)
            }
        } else {
            if let Some(trace) = &mut self.trace_mb_filters {
                match band {
                    FilterBand::Low => trace.0.run(l, r),
                    FilterBand::Mid => trace.1.run(l, r),
                    FilterBand::High => trace.2.run(l, r),
                }
            } else {
                println!("NO FILTER");
                (l, r)
            }
        }
    }

    fn radial_scale(x: f32, y: f32) -> f32 {
        let sf = 0.3;
        let mag = (x * x + y * y).sqrt();
        let scaled = mag.powf(sf);
        if mag > 1e-6 { scaled / mag } else { 0.0 }
    }

    fn get_coord_from_meterkind(&self, l: f32, r: f32) -> (f32, f32) {
        match self.kind {
            StereometerKind::LinearBipolar => {
                ((l - r) * LINEAR_BIPOLAR_SF, (l + r) * LINEAR_BIPOLAR_SF)
            }
            StereometerKind::ScaledBipolar => {
                let rscale = Self::radial_scale(l, r);
                ((l - r) * rscale / SQRT_3, (l + r) * rscale / SQRT_3)
            }
            StereometerKind::LinearLissajous => (l, r),
            StereometerKind::ScaledLissajous => {
                let rscale = Self::radial_scale(l, r);
                (l * rscale, r * rscale)
            }
        }
    }

    fn set_positions(&mut self, is_live: bool, l: f32, r: f32) {
        match self.render_mode {
            RenderMode::FullSpectrum => {
                let (l, r) = self.filter_fs(is_live, l, r);
                let (l, r) = self.get_coord_from_meterkind(l, r);
                let pos = self.points_to_quad_vertices(l, r);
                if is_live {
                    self.live_buffer.extend(pos);
                } else {
                    self.trace_buffer.extend(pos);
                }
            }
            RenderMode::MultiBand => {
                let (lowl, lowr) = self.filter_mb(is_live, FilterBand::Low, l, r);
                let (midl, midr) = self.filter_mb(is_live, FilterBand::Mid, l, r);
                let (highl, highr) = self.filter_mb(is_live, FilterBand::High, l, r);
                let (lowl, lowr) = self.get_coord_from_meterkind(lowl, lowr);
                let (midl, midr) = self.get_coord_from_meterkind(midl, midr);
                let (highl, highr) = self.get_coord_from_meterkind(highl, highr);
                let (lowl, lowr) = (lowl * self.scale_factor, lowr * self.scale_factor);
                let (midl, midr) = (midl * self.scale_factor, midr * self.scale_factor);
                let (highl, highr) = (highl * self.scale_factor, highr * self.scale_factor);
                // let posl = pos2(center.x + lowl, center.y + lowr.neg());
                // let posm = pos2(center.x + midl, center.y + midr.neg());
                // let posh = pos2(center.x + highl, center.y + highr.neg());
                // if is_live {
                //     self.live_low_buffer.push(posl);
                //     self.live_mid_buffer.push(posm);
                //     self.live_high_buffer.push(posh);
                // } else {
                //     self.trace_low_buffer.push_back(posl);
                //     self.trace_mid_buffer.push_back(posm);
                //     self.trace_high_buffer.push_back(posh);
                // }
            }
        }
    }

    fn set_mesh(&mut self, mesh: &mut Mesh, is_live: bool) {
        match self.render_mode {
            RenderMode::FullSpectrum => {
                if is_live {
                    self.live_buffer.iter().for_each(|&pos| {
                        mesh.add_colored_rect(
                            Rect::from_min_size(pos, vec2(self.point_size, self.point_size)),
                            Color32::from_rgb(
                                self.fs_color.r as u8,
                                self.fs_color.g as u8,
                                self.fs_color.b as u8,
                            ),
                        );
                    });
                } else {
                    self.trace_buffer.iter().enumerate().for_each(|(i, &pos)| {
                        let alpha =
                            ((i as f32 / TraceDensity::Max.count() as f32) * u8::MAX as f32) as u8;
                        mesh.add_colored_rect(
                            Rect::from_min_size(pos, vec2(self.point_size, self.point_size)),
                            Color32::from_rgba_unmultiplied(
                                self.fs_color.r as u8,
                                self.fs_color.g as u8,
                                self.fs_color.b as u8,
                                alpha,
                            ),
                        );
                    });
                }
            }
            RenderMode::MultiBand => {
                if is_live {
                    self.live_low_buffer.iter().for_each(|&pos| {
                        mesh.add_colored_rect(
                            Rect::from_min_size(pos, vec2(self.point_size, self.point_size)),
                            Color32::from_rgb(
                                self.mb_color[0].r as u8,
                                self.mb_color[0].g as u8,
                                self.mb_color[0].b as u8,
                            ),
                        );
                    });
                    self.live_mid_buffer.iter().for_each(|&pos| {
                        mesh.add_colored_rect(
                            Rect::from_min_size(pos, vec2(self.point_size, self.point_size)),
                            Color32::from_rgb(
                                self.mb_color[1].r as u8,
                                self.mb_color[1].g as u8,
                                self.mb_color[1].b as u8,
                            ),
                        );
                    });
                    self.live_high_buffer.iter().for_each(|&pos| {
                        mesh.add_colored_rect(
                            Rect::from_min_size(pos, vec2(self.point_size, self.point_size)),
                            Color32::from_rgb(
                                self.mb_color[2].r as u8,
                                self.mb_color[2].g as u8,
                                self.mb_color[2].b as u8,
                            ),
                        );
                    });
                } else {
                    self.trace_low_buffer
                        .iter()
                        .enumerate()
                        .for_each(|(i, &pos)| {
                            let alpha = ((i as f32 / TraceDensity::Max.count() as f32)
                                * u8::MAX as f32) as u8;
                            mesh.add_colored_rect(
                                Rect::from_min_size(pos, vec2(self.point_size, self.point_size)),
                                Color32::from_rgba_unmultiplied(
                                    self.mb_color[0].r as u8,
                                    self.mb_color[0].g as u8,
                                    self.mb_color[0].b as u8,
                                    alpha,
                                ),
                            );
                        });
                    self.trace_mid_buffer
                        .iter()
                        .enumerate()
                        .for_each(|(i, &pos)| {
                            let alpha = ((i as f32 / TraceDensity::Max.count() as f32)
                                * u8::MAX as f32) as u8;
                            mesh.add_colored_rect(
                                Rect::from_min_size(pos, vec2(self.point_size, self.point_size)),
                                Color32::from_rgba_unmultiplied(
                                    self.mb_color[1].r as u8,
                                    self.mb_color[1].g as u8,
                                    self.mb_color[1].b as u8,
                                    alpha,
                                ),
                            );
                        });
                    self.trace_high_buffer
                        .iter()
                        .enumerate()
                        .for_each(|(i, &pos)| {
                            let alpha = ((i as f32 / TraceDensity::Max.count() as f32)
                                * u8::MAX as f32) as u8;
                            mesh.add_colored_rect(
                                Rect::from_min_size(pos, vec2(self.point_size, self.point_size)),
                                Color32::from_rgba_unmultiplied(
                                    self.mb_color[2].r as u8,
                                    self.mb_color[2].g as u8,
                                    self.mb_color[2].b as u8,
                                    alpha,
                                ),
                            );
                        });
                }
            }
        }
    }

    fn limit_trace_buffers(&mut self) {
        // trace buffers hold VERTICES_PER_QUAD vertices per sample, so the
        // sample-count density must be scaled to vertices.
        let cap = self.trace_density.count() * VERTICES_PER_QUAD;
        while self.trace_buffer.len() > cap {
            self.trace_buffer.pop_front();
        }
        while self.trace_low_buffer.len() > cap {
            self.trace_low_buffer.pop_front();
        }
        while self.trace_mid_buffer.len() > cap {
            self.trace_mid_buffer.pop_front();
        }
        while self.trace_high_buffer.len() > cap {
            self.trace_high_buffer.pop_front();
        }
    }

    fn clear_live_buffers(&mut self) {
        self.live_buffer.clear();
        self.live_low_buffer.clear();
        self.live_mid_buffer.clear();
        self.live_high_buffer.clear();
    }

    pub fn draw(&mut self, p: &AudioPlayer, size: Vec2) -> (Vec<Pos2>, Vec<Pos2>) {
        let num_channels = p.contents.num_channels as usize;

        let sample_pos = p.position().as_secs_f64();
        let sample_idx = (sample_pos * p.contents.sample_rate as f64) as usize;
        let last_idx = self.last_sample_idx;
        if sample_idx < last_idx {
            self.trace_buffer.clear();
        }

        let mut is_live = false;
        let trace_window = p
            .contents
            .samples
            .get(last_idx * num_channels..sample_idx * num_channels)
            .unwrap_or_default();

        trace_window.chunks_exact(2).for_each(|s| {
            let l = s.first().unwrap();
            let r = s.last().unwrap_or(l);
            self.set_positions(is_live, *l, *r);
        });
        self.limit_trace_buffers();
        self.last_sample_idx = sample_idx;

        is_live = true;
        let live_window = p
            .contents
            .samples
            .get(sample_idx * num_channels..sample_idx * num_channels + self.live_density.count())
            .unwrap_or_default();

        self.clear_live_buffers();
        live_window.chunks_exact(2).for_each(|s| {
            let l = s.first().unwrap();
            let r = s.last().unwrap_or(l);
            self.set_positions(is_live, *l, *r);
        });
        (self.live_buffer.clone(), self.trace_buffer.clone().into())
    }
}
