use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use bevy::asset::UnapprovedPathMode;
use bevy::audio::Source;
use bevy::prelude::*;
use bevy_file_dialog::prelude::*;
mod palette;

struct FontBlock {
    icon: Handle<Font>,
    text: Handle<Font>,
}

#[derive(Component, Debug)]
struct AudioFileContents {
    duration: f32,
    sample_rate: u32,
    num_channels: usize,
    samples: Vec<f32>,
}

#[derive(Component, Debug, Clone)]
struct Goniometer(VecDeque<f32>);

#[derive(Component, Debug, Clone)]
struct PointArray(Vec<Entity>);

#[derive(Component)]
struct PlayingAudio;

#[derive(Component)]
struct TimelineScrubber(Option<Duration>);

#[derive(Component)]
struct DrawableCursor;

#[derive(Component)]
struct PreviewCanvas;

#[derive(Component)]
struct DurationText;

fn main() {
    App::new()
        .insert_resource(ClearColor(palette::VOID))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Polarity".into(),
                        // mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..Default::default()
                }),
        )
        .add_plugins(
            FileDialogPlugin::new()
                .with_save_file::<AudioFileContents>()
                .with_load_file::<AudioFileContents>(),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                file_loaded,
                toggle_audio_playback,
                update_timeline_scrubber,
                update_goniometer_data,
                draw_goniometer_points,
            ),
        )
        .run();
}

fn button_on_hover(
    event: On<Pointer<Over>>,
    mut q_bgcol: Query<&mut BackgroundColor, With<Button>>,
    mut q_textcol: Query<&mut TextColor>,
    q_children: Query<&Children>,
) {
    let target = event.event_target();
    if let Ok(mut bgcol) = q_bgcol.get_mut(target) {
        bgcol.0 = palette::BRIGHT;
        if let Ok(children) = q_children.get(target) {
            for child in children {
                if let Ok(mut c) = q_textcol.get_mut(*child) {
                    c.0 = palette::INK;
                }
            }
        }
    }
}
fn button_on_leave(
    event: On<Pointer<Out>>,
    mut q_bgcol: Query<&mut BackgroundColor, With<Button>>,
    mut q_textcol: Query<&mut TextColor>,
    q_children: Query<&Children>,
) {
    let target = event.event_target();
    if let Ok(mut bgcol) = q_bgcol.get_mut(target) {
        bgcol.0 = palette::BG_MED;
        if let Ok(children) = q_children.get(target) {
            for child in children {
                if let Ok(mut c) = q_textcol.get_mut(*child) {
                    c.0 = palette::TEXT;
                }
            }
        }
    }
}

fn spawn_file_handle_button(
    parent: &mut ChildSpawnerCommands,
    icon_font: Handle<Font>,
    text_font: Handle<Font>,
    button_text: &str,
    arrow_glyph: &str,
    on_click: fn(On<Pointer<Click>>, Commands),
    add_border: bool,
) {
    parent
        .spawn((
            Button,
            Node {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                height: percent(100.),
                width: percent(50.),
                border: if add_border {
                    UiRect {
                        right: px(palette::FRAME_WIDTH),
                        ..Default::default()
                    }
                } else {
                    UiRect::default()
                },
                ..Default::default()
            },
            BackgroundColor(palette::BG_MED),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(arrow_glyph),
                TextColor(palette::TEXT),
                TextFont {
                    font: icon_font.clone(),
                    font_size: palette::font_size::ICON,
                    ..Default::default()
                },
            ));
            parent.spawn((
                Text::new(button_text),
                TextColor(palette::TEXT),
                TextFont {
                    font: text_font.clone(),
                    font_size: palette::font_size::BRAND,
                    weight: FontWeight(palette::font_weight::HEAVY),
                    ..Default::default()
                },
            ));
        })
        .observe(button_on_hover)
        .observe(button_on_leave)
        .observe(on_click);
}

fn spawn_timeline(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            // Timeline base div
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                height: percent(100.),
                width: percent(100.),
                ..Default::default()
            },
            BackgroundColor(palette::BORDER),
        ))
        .with_children(|parent| {
            spawn_transport_info(parent);
            spawn_waveform_view(parent);
        });
}

fn spawn_transport_info(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        // transport and file info bar
        Node {
            height: percent(21.),
            width: percent(100.),
            border: UiRect {
                bottom: px(palette::FRAME_WIDTH),
                ..Default::default()
            },
            ..Default::default()
        },
        BackgroundColor(palette::BG),
        BorderColor::all(palette::BORDER),
    ));
}

fn spawn_waveform_view(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            // track waveform view section
            Node {
                height: percent(77.),
                width: percent(100.),
                ..Default::default()
            },
            BackgroundColor(palette::BG_DARK),
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("Song Duration: , "), DurationText));
            parent.spawn((Text::new("Time Elapsed: 0.0000s"), TimelineScrubber(None)));
        });
}

fn spawn_preview_canvas(parent: &mut ChildSpawnerCommands) {
    // Preview Canvas
    parent.spawn((
        PreviewCanvas,
        Node {
            height: percent(100.),
            width: percent(75.),
            border: UiRect {
                left: px(palette::FRAME_WIDTH),
                ..Default::default()
            },
            ..Default::default()
        },
        // BackgroundColor(palette::VOID),
    ));
}

fn spawn_control_panel(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    // control panel base
    parent
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                right: px(0.),
                height: percent(100.),
                width: px(360.),
                padding: UiRect {
                    left: px(palette::spacing::S1),
                    ..Default::default()
                },
                border: UiRect {
                    left: px(palette::FRAME_WIDTH),
                    ..Default::default()
                },
                ..Default::default()
            },
            BackgroundColor(palette::BG),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            // Import/Export button holder
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        height: percent(5.),
                        width: percent(100.),
                        border: UiRect {
                            left: px(palette::FRAME_WIDTH),
                            bottom: px(palette::FRAME_WIDTH),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    BackgroundColor(palette::BG_MED),
                    BorderColor::all(palette::BORDER),
                ))
                .with_children(|parent| {
                    spawn_file_handle_button(
                        parent,
                        fonts.icon.clone(),
                        fonts.text.clone(),
                        "IMPORT",
                        "\u{e5db}",
                        |event: On<Pointer<Click>>, commands| import_file(commands),
                        true,
                    );
                    spawn_file_handle_button(
                        parent,
                        fonts.icon.clone(),
                        fonts.text.clone(),
                        "EXPORT",
                        "\u{e5d8}",
                        |event: On<Pointer<Click>>, commands| export_file(commands),
                        false,
                    );
                });
            // Inner div for control panel
            parent.spawn((
                Node {
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    height: percent(100.),
                    width: percent(100.),
                    border: UiRect {
                        left: px(palette::FRAME_WIDTH),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                BackgroundColor(palette::BG_MED),
                BorderColor::all(palette::BORDER),
            ));
        });
}

fn import_file(mut commands: Commands) {
    commands
        .dialog()
        .set_directory("~")
        .add_filter("Audio", &["wav", "mp3", "m4a", "ogg", "flac"])
        .load_file::<AudioFileContents>();
}

fn export_file(mut commands: Commands) {
    // TODO: Load file byte contents
    // commands.dialog().save_file::<AudioFileContents>();
}

fn file_loaded(
    mut event: MessageReader<DialogFileLoaded<AudioFileContents>>,
    mut timeline_scrubber: Single<&mut TimelineScrubber>,
    mut decoder_text: Single<&mut Text, With<DurationText>>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    for f in event.read() {
        let bytes: Arc<[u8]> = f.contents.clone().into();
        let audio_src = AudioSource { bytes };
        let duration = audio_src.decoder().total_duration();
        decoder_text.0 = format!("Song Duration: {}s, ", duration.unwrap().as_secs_f32());
        timeline_scrubber.0 = duration;

        let file_contents = AudioFileContents {
            duration: audio_src.decoder().total_duration().unwrap().as_secs_f32(),
            sample_rate: audio_src.decoder().sample_rate(),
            num_channels: audio_src.decoder().channels() as usize,
            samples: audio_src
                .decoder()
                .map(|sample| sample as f32 / i16::MAX as f32)
                .collect(),
        };

        // info!("File contents: {:#?}", file_contents);

        commands.spawn((
            PlayingAudio,
            AudioPlayer::new(asset_server.load(f.path.clone())),
            PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Loop,
                ..Default::default()
            },
            file_contents,
        ));
        info!("Opened file `{}`", f.file_name);
    }
}

fn toggle_audio_playback(
    press: Res<ButtonInput<KeyCode>>,
    q_audio: Query<&AudioSink, With<PlayingAudio>>,
) {
    if press.just_pressed(KeyCode::Space)
        && let Ok(sink) = q_audio.single()
    {
        sink.toggle_playback();
    }
}

fn update_timeline_scrubber(
    q_playing_audio: Single<&AudioSink, With<PlayingAudio>>,
    mut q_progress_text: Single<&mut Text, With<TimelineScrubber>>,
) {
    let pos = q_playing_audio.position().as_secs_f32();
    q_progress_text.0 = format!("Time Elapsed: {:.2}s", pos,);
}

fn update_goniometer_data(
    q_playing_audio: Single<&AudioSink, With<PlayingAudio>>,
    q_audio_data: Single<&AudioFileContents>,
    mut q_goni: Single<&mut Goniometer, With<DrawableCursor>>,
    q_canvas: Single<&UiGlobalTransform, With<PreviewCanvas>>,
    q_camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let canvas_2d = q_canvas.translation;
    let (camera, camera_xform) = *q_camera;

    let pos = q_playing_audio.position().as_secs_f64();
    let sample_idx = std::cmp::min(
        (q_audio_data.sample_rate as f64 * pos) as usize,
        q_audio_data.samples.len() - q_audio_data.sample_rate as usize,
    );

    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_xform, canvas_2d) {
        q_goni.0.push_back(
            world_pos.x + q_audio_data.samples[sample_idx * q_audio_data.num_channels] * 100.,
        );
        q_goni.0.push_back(
            world_pos.y + q_audio_data.samples[sample_idx * q_audio_data.num_channels + 1] * 100.,
        );
    }
    while q_goni.0.len() > 512 {
        q_goni.0.pop_front();
        q_goni.0.pop_front();
    }
}

fn draw_goniometer_points(
    q_cursor: Single<(&Goniometer, &mut PointArray), With<DrawableCursor>>,
    mut q_points: Query<&mut Transform>,
) {
    let goniometer = q_cursor.0;

    for (i, entity) in q_cursor.1.0.iter().enumerate() {
        if let Ok(mut transform) = q_points.get_mut(*entity) {
            transform.translation.x = goniometer.0[i * 2];
            transform.translation.y = goniometer.0[i * 2 + 1];
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let fonts = FontBlock {
        icon: asset_server.load("fonts/material-symbols/MaterialSymbolsOutlined.ttf"),
        text: asset_server.load("fonts/inter/InterVariable.ttf"),
    };
    let mut point_ids = Vec::new();
    point_ids.reserve_exact(256);
    for _ in 0..256 {
        let point = commands
            .spawn((
                Mesh2d(meshes.add(Circle::new(2.0))),
                MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
                Transform::from_xyz(0., 0., 1.),
            ))
            .id();
        point_ids.push(point);
    }
    commands.spawn(Camera2d);
    commands.spawn((
        Goniometer(VecDeque::from(vec![0.; 512])),
        DrawableCursor,
        PointArray(point_ids),
    ));
    commands
        .spawn((
            Node {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                height: percent(100.),
                width: percent(100.),
                padding: UiRect::all(px(palette::APP_PADDING)),
                ..Default::default()
            },
            // BackgroundColor(palette::VOID),
        ))
        .with_children(|parent| {
            // main app base
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        height: percent(100.),
                        width: percent(100.),
                        border: UiRect::all(px(palette::FRAME_WIDTH)),
                        ..Default::default()
                    },
                    // BackgroundColor(palette::BG),
                    BorderColor::all(palette::BORDER),
                ))
                .with_children(|parent| {
                    // Top section holder for canvas + control panel
                    parent
                        .spawn((
                            Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                height: percent(100.),
                                width: percent(100.),
                                border: UiRect::all(px(palette::FRAME_WIDTH)),
                                ..Default::default()
                            },
                            // BackgroundColor(palette::BG),
                            BorderColor::all(palette::BORDER),
                        ))
                        .with_children(|parent| {
                            spawn_control_panel(parent, &fonts);
                            spawn_preview_canvas(parent);
                        });
                    // Bottom section holder for timeline
                    parent
                        .spawn((
                            Node {
                                display: Display::Flex,
                                height: percent(15.),
                                width: percent(100.),
                                border: UiRect::all(px(palette::FRAME_WIDTH)),
                                ..Default::default()
                            },
                            // BackgroundColor(palette::BG),
                            BorderColor::all(palette::BORDER),
                        ))
                        .with_children(spawn_timeline);
                });
        });
}
