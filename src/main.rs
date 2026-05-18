use polarity::goniometer::{self, Goniometer};
use polarity::palette;
use polarity::{
    AudioFileContents, DrawableCursor, PlayingAudio, PointArray, PreviewCanvas, TimelineScrubber,
    WINDOW_SIZE,
};
use std::collections::VecDeque;
use std::sync::Arc;

use bevy::asset::UnapprovedPathMode;
use bevy::audio::Source;
use bevy::prelude::*;
use bevy_file_dialog::prelude::*;

struct FontBlock {
    icon: Handle<Font>,
    text: Handle<Font>,
}

#[derive(States, Component, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum GeneratorChoice {
    #[default]
    Goniometer,
    Oscilloscope,
}

#[derive(Component)]
struct DurationText;

const POINT_SIZE: f32 = 0.75;

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
        .init_state::<GeneratorChoice>()
        .add_systems(OnEnter(GeneratorChoice::Goniometer), spawn_goniometer)
        .add_systems(OnExit(GeneratorChoice::Goniometer), despawn_goniometer)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (file_loaded, toggle_audio_playback, update_timeline_scrubber),
        )
        .add_systems(
            Update,
            (goniometer::update, goniometer::draw)
                .chain()
                .run_if(in_state(GeneratorChoice::Goniometer)),
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
                height: percent(100.),
                width: percent(100.),
                ..Default::default()
            },
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
            height: percent(25.),
            width: percent(100.),
            border: UiRect {
                top: px(palette::FRAME_WIDTH),
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
                height: percent(100.),
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
            ..Default::default()
        },
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
                width: percent(25.),
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

fn spawn_goniometer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut point_ids = Vec::new();
    point_ids.reserve_exact(WINDOW_SIZE);
    for i in 0..WINDOW_SIZE {
        let point = commands
            .spawn((
                Mesh2d(meshes.add(Circle::new(POINT_SIZE))),
                if i % 3 == 0 {
                    MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.0)))
                } else if i % 3 == 1 {
                    if i % 2 == 0 {
                        MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.0)))
                    } else {
                        MeshMaterial2d(materials.add(Color::srgb(0.0, 0.0, 1.0)))
                    }
                } else {
                    if i % 2 == 0 {
                        MeshMaterial2d(materials.add(Color::srgb(0.0, 0.0, 1.0)))
                    } else {
                        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 1.0)))
                    }
                },
                Transform::from_xyz(0., 0., 1.),
            ))
            .id();
        point_ids.push(point);
    }
    let goni_id = commands.spawn_empty().id();
    commands.entity(goni_id).insert((
        Goniometer {
            window_buffer: VecDeque::from([Vec2 { x: 0., y: 0. }; WINDOW_SIZE]),
            id: goni_id,
        },
        DrawableCursor,
        PointArray(point_ids),
    ));
    info!("Spawned Goniometer");
}

fn despawn_goniometer(
    mut commands: Commands,
    q_goniometer: Single<(&Goniometer, &PointArray), With<DrawableCursor>>,
) {
    let (goniometer, point_array) = *q_goniometer;
    for e in &point_array.0 {
        commands.entity(*e).despawn();
    }
    commands.entity(goniometer.id).despawn();
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let fonts = FontBlock {
        icon: asset_server.load("fonts/material-symbols/MaterialSymbolsOutlined.ttf"),
        text: asset_server.load("fonts/inter/InterVariable.ttf"),
    };
    commands.spawn(Camera2d);
    commands
        .spawn((Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            height: percent(100.),
            width: percent(100.),
            padding: UiRect::all(px(palette::APP_PADDING)),
            ..Default::default()
        },))
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
                    BorderColor::all(palette::BORDER),
                ))
                .with_children(|parent| {
                    // Top section holder for canvas + control panel
                    parent
                        .spawn((Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            height: percent(100.),
                            width: percent(100.),
                            ..Default::default()
                        },))
                        .with_children(|parent| {
                            spawn_control_panel(parent, &fonts);
                            spawn_preview_canvas(parent);
                        });
                    // Bottom section holder for timeline
                    parent
                        .spawn((Node {
                            display: Display::Flex,
                            height: percent(15.),
                            width: percent(100.),
                            ..Default::default()
                        },))
                        .with_children(spawn_timeline);
                });
        });
}
