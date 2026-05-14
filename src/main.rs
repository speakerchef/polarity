use bevy::prelude::*;
mod palette;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Polarity".into(),
                mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
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
                                    left: px(palette::Spacing::S1),
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
                                    // Import button
                                    parent
                                        .spawn((
                                            Node {
                                                display: Display::Flex,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                height: percent(100.),
                                                width: percent(50.),
                                                border: UiRect {
                                                    right: px(palette::FRAME_WIDTH),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            BackgroundColor(palette::BG_MED),
                                            BorderColor::all(palette::BORDER),
                                        ))
                                        .with_child((
                                            Text("IMPORT".to_string()),
                                            TextColor(palette::TEXT),
                                            TextFont {
                                                ..Default::default()
                                            },
                                        ));
                                    // Export button
                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                display: Display::Flex,
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                height: percent(100.),
                                                width: percent(50.),
                                                ..Default::default()
                                            },
                                            BackgroundColor(palette::BG_MED),
                                        ))
                                        .with_child((
                                            Text("EXPORT".to_string()),
                                            TextColor(palette::TEXT),
                                            TextFont {
                                                ..Default::default()
                                            },
                                        ));
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
