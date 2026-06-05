use bevy::asset::UnapprovedPathMode;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;
use bevy_file_dialog::prelude::*;
use polarity::stereometer::{self, StereometerParams};
use polarity::ui::control_panel::{file_loaded, loop_audio, spawn_control_panel};
use polarity::ui::generator_color::{
    HighHsla, HslaSliderUpdater, LowHsla, MainHsla, MidHsla, set_color_display_with_render_mode,
};
use polarity::ui::generator_filtering::{freq_amt_text_update, freq_slider_update};
use polarity::ui::generator_render::watch_render_mode;
use polarity::ui::generator_visual::{
    dot_size_amt_text_update, dot_size_slider_update, scale_amt_text_update, scale_slider_update,
};
use polarity::ui::postfx_sparkle::{sparkle_slider_update, sparkle_text_update};
use polarity::ui::timeline::{
    handle_audio_end, spawn_timeline, toggle_audio_playback, update_playback_indicator,
    update_timeline_scrubber,
};
use polarity::ui::{on_scroll_handler, send_scroll_events};
use polarity::{
    AudioFileContents, CustomMaterial, FontBlock, PreviewCanvas, UserPlaybackMode, palette,
};

#[derive(States, Component, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum GeneratorChoice {
    #[default]
    Stereometer,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(palette::VOID))
        .insert_resource(UserPlaybackMode::default())
        .insert_resource(StereometerParams {
            color: Hsla::new(0.0, 1.0, 0.5, 1.0),
            multiband_color: (
                LinearRgba::RED.into(),
                LinearRgba::GREEN.into(),
                LinearRgba::BLUE.into(),
            ),
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
        .add_systems(Update, (update_playback_indicator,))
        .add_systems(Update, toggle_audio_playback)
        .add_systems(Update, (file_loaded, update_timeline_scrubber))
        .add_systems(
            Update,
            (stereometer::update, stereometer::draw)
                .chain()
                .run_if(in_state(GeneratorChoice::Stereometer)),
        )
        .add_systems(Update, watch_render_mode)
        .add_systems(Update, (scale_slider_update, scale_amt_text_update))
        .add_systems(Update, (dot_size_slider_update, dot_size_amt_text_update))
        .add_systems(Update, (MainHsla::slider_update, MainHsla::text_update))
        .add_systems(Update, (LowHsla::slider_update, LowHsla::text_update))
        .add_systems(Update, (MidHsla::slider_update, MidHsla::text_update))
        .add_systems(Update, (HighHsla::slider_update, HighHsla::text_update))
        .add_systems(Update, set_color_display_with_render_mode)
        .add_systems(Update, (sparkle_text_update, sparkle_slider_update))
        .add_systems(Update, (freq_amt_text_update, freq_slider_update))
        .add_systems(Update, send_scroll_events)
        .add_systems(Update, handle_audio_end)
        .add_observer(loop_audio)
        .add_observer(on_scroll_handler)
        .run();
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
                            flex_grow: 1.0,
                            min_height: px(0),
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
                        .with_children(|parent| spawn_timeline(parent, &fonts));
                });
        });
}
