use eframe::{
    egui::TextBuffer,
    egui_wgpu::{self, wgpu},
};
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent};
use std::{
    io::Write,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    Preset,
    audio::audio_player::*,
    generators::{
        Envelope,
        rendering::{
            OutputResources, RendererCallback, get_gpu_frame, run_effects_render_pipeline,
            run_output_render_pipeline, run_source_render_pipeline,
        },
    },
    state::PlaybackMode,
    ui::app_widgets::{main_window, menu_bar},
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

impl eframe::App for PolarityApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if !self.st.bool.fullscreen {
            menu_bar(&mut self.st, ui);
        }
        main_window(ui, &mut self.st, &mut self.player, frame);
    }
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        theme::apply_theme(ctx, self.st.bool.dark_mode);

        self.handle_audio_import();
        self.handle_playback(ctx);
        self.st.update_filters(self.player.as_ref());
        self.handle_preset_state();
        if self.st.bool.export_enabled {
            self.handle_file_export(ctx, frame);
        }
    }
}

impl PolarityApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        let mut st = AppState::default();
        theme::apply_theme(&cc.egui_ctx, st.bool.dark_mode);
        setup_wgpu(&mut st, cc);

        unsafe { std::env::set_var("KEEP_ONLY_FFMPEG", "true") };
        // cannot export without ffmpeg
        ffmpeg_sidecar::download::auto_download()
            .unwrap_or_else(|_| st.bool.export_enabled = false);

        Self {
            st,
            ..Default::default()
        }
    }

    fn load_file(&mut self, path: PathBuf) {
        let (paused, clear_env) = (true, false);
        if let Some(old_player) = &self.player {
            if *old_player.contents.path != path {
                self.spawn_audio_player(path, paused, clear_env);
            }
        } else {
            self.spawn_audio_player(path, paused, clear_env);
        }
    }

    fn spawn_audio_player(&mut self, path: PathBuf, paused: bool, clear_envelopes: bool) {
        // Clear old stuff
        self.player.take();
        self.st.stereo.clear_live_buffers();
        self.st.stereo.clear_trace_buffers();
        self.st.filterbank.live_fs_filters.take();
        self.st.filterbank.trace_fs_filters.take();
        self.st.filterbank.live_mb_filters.take();
        self.st.filterbank.trace_mb_filters.take();
        if clear_envelopes {
            self.st.env_a.take();
            self.st.env_b.take();
            self.st.env_c.take();
            self.st.env_d.take();
        }
        self.player = AudioPlayer::new(path, paused)
            .inspect_err(|err| println!("error creating audio player: {}", err))
            .ok();
        if let Some(p) = &self.player
            && self.st.env_a.is_none()
        {
            self.st.env_a = Some(Envelope::new(1., 100., 0.0, p.contents.sample_rate));
            self.st.env_b = Some(Envelope::new(1., 100., -0.40, p.contents.sample_rate));
            self.st.env_c = Some(Envelope::new(75., 800., 0.0, p.contents.sample_rate));
            self.st.env_d = Some(Envelope::new(1., 100., 0.0, p.contents.sample_rate));
        }
    }

    fn handle_playback(&mut self, ctx: &egui::Context) {
        // disable lazy refresh when audio loaded
        if let Some(p) = &self.player
            && !p.is_paused()
        {
            ctx.request_repaint_after_secs(Duration::from_millis(16).as_secs_f32());
        }

        if ctx.input(|i| i.key_pressed(Key::Space))
            && let Some(player) = &mut self.player
        {
            player.toggle_playback();
        }

        // Check if playback has ended
        if let Some(player) = &self.player
            && player.ended()
        {
            let paused = matches!(self.st.playback_mode, PlaybackMode::Once);
            self.spawn_audio_player(player.contents.path.to_path_buf(), paused, false);
        }
    }

    fn handle_audio_import(&mut self) {
        if self.st.bool.import_open {
            self.st.bool.import_open = false;
            self.st.bool.start_render = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio", &["wav", "mp3", "ogg", "m4a"])
                .pick_file()
            {
                self.load_file(path);
            };
        }
    }

    fn handle_preset_state(&mut self) {
        if self.st.bool.open_preset_save_file_picker {
            self.st.bool.open_preset_save_file_picker = false;
            self.st.bool.show_preset_save_modal = false;
            if let Some(save_path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_can_create_directories(true)
                .set_directory("~/")
                .save_file()
            {
                self.st.preset_save_path = Some(save_path)
            }
        }
        if self.st.bool.open_preset_load_file_picker {
            self.st.bool.open_preset_load_file_picker = false;
            self.st.bool.show_preset_load_modal = false;

            if let Some(load_path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_can_create_directories(true)
                .set_directory("~/")
                .pick_file()
            {
                self.st.preset_load_path = Some(load_path)
            }
        }

        if self.st.bool.save_preset {
            self.save_preset();
            self.st.bool.save_preset = false;
            self.st.bool.show_preset_save_modal = false;
        }
        if self.st.bool.load_preset {
            self.load_preset();
            self.st.bool.load_preset = false;
            self.st.bool.show_preset_load_modal = false;
        }
    }

    fn save_preset(&mut self) {
        let Ok(data) = serde_json::to_vec(&Preset {
            stereometer: self.st.stereo.clone(),
            fluidwave: self.st.fwave.clone(),
            oscilloscope: self.st.osci.clone(),
            polar_patterns: self.st.polar_pat.clone(),
        }) else {
            return;
        };
        let Some(path) = &self.st.preset_save_path else {
            return;
        };
        std::fs::write(path, data).unwrap_or_else(|e| println!("Error saving preset: {e}"));
    }
    fn load_preset(&mut self) {
        let Some(path) = &self.st.preset_load_path else {
            return;
        };
        let fstr = std::fs::read_to_string(path)
            .inspect_err(|e| println!("error opening preset: {e}"))
            .unwrap_or_default();
        let p: Preset = serde_json::from_str(&fstr)
            .inspect_err(|e| println!("error parsing preset: {e}"))
            .unwrap_or_default();
        self.st.stereo = p.stereometer;
        self.st.fwave = p.fluidwave;
        self.st.osci = p.oscilloscope;
        self.st.polar_pat = p.polar_patterns;
    }

    fn handle_file_export(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if self.st.bool.open_export_path_picker {
            self.st.bool.open_export_path_picker = false;
            self.st.bool.show_export_modal = false;

            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Video", &["mp4"])
                .set_can_create_directories(true)
                .save_file()
            {
                self.st.export_path.take();
                self.st.export_path = Some(path);
                self.st.bool.show_export_modal = true;
            }
        }

        if let Some(p) = &self.player
            && self.st.export_path.is_some()
            && (self.st.bool.start_render || self.st.bool.rendering)
        {
            if !self.st.bool.export_enabled {
                self.st.bool.start_render = false;
                self.st.bool.rendering = false;
                return;
            }

            self.st.bool.start_render = false;
            self.st.bool.rendering = true;
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
    let export_sample_idx = (frac * p.contents.sample_rate as f32) as usize;

    let (Some(env_a), Some(env_b), Some(env_c), Some(env_d)) =
        (&mut st.env_a, &mut st.env_b, &mut st.env_c, &mut st.env_d)
    else {
        return vec![0];
    };
    env_a.run_differential_follower(p, Some(export_sample_idx));
    env_b.run_differential_follower(p, Some(export_sample_idx));
    env_c.run_differential_follower(p, Some(export_sample_idx));
    env_d.run_differential_follower(p, Some(export_sample_idx));

    let mut fbank = std::mem::take(&mut st.filterbank);
    st.active_gen()
        .prepare(&mut fbank, p, Some(export_sample_idx));
    st.filterbank = fbank;

    let render_data = RendererCallback {
        canvas_size: vec2(w as f32, h as f32),
        params: st.build_renderer_callback_params(false, fps),
    };

    // Main pipeline
    run_source_render_pipeline(
        &render_data.params,
        device,
        queue,
        &mut command_encoder,
        &mut st.resources,
        dim,
    );
    let effects_data = st.build_effects_callback_params();
    run_effects_render_pipeline(
        &effects_data,
        device,
        queue,
        &mut command_encoder,
        &mut st.resources,
    );

    // Output
    let out_res = st.resources.get::<OutputResources>().unwrap();
    run_output_render_pipeline(&mut command_encoder, out_res);
    queue.submit(Some(command_encoder.finish()));
    get_gpu_frame(device, out_res)
}

fn spawn_ffmpeg_writer(st: &mut AppState, p: &AudioPlayer, fps: usize, dim: (u32, u32)) {
    let Some(output_path) = st.export_path.as_ref() else {
        return;
    };
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
        .args(["-y", output_path.to_string_lossy().as_str()])
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
        if st.cur_frame_idx >= st.export_config.total_frames || st.bool.export_canceled {
            drop(std::mem::take(&mut st.export_tx));
            st.writer_handle.take().unwrap().join().unwrap();
            st.logger_handle.take().unwrap().join().unwrap();
            st.bool.rendering = false;
            st.bool.show_export_modal = false;
            st.cur_frame_idx = 0;
            st.export_elapsed_time.take();
            st.prev_export_timestamp.take();
            st.export_config.total_frames = 0;
            st.bool.export_canceled = false;
            println!("Finished");
            break;
        }
        let frame = render_wgpu_frame(st, p, device, queue, fps, (w, h));
        st.export_tx.as_ref().unwrap().send(frame).unwrap();
        st.cur_frame_idx += 1;
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
            egui::Slider::new(&mut st.fwave.viscosity_amount, 0.00..=0.05)
                .text("viscosity_strength"),
        );
    });
}
