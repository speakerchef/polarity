use biquad::*;
use eframe::egui_wgpu::{self, wgpu};
use egui_winit::winit::dpi::LogicalSize;
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent};
use std::{
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    audio::{StereoFilter, audio_player::*},
    generators::{
        rendering::{
            RendererCallback, effects_render_pipeline, get_gpu_frame, main_render_pipeline,
            output_render_pipeline, prep_bloom_resources_for_effects,
            prep_meter_resources_for_effects, prep_output_resources_for_effects,
        },
        stereometer::{FilterMode, Stereometer},
    },
    state::PlaybackMode,
    ui::{
        app_widgets::{export_modal, menu_bar, preset_modal, window_drag_tooltip},
        canvas, control_panel, timeline,
        timeline_widgets::{SHARP, border},
    },
    wgpu_init::setup_wgpu,
};
use eframe::egui;
use eframe::egui::{Key, StrokeKind, vec2};

use crate::{state::AppState, ui::palette as plt, ui::theme};

#[derive(Default)]
pub struct PolarityApp {
    st: AppState,
    player: Option<AudioPlayer>,
}

const BATCH_SIZE: usize = 30;

fn export_batched_frames(
    st: &mut AppState,
    p: &AudioPlayer,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let fps = st.export_config.frame_rate.value();
    let canvas_size = st.export_config.resolution.value();
    let (w, h) = (canvas_size.0, canvas_size.1);

    let bloom_amt = st.stereo.bloom;
    let device = &wgpu_render_state.device;
    let queue = &wgpu_render_state.queue;

    let meter_res = st.stereometer_render_resources.as_mut().unwrap();
    let bloom_res = st.bloom_render_resources.as_mut().unwrap();
    let out_res = st.output_render_resources.as_mut().unwrap();

    // Spawn writer thread for entire job
    if st.writer_handle.is_none() {
        let quality = st.export_config.quality.value();
        let total_frames = p.contents.duration.as_secs_f32() * fps as f32;

        let mut output = FfmpegCommand::new()
            .format("rawvideo")
            .args(["-pixel_format", "bgra"])
            .size(w, h)
            .rate(fps as f32)
            .input("-")
            .input(p.contents.path.to_string_lossy())
            .codec_video("libx264")
            .crf(quality as u32)
            .preset("veryfast")
            .pix_fmt("yuv444p")
            .codec_audio("aac")
            .args(["-b:a", "320k"])
            .args(["-y", "out.mp4"])
            .spawn()
            .unwrap();

        let mut stdin = output.take_stdin().unwrap();
        let (tx, rx) = flume::bounded::<Vec<u8>>(4);
        let write_handle = std::thread::spawn(move || {
            rx.iter().for_each(|frame| {
                stdin.write_all(&frame).unwrap();
            });
            drop(stdin);
        });
        let log_handle = std::thread::spawn(move || {
            for event in output.iter().unwrap() {
                match event {
                    FfmpegEvent::Log(_, _) => (),
                    FfmpegEvent::Error(_e) => (),
                    FfmpegEvent::Progress(prog) => println!("{}", prog.raw_log_message),
                    FfmpegEvent::Done | FfmpegEvent::LogEOF => break,
                    _ => (),
                }
            }
            output.wait().unwrap();
        });
        st.writer_handle = Some(write_handle);
        st.logger_handle = Some(log_handle);
        st.export_tx = Some(tx);
        st.export_config.total_frames = total_frames as usize;
    }

    for _ in 0..BATCH_SIZE {
        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("export command encoder"),
        });

        st.cur_frame_idx += 1;
        if st.cur_frame_idx >= st.export_config.total_frames || st.export_canceled {
            drop(std::mem::take(&mut st.export_tx));
            st.writer_handle.take().unwrap().join().unwrap();
            st.logger_handle.take().unwrap().join().unwrap();
            st.rendering = false;
            st.show_export_modal = false;
            st.cur_frame_idx = 0;
            st.export_elapsed_time.take();
            st.prev_export_timestamp.take();
            st.export_config.total_frames = 0;
            st.export_canceled = false;
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
            menu_bar(&mut self.st, ui);
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
                    .fill(plt::BG(self.st.dark_mode)),
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
                if self.st.show_preset_load_modal || self.st.show_preset_save_modal {
                    preset_modal(ui, &mut self.st);
                }

                if self.st.rendering {
                    self.player.as_ref().unwrap().pause();
                } else {
                    canvas::draw(ui, &mut self.st, &self.player);
                }
            })
            .response;

        if !self.st.fullscreen {
            resp.rect = resp.rect.translate(vec2(0., -6.));
            resp.rect = resp.rect.shrink2(vec2(12.0, 6.0));
            ui.painter().rect_stroke(
                resp.rect,
                SHARP,
                border(ui.style().visuals.dark_mode),
                StrokeKind::Inside,
            );
        }
    }
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        theme::apply_theme(ctx, self.st.dark_mode);

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

            if let (Some(t), Some(start_point)) = (
                self.st.export_elapsed_time.as_mut(),
                self.st.prev_export_timestamp,
            ) {
                *t = Instant::now().duration_since(start_point);
            } else {
                self.st.prev_export_timestamp = Some(Instant::now());
                self.st.export_elapsed_time = Some(Duration::default());
            }
            export_batched_frames(&mut self.st, p, wgpu_render_state);

            ctx.request_repaint();
        }

        // Open audio import dialog
        if self.st.import_open {
            self.st.file_dialog.pick_file();
            self.st.import_open = false;
            self.st.start_render = false;
        }

        // save dialog
        if self.st.open_preset_save_file_picker {
            self.st.dir_dialog.pick_directory();
            self.st.file_picked = true;
            self.st.open_preset_save_file_picker = false;
            self.st.show_preset_save_modal = false;
            self.st.picked_preset_save_dir = true;
        }
        // load dialog
        if self.st.open_preset_load_file_picker {
            self.st.dir_dialog.pick_file();
            self.st.file_picked = true;
            self.st.open_preset_load_file_picker = false;
            self.st.show_preset_load_modal = false;
            self.st.picked_preset_load_file = true;
        }

        // check if user picked save directory
        if let Some(path) = self
            .st
            .dir_dialog
            .update(ctx)
            .picked()
            .map(|p| p.to_path_buf())
            && (!self.st.save_preset && self.st.picked_preset_save_dir)
        {
            if let Some(oldpath) = &self.st.preset_save_path {
                if *oldpath != path {
                    self.st.preset_save_path = Some(path);
                    self.st.show_preset_save_modal = true;
                }
            } else {
                self.st.preset_save_path = Some(path);
                self.st.show_preset_save_modal = true;
            }
            self.st.picked_preset_save_dir = false;
        };

        // check if user picked load directory
        if let Some(path) = self
            .st
            .dir_dialog
            .update(ctx)
            .picked()
            .map(|p| p.to_path_buf())
            && (!self.st.load_preset && self.st.picked_preset_load_file)
        {
            if let Some(oldpath) = &self.st.preset_load_path {
                if *oldpath != path {
                    self.st.preset_load_path = Some(path);
                    self.st.show_preset_load_modal = true;
                }
            } else {
                self.st.preset_load_path = Some(path);
                self.st.show_preset_load_modal = true;
            }
            self.st.picked_preset_load_file = false;
        };

        // Check if user picked an audio file
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

        if self.st.save_preset {
            let Ok(data) = serde_json::to_vec(&self.st.stereo) else {
                return;
            };
            let Some(path) = &self.st.preset_save_path else {
                return;
            };
            std::fs::write(path.join(format!("{}.json", self.st.preset_name)), data)
                .unwrap_or_else(|e| println!("Error saving preset: {e}"));
            self.st.save_preset = false;
            self.st.show_preset_save_modal = false;
            println!("Saved Preset");
        }
        if self.st.load_preset {
            let Some(path) = &self.st.preset_load_path else {
                return;
            };
            let fstr = std::fs::read_to_string(path).unwrap();
            let stereometer: Stereometer = serde_json::from_str(&fstr).unwrap();
            self.st.load_preset = false;
            self.st.show_preset_load_modal = false;
            self.st.stereo = stereometer;
            println!("Loaded Preset");
        }

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

impl PolarityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        let mut st = AppState::default();
        theme::apply_theme(&cc.egui_ctx, st.dark_mode);
        setup_wgpu(&mut st, cc);
        Self {
            st,
            ..Default::default()
        }
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
            window_drag_tooltip(ui);
        }

        if let Some(start_time) = self.st.window_drag_tooltip_modal_deadline
            && Instant::now().duration_since(start_time) >= Duration::from_secs(5)
        {
            self.st.window_drag_tooltip_modal_open = false;
        }
    }
}
