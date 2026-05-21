use bevy::asset::{RenderAssetUsages, UnapprovedPathMode};
use bevy::audio::Source;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy::text::{FontFeatureTag, FontFeatures};
use bevy_file_dialog::prelude::*;
use biquad::*;
use polarity::stereometer::{self, StereoFilter, Stereometer, StereometerKind};
use polarity::{
    AudioFileContents, CRT_P1, CRT_P7, DrawableCursor, HISTORY_MAGENTA, HISTORY_WINDOW_SIZE,
    HistoryMesh, LIVE_MAGENTA, LIVE_WINDOW_SIZE, LiveMesh, NUM_VERTICES, PlayingAudio,
    PreviewCanvas, TimelineScrubber, palette,
};
use std::collections::VecDeque;
use std::sync::Arc;

struct FontBlock {
    icon: Handle<Font>,
    text: Handle<Font>,
}

#[derive(States, Component, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum GeneratorChoice {
    #[default]
    Stereometer,
    Oscilloscope,
}

const SHADER_ASSET_PATH: &str = "shaders/custom_material.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    #[uniform(0)]
    color: LinearRgba,
    alpha_mode: AlphaMode2d,
}

impl Material2d for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        self.alpha_mode
    }
}

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
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..Default::default()
                }),
        )
        .add_plugins(Material2dPlugin::<CustomMaterial>::default())
        .add_plugins(
            FileDialogPlugin::new()
                .with_save_file::<AudioFileContents>()
                .with_load_file::<AudioFileContents>(),
        )
        .init_state::<GeneratorChoice>()
        .add_systems(OnEnter(GeneratorChoice::Stereometer), spawn_stereometer)
        .add_systems(OnExit(GeneratorChoice::Stereometer), despawn_stereometer)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (file_loaded, toggle_audio_playback, update_timeline_scrubber),
        )
        .add_systems(
            Update,
            (stereometer::update, stereometer::draw)
                .chain()
                .run_if(in_state(GeneratorChoice::Stereometer)),
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
                    c.0 = palette::DIM;
                }
            }
        }
    }
}

fn spawn_file_handle_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &FontBlock,
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
                height: px(38),
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
                TextColor(palette::DIM),
                TextFont {
                    font: fonts.icon.clone(),
                    font_size: palette::font_size::ICON,
                    ..Default::default()
                },
            ));
            parent.spawn((
                Text::new(button_text),
                TextColor(palette::DIM),
                TextFont {
                    font: fonts.text.clone(),
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
            height: percent(37.),
            width: percent(100.),
            border: UiRect {
                top: px(palette::FRAME_WIDTH),
                bottom: px(palette::FRAME_WIDTH),
                ..Default::default()
            },
            ..Default::default()
        },
        BackgroundColor(palette::BG_MED),
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
                padding: UiRect::all(px(4.)),
                ..Default::default()
            },
            BackgroundColor(palette::BG_MED),
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

fn spawn_primary_dropdown_headers(
    parent: &mut ChildSpawnerCommands,
    names: &[&str],
    fonts: &FontBlock,
) {
    names.iter().enumerate().for_each(|(i, &name)| {
        parent
            .spawn((
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    height: px(46),
                    width: percent(100.),
                    border: if i != 0 {
                        UiRect::top(px(1))
                    } else {
                        default()
                    },
                    ..Default::default()
                },
                BorderColor::all(palette::BORDER),
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            height: percent(100.),
                            width: px(46),
                            border: UiRect::right(px(1)),
                            ..Default::default()
                        },
                        BackgroundColor(palette::SURFACE),
                        BorderColor::all(palette::BORDER),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(format!("0{}", i)),
                            TextFont {
                                font: fonts.text.clone(),
                                font_size: palette::font_size::ICON,
                                weight: FontWeight(palette::font_weight::MED),
                                ..Default::default()
                            },
                        ));
                    });

                parent
                    .spawn((
                        Node {
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::FlexStart,
                            height: px(46.),
                            width: percent(100.),
                            padding: UiRect::left(px(12)).with_right(px(12)),
                            ..Default::default()
                        },
                        BackgroundColor(palette::SURFACE),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(name.to_string()),
                            TextFont {
                                font: fonts.text.clone(),
                                font_size: palette::font_size::ICON,
                                weight: FontWeight(palette::font_weight::MED),
                                ..Default::default()
                            },
                        ));
                    });
            });
    });
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
                        height: px(38.),
                        width: percent(100.),
                        border: UiRect {
                            left: px(palette::FRAME_WIDTH * 2.),
                            bottom: px(palette::FRAME_WIDTH),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    BackgroundColor(palette::BG),
                    BorderColor::all(palette::BORDER),
                ))
                .with_children(|parent| {
                    spawn_file_handle_button(
                        parent,
                        fonts,
                        "IMPORT",
                        "\u{e5db}",
                        |_: On<Pointer<Click>>, commands| import_file(commands),
                        true,
                    );
                    spawn_file_handle_button(
                        parent,
                        fonts,
                        "EXPORT",
                        "\u{e5d8}",
                        |_: On<Pointer<Click>>, commands| export_file(commands),
                        false,
                    );
                });
            // Inner div for control panel
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        align_items: AlignItems::Start,
                        flex_direction: FlexDirection::Column,
                        // justify_content: JustifyContent::Center,
                        height: percent(100.),
                        width: percent(100.),
                        border: UiRect {
                            left: px(palette::FRAME_WIDTH * 2.),
                            ..Default::default()
                        },
                        padding: UiRect::all(px(12.)),
                        ..Default::default()
                    },
                    BackgroundColor(palette::BG),
                    BorderColor::all(palette::BORDER),
                ))
                .with_children(|parent| {
                    spawn_primary_dropdown_headers(parent, &["GENERATOR", "MODIFIER"], fonts)
                });
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
        decoder_text.0 = format!("Song Duration: {:.2}s, ", duration.unwrap().as_secs_f64());
        timeline_scrubber.0 = duration;

        let file_contents = AudioFileContents {
            duration: audio_src.decoder().total_duration().unwrap().as_secs_f64(),
            sample_rate: audio_src.decoder().sample_rate(),
            num_channels: audio_src.decoder().channels() as usize,
            samples: audio_src
                .decoder()
                .map(|sample| sample as f32 / i16::MAX as f32)
                .collect(),
        };

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
    audio: Single<&AudioFileContents>,
    mut q_progress_text: Single<&mut Text, With<TimelineScrubber>>,
) {
    let pos = q_playing_audio.position().as_secs_f64() % audio.duration;
    q_progress_text.0 = format!("Time Elapsed: {:.2}s", pos,);
}

fn spawn_stereometer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let mut live_mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let mut history_mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let live_zeros: Vec<[f32; 3]> = vec![[0., 0., 0.]; LIVE_WINDOW_SIZE * NUM_VERTICES];
    let hist_zeros: Vec<[f32; 3]> = vec![[0., 0., 0.]; HISTORY_WINDOW_SIZE * NUM_VERTICES];
    let hist_colors: Vec<[f32; 4]> = (0..HISTORY_WINDOW_SIZE)
        .flat_map(|i| {
            let alpha = (i as f32 / HISTORY_WINDOW_SIZE as f32).powf(9.);
            let c = HISTORY_MAGENTA.with_alpha(alpha).to_f32_array();
            std::iter::repeat_n(c, 6)
        })
        .collect();
    live_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, live_zeros);
    live_mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![LIVE_MAGENTA.to_f32_array(); LIVE_WINDOW_SIZE * NUM_VERTICES],
    );
    history_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, hist_zeros);
    history_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, hist_colors);
    commands.spawn(());

    let goni_id = commands.spawn_empty().id();

    let fs = 48000.hz();
    let lpf_coeffs =
        Coefficients::<f32>::from_params(Type::LowPass, fs, 300.hz(), Q_BUTTERWORTH_F32).unwrap();
    let bpf_coeffs =
        Coefficients::<f32>::from_params(Type::BandPass, fs, 1.khz(), Q_BUTTERWORTH_F32).unwrap();
    let hpf_coeffs =
        Coefficients::<f32>::from_params(Type::HighPass, fs, 3.khz(), Q_BUTTERWORTH_F32).unwrap();
    commands.entity(goni_id).insert((
        Stereometer {
            kind: StereometerKind::default(),
            live_buffer: VecDeque::from([Vec2::ZERO; LIVE_WINDOW_SIZE]),
            history_buffer: VecDeque::from([Vec2::ZERO; HISTORY_WINDOW_SIZE]),
            last_sample_idx: 0,
            id: goni_id,
            filterbank: (
                StereoFilter::new(lpf_coeffs),
                StereoFilter::new(bpf_coeffs),
                StereoFilter::new(hpf_coeffs),
            ),
        },
        DrawableCursor,
    ));
    commands.spawn((
        LiveMesh,
        Mesh2d(meshes.add(live_mesh)),
        MeshMaterial2d(materials.add(CustomMaterial {
            color: LinearRgba::default(),
            alpha_mode: AlphaMode2d::Blend,
        })),
    ));
    commands.spawn((
        HistoryMesh,
        Mesh2d(meshes.add(history_mesh)),
        MeshMaterial2d(materials.add(CustomMaterial {
            color: LinearRgba::default(),
            alpha_mode: AlphaMode2d::Blend,
        })),
    ));
    info!("Spawned Goniometer");
}

fn despawn_stereometer(
    mut commands: Commands,
    q_goniometer: Single<&Stereometer, With<DrawableCursor>>,
) {
    commands.entity(q_goniometer.id).despawn();
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let fonts = FontBlock {
        icon: asset_server.load("fonts/material-symbols/MaterialSymbolsOutlined.ttf"),
        text: asset_server.load("fonts/inter/InterVariable.ttf"),
    };
    commands.spawn((Camera2d, Bloom::default()));
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
