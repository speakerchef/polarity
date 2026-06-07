use crate::{Rgba, audio::StereoFilter, state::*};
use egui::{Color32, Mesh, Pos2, Rect, pos2, vec2};
use std::{collections::VecDeque, ops::Neg};

use crate::{audio::audio_player::AudioPlayer, state::TraceDensity};

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

    pub live_buffer: VecDeque<Pos2>,
    pub live_low_buffer: VecDeque<Pos2>,
    pub live_mid_buffer: VecDeque<Pos2>,
    pub live_high_buffer: VecDeque<Pos2>,

    pub last_sample_idx: usize,
    pub trace_buffer: VecDeque<Pos2>,
    pub trace_low_buffer: VecDeque<Pos2>,
    pub trace_mid_buffer: VecDeque<Pos2>,
    pub trace_high_buffer: VecDeque<Pos2>,

    pub scale_factor: f32,
    pub point_size: f32,
}
enum FilterBand {
    Low,
    Mid,
    High,
}

impl Stereometer {
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
        let rscale = Self::radial_scale(l, r);
        match self.kind {
            StereometerKind::LinearBipolar => {
                ((l - r) * LINEAR_BIPOLAR_SF, (l + r) * LINEAR_BIPOLAR_SF)
            }
            StereometerKind::ScaledBipolar => {
                ((l - r) * rscale / SQRT_3, (l + r) * rscale / SQRT_3)
            }
            StereometerKind::LinearLissajous => (l, r),
            StereometerKind::ScaledLissajous => (l * rscale, r * rscale),
        }
    }

    fn set_positions(&mut self, center: Pos2, is_live: bool, l: f32, r: f32) {
        match self.render_mode {
            RenderMode::FullSpectrum => {
                let (l, r) = self.filter_fs(is_live, l, r);
                let (l, r) = self.get_coord_from_meterkind(l, r);
                let (sl, sr) = (l * self.scale_factor, r * self.scale_factor);
                let pos = pos2(center.x + sl, center.y + sr.neg());
                if is_live {
                    self.live_buffer.push_back(pos);
                } else {
                    self.trace_buffer.push_back(pos);
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
                let posl = pos2(center.x + lowl, center.y + lowr.neg());
                let posm = pos2(center.x + midl, center.y + midr.neg());
                let posh = pos2(center.x + highl, center.y + highr.neg());
                if is_live {
                    self.live_low_buffer.push_back(posl);
                    self.live_mid_buffer.push_back(posm);
                    self.live_high_buffer.push_back(posh);
                } else {
                    self.trace_low_buffer.push_back(posl);
                    self.trace_mid_buffer.push_back(posm);
                    self.trace_high_buffer.push_back(posh);
                }
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
        while self.trace_buffer.len() > self.trace_density.count() {
            self.trace_buffer.pop_front();
        }
        while self.trace_low_buffer.len() > self.trace_density.count() {
            self.trace_low_buffer.pop_front();
        }
        while self.trace_mid_buffer.len() > self.trace_density.count() {
            self.trace_mid_buffer.pop_front();
        }
        while self.trace_high_buffer.len() > self.trace_density.count() {
            self.trace_high_buffer.pop_front();
        }
    }

    fn clear_live_buffers(&mut self) {
        self.live_buffer.clear();
        self.live_low_buffer.clear();
        self.live_mid_buffer.clear();
        self.live_high_buffer.clear();
    }

    pub fn draw(&mut self, p: &AudioPlayer, center: Pos2) -> Mesh {
        let num_channels = p.contents.num_channels as usize;
        let mut live_mesh = Mesh::default();
        let mut trace_mesh = Mesh::default();

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
            self.set_positions(center, is_live, *l, *r);
        });
        self.limit_trace_buffers();
        self.set_mesh(&mut trace_mesh, is_live);
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
            self.set_positions(center, is_live, *l, *r);
        });
        self.set_mesh(&mut live_mesh, is_live);

        live_mesh.append(trace_mesh);
        live_mesh
    }
}
