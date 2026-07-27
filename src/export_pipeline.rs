use std::io::Write;

use eframe::{egui::vec2, egui_wgpu};
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent};

use crate::{
    HardwareEncoder,
    generators::rendering::{
        OutputResources, RendererCallback, get_gpu_frame, run_effects_render_pipeline,
        run_output_render_pipeline, run_source_render_pipeline,
    },
    state::AppState,
};

pub fn render_wgpu_frame(
    st: &mut AppState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    fps: usize,
    dim: (u32, u32),
) -> Vec<u8> {
    let (w, h) = (dim.0, dim.1);
    let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("export command encoder"),
    });

    // Cannot have live input for export
    let Some(player) = st.player.take() else {
        return vec![0];
    };

    let frac = st.cur_frame_idx as f32 / fps as f32;
    let export_sample_idx = (frac * player.contents.sample_rate as f32) as usize;

    st.env_bank.run_follower(&player, Some(export_sample_idx));

    let mut fbank = std::mem::take(&mut st.filterbank);
    let env_bank = std::mem::take(&mut st.env_bank);

    st.active_gen()
        .prepare(&mut fbank, &env_bank, &player, Some(export_sample_idx));

    st.filterbank = fbank;
    st.env_bank = env_bank;
    st.player = Some(player);

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

pub fn spawn_ffmpeg_writer(st: &mut AppState, fps: usize, dim: (u32, u32)) {
    let Some(output_path) = st.export_path.as_ref() else {
        return;
    };
    let p = st.player.as_ref().expect("check at handle_file_export");
    let (w, h) = (dim.0, dim.1);
    let quality = st.export_config.quality.value();
    let total_frames = p.contents.duration.as_secs_f32() * fps as f32;
    let pix_fmt = st.export_config.pix_fmt;

    let mut hardware_encoder = None;
    if let Ok(encoders) = std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
    {
        for e in HardwareEncoder::ALL {
            if let Ok(s) = str::from_utf8(&encoders.stdout)
                && s.contains(e.label())
            {
                hardware_encoder = Some(e);
            }
        }
    }

    let encoder = if let Some(e) = hardware_encoder
        && st.export_config.use_hw_encoder
    {
        e.label()
    } else {
        "libx264"
    };

    println!("{encoder}");

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
        .pix_fmt(pix_fmt.as_ffmpeg_arg())
        .codec_audio("aac")
        .args(["-b:a", "320k"])
        .args(["-y", &output_path.to_string_lossy()])
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

pub fn export_batched_frames(
    st: &mut AppState,
    wgpu_render_state: &egui_wgpu::RenderState,
    batch_size: usize,
) {
    let fps = st.export_config.frame_rate.value();
    let canvas_size = st.export_config.resolution.value();
    let (w, h) = (canvas_size.0, canvas_size.1);

    // Spawn writer thread for entire job
    if st.writer_handle.is_none() {
        spawn_ffmpeg_writer(st, fps, (w, h));
    }

    let device = &wgpu_render_state.device;
    let queue = &wgpu_render_state.queue;

    for _ in 0..batch_size {
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
            if st.export_config.open_after {
                let (open_cmd, args) = cfg_select! {
                    target_os = "macos" => ("open", [""]),
                    target_os = "windows" => ( "cmd" , ["start", ""]),
                    target_os = "linux" => ( "xdg-open" , [""]),
                };

                let _status = std::process::Command::new(open_cmd)
                    .args(args)
                    .arg(st.export_path.as_ref().unwrap())
                    .status();
            }
            break;
        }
        let frame = render_wgpu_frame(st, device, queue, fps, (w, h));
        st.export_tx.as_ref().unwrap().send(frame).unwrap();
        st.cur_frame_idx += 1;
    }
}
