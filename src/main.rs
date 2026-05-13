use bevy::{color::palettes, prelude::*};
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
    commands
        .spawn((
            Node {
                display: Display::Flex,
                height: percent(100.),
                width: percent(100.),
                border: UiRect::all(px(1)),
                ..Default::default()
            },
            BackgroundColor(palette::BG),
            BorderColor::all(palette::BORDER_STRONG),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0.),
                    height: percent(100.),
                    width: percent(25.),
                    ..Default::default()
                },
                BackgroundColor(palette::PANEL),
            ));
            parent
                .spawn((
                    Node {
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
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: px(0.9),
                            height: percent(77.),
                            width: percent(100.),
                            ..Default::default()
                        },
                        BackgroundColor(palette::PANEL),
                    ));
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(0.),
                            height: percent(21.),
                            width: percent(100.),
                            margin: UiRect {
                                top: px(1),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        BackgroundColor(palette::PANEL),
                    ));
                });
        });
}
