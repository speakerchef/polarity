use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{
    Preset,
    audio::audio_inputs::*,
    export_pipeline,
    generators::Envelope,
    get_audio_capture_permission,
    state::{InputMode, PlaybackMode},
    ui::app_widgets::{main_window, menu_bar},
    wgpu_init::setup_wgpu,
};
use eframe::egui;
use eframe::egui::Key;

use crate::{state::AppState, ui::theme};

#[derive(Default)]
pub struct PolarityApp {
    st: AppState,
}

const BATCH_SIZE: usize = 30;

impl eframe::App for PolarityApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if !self.st.bool.fullscreen {
            menu_bar(&mut self.st, ui);
        }
        main_window(ui, &mut self.st, frame);
    }
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        theme::apply_theme(ctx, self.st.bool.dark_mode);

        self.handle_live_audio_stream();
        self.handle_audio_import();
        self.handle_playback(ctx);
        self.st.update_filters();
        self.st.update_envelopes();
        self.handle_preset_state();
        self.handle_file_export(ctx, frame);
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

        st.audio_capture_permission = get_audio_capture_permission();
        st.live_input_device = st.live_input.start_stream(1024, None).unwrap().into();
        st.new_live_input_device = st.live_input_device.clone();
        st.default_live_input_device = st.live_input_device.clone();

        Self { st }
    }

    fn handle_live_audio_stream(&mut self) {
        self.st.update_live_input_device();
        self.st.live_input.update_buffer();
    }

    fn load_file(&mut self, path: PathBuf) {
        let (paused, clear_env) = (true, false);
        if let Some(old_player) = &self.st.player {
            if *old_player.contents.path != path {
                self.spawn_audio_player(path, paused, clear_env);
            }
        } else {
            self.spawn_audio_player(path, paused, clear_env);
        }
    }

    fn spawn_audio_player(&mut self, path: PathBuf, paused: bool, clear_envelopes: bool) {
        // Clear old stuff
        let st = &mut self.st;

        st.player.take();
        st.stereo.clear_live_buffers();
        st.stereo.clear_trace_buffers();

        if clear_envelopes {
            st.clear_envelopes();
        }
        st.player = AudioPlayer::new(path, paused)
            .inspect_err(|err| println!("error creating audio player: {}", err))
            .ok();

        let env = &mut st.env_bank;
        if let Some(p) = &st.player
            && env.env_a.is_none()
        {
            env.env_a = Some(Envelope::new(1., 100., 0.0, p.contents.sample_rate));
            env.env_b = Some(Envelope::new(1., 100., -0.40, p.contents.sample_rate));
            env.env_c = Some(Envelope::new(75., 800., 0.0, p.contents.sample_rate));
            env.env_d = Some(Envelope::new(1., 100., 0.0, p.contents.sample_rate));
        }
    }

    fn handle_playback(&mut self, ctx: &egui::Context) {
        let Some(player) = self.st.player.as_ref() else {
            return;
        };
        if self.st.input_mode == InputMode::Live {
            player.pause();
        }

        if ctx.input(|i| i.key_pressed(Key::Space)) {
            player.toggle_playback();
        }

        if player.ended() {
            let paused = matches!(self.st.playback_mode, PlaybackMode::Once);
            self.spawn_audio_player(player.contents.path.to_path_buf(), paused, false);
        }
    }

    fn handle_audio_import(&mut self) {
        if let Ok(dev) = std::env::var("DEV")
            && dev == "true"
            && !self.st.bool.debug_file_loaded
            && let Ok(path) = std::env::var("DEBUG_AUDIO_FILE_PATH")
        {
            println!("{path}");
            self.load_file(path.into());
            self.st.bool.debug_file_loaded = true;
            return;
        }

        if self.st.bool.import_open {
            self.st.bool.import_open = false;
            self.st.bool.start_render = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio", &["wav", "mp3", "ogg", "m4a", "flac"])
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
            gen_kind: self.st.gen_kind,
            stereometer: self.st.stereo.clone(),
            fluidwave: self.st.fwave.clone(),
            oscilloscope: self.st.osci.clone(),
            polar_patterns: self.st.polar_pat.clone(),
            cymatics: self.st.cymatics.clone(),
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
        self.st.gen_kind = p.gen_kind;
        self.st.stereo = p.stereometer;
        self.st.fwave = p.fluidwave;
        self.st.osci = p.oscilloscope;
        self.st.polar_pat = p.polar_patterns;
        self.st.cymatics = p.cymatics;
    }

    fn handle_file_export(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if !self.st.bool.export_enabled {
            return;
        }
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

        if self.st.player.is_some()
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
            let Some(wgpu_render_state) = frame.wgpu_render_state() else {
                println!("error: wgpu not available on this device");
                return;
            };

            if let (Some(t), Some(start_point)) = (
                self.st.export_elapsed_time.as_mut(),
                self.st.prev_export_timestamp,
            ) {
                *t = Instant::now().duration_since(start_point);
            } else {
                self.st.prev_export_timestamp = Some(Instant::now());
                self.st.export_elapsed_time = Some(Duration::default());
            }

            export_pipeline::export_batched_frames(&mut self.st, wgpu_render_state, BATCH_SIZE);
            ctx.request_repaint();
        }
    }
}

#[allow(unused)]
fn debug_window(ui: &mut egui::Ui, st: &mut AppState) {
    egui::Window::new("Debug").show(ui.ctx(), |ui| {});
}
