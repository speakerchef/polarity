use bevy::prelude::*;

use crate::{
    palette,
    stereometer::{StereometerKind, StereometerParams},
    ui::interactions::*,
};

#[derive(Component, Clone)]
pub struct ModeSubmenu;

pub fn spawn_mode_submenu(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            ModeSubmenu,
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: percent(100.),
                border: UiRect::horizontal(px(1)).with_top(px(1)),
                padding: UiRect::all(px(palette::APP_PADDING)),
                ..Default::default()
            },
            BackgroundColor(palette::BG),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            (1..=4).for_each(|i| {
                parent
                    .spawn((
                        Node {
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            height: px(60.),
                            width: px(60.),
                            border: UiRect::all(px(palette::FRAME_WIDTH)),
                            ..Default::default()
                        },
                        BackgroundColor(palette::BG),
                        BorderColor::all(palette::BORDER),
                    ))
                    .with_children(|parent| {
                        parent.spawn((Text::new(i.to_string()),));
                    })
                    .observe(on_hover_bg)
                    .observe(on_leave_bg)
                    .observe(match i {
                        1 => |_: On<Pointer<Click>>, mut params: ResMut<StereometerParams>| {
                            params.kind = StereometerKind::LinearBipolar;
                        },
                        2 => |_: On<Pointer<Click>>, mut params: ResMut<StereometerParams>| {
                            params.kind = StereometerKind::ScaledBipolar;
                        },
                        3 => |_: On<Pointer<Click>>, mut params: ResMut<StereometerParams>| {
                            params.kind = StereometerKind::LinearLissajous;
                        },
                        4 => |_: On<Pointer<Click>>, mut params: ResMut<StereometerParams>| {
                            params.kind = StereometerKind::ScaledLissajous;
                        },
                        _ => unreachable!(),
                    });
            });
        });
}
