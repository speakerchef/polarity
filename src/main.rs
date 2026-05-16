use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy_file_dialog::prelude::*;
mod palette;

struct AudioFileContents;

#[derive(Component)]
struct PlayingAudio;

fn main() {
    App::new()
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
        .add_systems(Update, (file_loaded, toggle_audio_playback))
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

fn load_file_dialog(mut commands: Commands) {
    commands
        .dialog()
        .set_directory("~")
        .add_filter("Audio", &["wav", "mp3", "m4a", "ogg", "flac"])
        .load_file::<AudioFileContents>();
}

fn file_loaded(
    mut event: MessageReader<DialogFileLoaded<AudioFileContents>>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    for f in event.read() {
        commands.spawn((
            PlayingAudio,
            AudioPlayer::new(asset_server.load(f.path.clone())),
            PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Loop,
                ..Default::default()
            },
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

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let inter: Handle<Font> = asset_server.load("fonts/inter/InterVariable.ttf");
    let icon_font: Handle<Font> =
        asset_server.load("fonts/material-symbols/MaterialSymbolsOutlined.ttf");
    commands.spawn(Camera2d);
    // root box
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
            BackgroundColor(palette::VOID),
        ))
        .with_children(|parent| {
            // main app base
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        height: percent(100.),
                        width: percent(100.),
                        border: UiRect::all(px(palette::FRAME_WIDTH)),
                        ..Default::default()
                    },
                    BackgroundColor(palette::BG),
                    BorderColor::all(palette::BORDER),
                ))
                .with_children(|parent| {
                    // control panel sidebar base
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
                                        icon_font.clone(),
                                        inter.clone(),
                                        "IMPORT",
                                        "\u{e5db}",
                                        |event: On<Pointer<Click>>, commands| {
                                            load_file_dialog(commands)
                                        },
                                        true,
                                    );
                                    spawn_file_handle_button(
                                        parent,
                                        icon_font.clone(),
                                        inter.clone(),
                                        "EXPORT",
                                        "\u{e5d8}",
                                        |event: On<Pointer<Click>>, commands| {
                                            load_file_dialog(commands)
                                        },
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
                    // Preview Canvas
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0.),
                            height: percent(100.),
                            width: percent(75.),
                            border: UiRect {
                                left: px(palette::FRAME_WIDTH),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        BackgroundColor(palette::VOID),
                    ));
                    parent
                        .spawn((
                            // Timeline base div
                            Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                position_type: PositionType::Absolute,
                                bottom: px(0.),
                                height: percent(13.),
                                width: percent(100.),
                                ..Default::default()
                            },
                            BackgroundColor(palette::BORDER),
                        ))
                        .with_children(|parent| {
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
                            parent.spawn((
                                // track waveform view section
                                Node {
                                    height: percent(77.),
                                    width: percent(100.),
                                    ..Default::default()
                                },
                                BackgroundColor(palette::BG_DARK),
                            ));
                        });
                });
        });
}
