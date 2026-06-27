use biquad::*;
use eframe::{
    egui::Pos2,
    egui_wgpu::{self, wgpu},
};
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent};
use std::{
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    GeneratorKind,
    audio::{StereoFilter, audio_player::*},
    envelope_follower,
    generators::{
        rendering::{
            EffectsCallback, OutputResources, effects_render_pipeline, get_gpu_frame,
            get_texture_view, main_render_pipeline, output_render_pipeline,
        },
        stereometer::{FilterMode, Stereometer},
    },
    state::PlaybackMode,
    ui::{
        app_widgets::{main_window, menu_bar},
        canvas::get_render_callback_data,
    },
    wgpu_init::setup_wgpu,
};
use eframe::egui;
use eframe::egui::{Key, vec2};

use crate::{state::AppState, ui::theme};

#[derive(Default)]
pub struct PolarityApp {
    st: AppState,
    player: Option<AudioPlayer>,
}

const BATCH_SIZE: usize = 30;

fn render_wgpu_frame(
    st: &mut AppState,
    p: &AudioPlayer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    fps: usize,
    dim: (u32, u32),
) -> Vec<u8> {
    let (w, h) = (dim.0, dim.1);
    let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("export command encoder"),
    });
    let frac = st.cur_frame_idx as f32 / fps as f32;
    st.export_sample_idx = (frac * p.contents.sample_rate as f32) as usize;

    match st.gen_kind {
        GeneratorKind::Stereometer => st.stereo.draw(p, Some(st.export_sample_idx)),
        GeneratorKind::Fluidwave => envelope_follower(p, st, false, fps),
    }

    let render_data = get_render_callback_data(st, vec2(w as f32, h as f32), false, fps);
    let texture_view = get_texture_view(&mut st.resources, device, (w, h), st.gen_kind);

    // Main pipeline
    main_render_pipeline(
        &render_data,
        device,
        queue,
        &mut command_encoder,
        &mut st.resources,
        texture_view,
    );

    let effects_data = EffectsCallback {
        top_left: Pos2::ZERO,
        bloom_amt: match st.gen_kind {
            crate::GeneratorKind::Stereometer => st.stereo.bloom,
            crate::GeneratorKind::Fluidwave => st.fwave.bloom,
        },
    };
    effects_render_pipeline(
        &effects_data,
        device,
        queue,
        &mut command_encoder,
        &mut st.resources,
    );

    // Output
    let out_res = st.resources.get::<OutputResources>().unwrap();
    output_render_pipeline(&mut command_encoder, out_res);
    queue.submit(Some(command_encoder.finish()));
    get_gpu_frame(device, out_res)
}

fn spawn_ffmpeg_writer(st: &mut AppState, p: &AudioPlayer, fps: usize, dim: (u32, u32)) {
    let (w, h) = (dim.0, dim.1);
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

fn export_batched_frames(
    st: &mut AppState,
    p: &AudioPlayer,
    wgpu_render_state: &egui_wgpu::RenderState,
) {
    let fps = st.export_config.frame_rate.value();
    let canvas_size = st.export_config.resolution.value();
    let (w, h) = (canvas_size.0, canvas_size.1);

    // Spawn writer thread for entire job
    if st.writer_handle.is_none() {
        spawn_ffmpeg_writer(st, p, fps, (w, h));
    }

    let device = &wgpu_render_state.device;
    let queue = &wgpu_render_state.queue;

    for _ in 0..BATCH_SIZE {
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
        let frame = render_wgpu_frame(st, p, device, queue, fps, (w, h));
        st.export_tx.as_ref().unwrap().send(frame).unwrap();
        st.cur_frame_idx += 1;
    }
}

impl eframe::App for PolarityApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // debug_window(ui, &mut self.st);
        if !self.st.fullscreen {
            menu_bar(&mut self.st, ui);
        }
        main_window(ui, &mut self.st, &mut self.player, frame);
    }
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        theme::apply_theme(ctx, self.st.dark_mode);

        self.handle_audio_import(ctx);
        self.handle_playback(ctx);
        self.update_filters();
        self.handle_preset_state(ctx);
        self.handle_file_export(ctx, frame);
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

    fn load_file(&mut self, path: PathBuf) {
        if let Some(old_player) = &self.player {
            if *old_player.contents.path != path {
                println!("Diff");
                self.spawn_audio_player(path, true);
            }
        } else {
            self.spawn_audio_player(path, true);
        }
    }

    fn spawn_audio_player(&mut self, path: PathBuf, paused: bool) {
        // Clear old player
        self.player.take();
        self.st.stereo.clear_live_buffers();
        self.st.stereo.clear_trace_buffers();

        self.player = AudioPlayer::new(path, paused)
            .inspect_err(|err| println!("{}", err))
            .ok();
    }

    fn handle_playback(&mut self, ctx: &egui::Context) {
        // disable lazy refresh when audio loaded
        if let Some(p) = &self.player
            && !p.is_paused()
        {
            ctx.request_repaint_after_secs(Duration::from_millis(16).as_secs_f32());
        }

        if ctx.input(|i| i.key_pressed(Key::Space)) {
            if let Some(player) = &mut self.player {
                player.toggle_playback();
            } else {
                println!("No audio player loaded");
            }
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

    fn handle_audio_import(&mut self, ctx: &egui::Context) {
        // Open audio import dialog
        if self.st.import_open {
            self.st.file_dialog.pick_file();
            self.st.import_open = false;
            self.st.start_render = false;
        }

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
    }

    fn handle_preset_state(&mut self, ctx: &egui::Context) {
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
    }

    fn handle_file_export(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if let Some(p) = &self.player
            && (self.st.start_render || self.st.rendering)
        {
            self.st.start_render = false;
            self.st.rendering = true;
            let wgpu_render_state = frame
                .wgpu_render_state()
                .expect("error: wgpu unavailable on device");

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
    }
}

#[allow(dead_code)]
fn debug_window(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Window::new("Debug").show(ui.ctx(), |ui| {
        ui.add(egui::Slider::new(&mut st.fwave.gravity, -100.0..=100.0).text("gravity"));
        ui.add(egui::Slider::new(&mut st.fwave.pressure_multiplier, 0.0..=400.0).text("pressure"));
        ui.add(
            egui::Slider::new(&mut st.fwave.target_density, 0.0..=6000.0).text("target density"),
        );
        ui.add(egui::Slider::new(&mut st.fwave.smoothing_radius, 0.01..=1.0).text("radius"));
        ui.add(
            egui::Slider::new(&mut st.fwave.near_pressure_multiplier, 0.00..=10.0)
                .text("near pressure multiplier"),
        );
        ui.add(
            egui::Slider::new(&mut st.fwave.viscosity_strength, 0.00..=0.05)
                .text("viscosity_strength"),
        );
    });
}
