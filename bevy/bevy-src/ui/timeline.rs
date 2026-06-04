use bevy::prelude::*;

use crate::{
    AudioFileContents, FontBlock, NullComponent, UserPlaybackMode, palette,
    ui::interactions::{on_click_toggle_bright_bg, on_hover_bg, on_leave_bg},
};

#[derive(Component)]
pub struct DurationText;

#[derive(Component)]
pub struct PlayingAudio;

#[derive(Component)]
pub struct TimelineScrubber(pub Option<std::time::Duration>);

#[derive(Component, Clone)]
struct SkipToStart;
#[derive(Component, Clone)]
pub struct PlayPause;
#[derive(Component, Clone)]
struct SkipToEnd;

#[derive(Component, Clone)]
pub struct FilePathText;
#[derive(Component, Clone)]
pub struct SampleRateInfoText;

#[derive(Component, Clone)]
pub struct LoopToggle;

#[derive(Event)]
pub struct AudioReloadEvent;

pub fn spawn_timeline(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            // Timeline base div
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                height: px(104.),
                width: percent(100.),
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            spawn_transport_info(parent, fonts);
            spawn_waveform_view(parent);
        });
}

fn skip_start(fonts: &FontBlock) -> impl Bundle {
    (
        SkipToStart,
        Node {
            height: percent(80),
            width: px(28),
            padding: UiRect::top(px(2)).with_left(px(5)),

            ..Default::default()
        },
        BackgroundColor(palette::BG),
        Text::new("\u{e045}"),
        TextLayout::no_wrap(),
        TextColor(palette::DIM),
        TextFont {
            font: fonts.icon.clone(),
            font_size: FontSize::Px(palette::font_size::ICON),
            weight: FontWeight(palette::font_weight::BODY),
            ..Default::default()
        },
    )
}

fn play_pause(fonts: &FontBlock) -> impl Bundle {
    (
        PlayPause,
        Node {
            height: percent(80),
            width: px(28),
            padding: UiRect::top(px(2)).with_left(px(5)),

            ..Default::default()
        },
        BackgroundColor(palette::BG),
        Text::new("\u{e037}"),
        TextLayout::no_wrap(),
        TextColor(palette::LIVE),
        TextFont {
            font: fonts.icon.clone(),
            font_size: FontSize::Px(palette::font_size::ICON + 1.),
            weight: FontWeight(palette::font_weight::BODY),
            ..Default::default()
        },
    )
}

pub fn update_playback_indicator(
    sink: Single<&AudioSink, With<PlayingAudio>>,
    mut icon: Single<(&mut Text, &mut TextColor), With<PlayPause>>,
) {
    if sink.is_paused() {
        icon.0.0 = "\u{e037}".to_string();
        icon.1.0 = palette::LIVE;
    } else {
        icon.0.0 = "\u{e034}".to_string();
        icon.1.0 = palette::WARN;
    }
}

fn skip_end(fonts: &FontBlock) -> impl Bundle {
    (
        SkipToEnd,
        Node {
            height: percent(80),
            width: px(28),
            padding: UiRect::top(px(2)).with_left(px(5)),

            ..Default::default()
        },
        BackgroundColor(palette::BG),
        Text::new("\u{e044}"),
        TextLayout::no_wrap(),
        TextColor(palette::DIM),
        TextFont {
            font: fonts.icon.clone(),
            font_size: FontSize::Px(palette::font_size::ICON),
            weight: FontWeight(palette::font_weight::BODY),
            ..Default::default()
        },
    )
}

fn meta_text<M: Component>(
    font: FontSource,
    default_text: &str,
    marker: M,
    col: Color,
    fs: Option<f32>,
) -> impl Bundle {
    (
        marker,
        Text::new(default_text),
        TextLayout::no_wrap(),
        TextColor(col),
        TextFont {
            font,
            font_size: FontSize::Px(if let Some(fs) = fs {
                fs
            } else {
                palette::font_size::META
            }),
            weight: FontWeight(palette::font_weight::BODY),
            ..Default::default()
        },
    )
}

fn spawn_timecode(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            Node {
                height: percent(100.),
                width: px(96),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::top(px(1)),
                border: UiRect::all(px(1)),
                ..Default::default()
            },
            BackgroundColor(palette::VOID),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: px(50),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(meta_text(
                        fonts.text.clone(),
                        "00:00",
                        TimelineScrubber(None),
                        palette::BRIGHT,
                        Some(palette::font_size::BODY),
                    ));
                });
            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: px(2),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(meta_text(
                        fonts.text.clone(),
                        " / ",
                        NullComponent,
                        palette::BRIGHT,
                        Some(palette::font_size::BODY),
                    ));
                });
            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: px(50),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(meta_text(
                        fonts.text.clone(),
                        "00:00",
                        DurationText,
                        palette::BRIGHT,
                        Some(palette::font_size::BODY),
                    ));
                });
        });
}

fn loop_toggle_on_click(_: On<Pointer<Click>>, mut usr_pb: ResMut<UserPlaybackMode>) {
    *usr_pb = match *usr_pb {
        UserPlaybackMode::Once => UserPlaybackMode::Loop,
        _ => UserPlaybackMode::Once,
    };
}

fn spawn_loop_toggle(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            Node {
                height: percent(100.),
                width: px(76),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::top(px(1)),
                border: UiRect::all(px(1)),
                ..Default::default()
            },
            BackgroundColor(palette::BG),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    LoopToggle,
                    Node {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        column_gap: px(4),
                        width: px(50),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(meta_text(
                        fonts.text.clone(),
                        "LOOP",
                        NullComponent,
                        palette::BRIGHT,
                        Some(palette::font_size::BODY),
                    ));
                    parent.spawn(meta_text(
                        fonts.icon.clone(),
                        "\u{e042}",
                        NullComponent,
                        palette::BRIGHT,
                        Some(palette::font_size::MED),
                    ));
                });
        })
        .observe(on_hover_bg)
        .observe(on_leave_bg)
        .observe(on_click_toggle_bright_bg)
        .observe(loop_toggle_on_click);
}

fn skip_start_on_click(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    audio: Single<&mut AudioSink, With<PlayingAudio>>,
) {
    if audio.empty() {
        commands.trigger(AudioReloadEvent);
    }
    audio
        .try_seek(std::time::Duration::from_secs(0))
        .unwrap_or_else(|e| info!("{}", e));
}

fn toggle_playback_mode(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    sink: Single<&mut AudioSink, With<PlayingAudio>>,
    fc: Single<&AudioFileContents>,
) {
    if sink.empty() || (sink.position() >= fc.duration) {
        sink.pause();
        commands.trigger(AudioReloadEvent);
        return;
    }
    sink.toggle_playback();
}

fn skip_end_on_click(
    _: On<Pointer<Click>>,
    audio: Single<&mut AudioSink, With<PlayingAudio>>,
    fc: Single<&AudioFileContents>,
) {
    audio
        .try_seek(fc.duration)
        .unwrap_or_else(|e| info!("{}", e));
}

fn spawn_playback_controls(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            height: percent(100.),
            column_gap: px(12),
            ..Default::default()
        })
        .with_children(|parent| {
            // Playback controls
            parent
                .spawn(skip_start(fonts))
                .observe(on_hover_bg)
                .observe(on_leave_bg)
                .observe(skip_start_on_click);

            parent
                .spawn(play_pause(fonts))
                .observe(on_hover_bg)
                .observe(on_leave_bg)
                .observe(toggle_playback_mode);
            parent
                .spawn(skip_end(fonts))
                .observe(on_hover_bg)
                .observe(on_leave_bg)
                .observe(skip_end_on_click);
        });
}

pub fn spawn_transport_info(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            // transport and file info bar
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                height: px(32.),
                width: percent(100.),
                padding: UiRect::horizontal(px(12.)),
                border: UiRect {
                    top: px(palette::FRAME_WIDTH),
                    bottom: px(palette::FRAME_WIDTH),
                    ..Default::default()
                },
                ..Default::default()
            },
            BackgroundColor(palette::BG_MED),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    height: percent(100.),
                    column_gap: px(12),
                    flex_grow: 1.0,
                    ..Default::default()
                })
                .with_children(|parent| {
                    spawn_playback_controls(parent, fonts);
                    spawn_timecode(parent, fonts);
                    // File info
                    parent.spawn(meta_text(
                        fonts.text.clone(),
                        "",
                        FilePathText,
                        palette::DIM,
                        None,
                    ));
                    // Sample Rate
                    parent.spawn(meta_text(
                        fonts.text.clone(),
                        "",
                        SampleRateInfoText,
                        palette::DIM,
                        None,
                    ));
                });

            // Looping + ranges
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexEnd,
                    align_items: AlignItems::Center,
                    height: percent(100.),
                    column_gap: px(12),
                    flex_grow: 1.0,
                    ..Default::default()
                })
                .with_children(|parent| {
                    spawn_loop_toggle(parent, fonts);
                });
        });
}

pub fn spawn_waveform_view(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        // track waveform view section
        Node {
            flex_grow: 1.0,
            width: percent(100.),
            padding: UiRect::all(px(4.)),
            ..Default::default()
        },
        BackgroundColor(palette::BG_MED),
    ));
}

pub fn toggle_audio_playback(
    press: Res<ButtonInput<KeyCode>>,
    sink: Single<&AudioSink, With<PlayingAudio>>,
    fc: Single<&AudioFileContents>,
    mut commands: Commands,
) {
    if press.just_pressed(KeyCode::Space) {
        if sink.empty() || (sink.position() >= fc.duration) {
            sink.pause();
            commands.trigger(AudioReloadEvent);
            return;
        }
        sink.toggle_playback();
    }
}

pub fn update_timeline_scrubber(
    sink: Single<&AudioSink, With<PlayingAudio>>,
    fc: Single<&AudioFileContents>,
    mut timecode_text: Single<&mut Text, With<TimelineScrubber>>,
) {
    let pos = sink.position().as_secs_f64();
    let mut mins = (pos / 60.) as u64;
    let mut secs = (pos % 60.) as u64;
    if sink.empty() {
        (mins, secs) = (
            (fc.duration.as_secs_f64() / 60.0) as u64,
            (fc.duration.as_secs_f64() % 60.0) as u64,
        )
    }
    timecode_text.0 = format!("{mins:02}:{secs:02}");
}

pub fn handle_audio_end(
    mut commands: Commands,
    sink: Single<&AudioSink, With<PlayingAudio>>,
    usr_pb: Res<UserPlaybackMode>,
) {
    if sink.empty() {
        if *usr_pb == UserPlaybackMode::Once {
            sink.pause();
        } else {
            commands.trigger(AudioReloadEvent);
        }
    }
}
