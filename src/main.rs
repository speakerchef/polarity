use bevy::asset::UnapprovedPathMode;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;
use bevy_file_dialog::prelude::*;
use polarity::stereometer::{self, StereometerParams};
use polarity::ui::control_panel::{file_loaded, spawn_control_panel};
use polarity::ui::generator_filtering::{
    freq_amt_text_update, freq_slider_update, update_filter_freq,
};
use polarity::ui::generator_visual::watch_color_input_edit;
use polarity::ui::postfx_sparkle::{sparkle_slider_update, sparkle_text_update};
use polarity::{
    AudioFileContents, CustomMaterial, DurationText, FontBlock, PlayingAudio, PreviewCanvas,
    TimelineScrubber, palette,
};

#[derive(States, Component, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum GeneratorChoice {
    #[default]
    Stereometer,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(palette::VOID))
        .insert_resource(StereometerParams {
            scale_factor: 250.0,
            ..Default::default()
        })
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
        .add_systems(
            OnEnter(GeneratorChoice::Stereometer),
            stereometer::spawn_stereometer,
        )
        .add_systems(
            OnExit(GeneratorChoice::Stereometer),
            stereometer::despawn_stereometer,
        )
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
        .add_systems(Update, (watch_color_input_edit,))
        .add_systems(Update, (sparkle_slider_update, sparkle_text_update))
        .add_systems(
            Update,
            (freq_slider_update, freq_amt_text_update, update_filter_freq),
        )
        .run();
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
                height: px(104.),
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
            min_height: vw(30.),
            flex_grow: 1.,
            min_width: px(0),
            aspect_ratio: Some(16. / 9.),
            ..Default::default()
        },
    ));
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

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let fonts = FontBlock {
        icon: FontSource::Handle(
            asset_server.load("fonts/material-symbols/MaterialSymbolsOutlined.ttf"),
        ),
        text: FontSource::Handle(asset_server.load("fonts/inter/InterVariable.ttf")),
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
            // main app container
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
                            spawn_preview_canvas(parent);
                            spawn_control_panel(parent, &fonts);
                        });
                    // Bottom section holder for timeline
                    parent
                        .spawn((Node {
                            display: Display::Flex,
                            height: px(104.),
                            flex_shrink: 0.,
                            width: percent(100.),
                            ..Default::default()
                        },))
                        .with_children(spawn_timeline);
                });
        });
}
