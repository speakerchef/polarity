use bevy::prelude::*;

use crate::{
    FontBlock, NullComponent, palette,
    stereometer::{StereometerInputMode, StereometerParams},
    ui::{interactions::*, spawn_body_text, spawn_selector_with_size},
};

#[derive(Component, Clone)]
pub struct InputSubmenu;
#[derive(Component, Clone)]
pub struct InputModeSelectorMenu;
#[derive(Component, Clone)]
pub struct InputModeDropdown;

pub fn spawn_input_submenu(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            InputSubmenu,
            Node {
                display: Display::None,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                width: percent(100.),
                ..Default::default()
            },
            BorderColor::all(palette::BORDER),
            BackgroundColor(palette::BG),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        height: px(palette::height::MENU_ITEM),
                        width: percent(100.),
                        padding: UiRect::horizontal(px(12)),
                        border: UiRect::horizontal(px(1)).with_bottom(px(1)),
                        ..Default::default()
                    },
                    BorderColor::all(palette::BORDER),
                ))
                .with_children(|parent| {
                    spawn_body_text(parent, "Mode", NullComponent, fonts);
                    spawn_selector_with_size(
                        parent,
                        palette::width::MED_SELECTOR_MENU,
                        "Full Spectrum",
                        InputModeSelectorMenu,
                        fonts,
                    )
                    .with_children(|parent| {
                        parent
                            .spawn((
                                InputModeDropdown,
                                Node {
                                    display: Display::None,
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    position_type: PositionType::Absolute,
                                    top: percent(100.),
                                    justify_content: JustifyContent::FlexStart,
                                    width: percent(100.),
                                    ..Default::default()
                                },
                                GlobalZIndex(1),
                            ))
                            .with_children(|parent| {
                                spawn_input_mode_options(parent, fonts);
                            });
                    })
                    .observe(input_mode_on_click);
                });
        });
}
fn input_mode_on_click(_: On<Pointer<Click>>, mut vis: Single<&mut Node, With<InputModeDropdown>>) {
    if vis.display != Display::Flex {
        vis.display = Display::Flex;
    } else {
        vis.display = Display::None;
    }
}

fn spawn_input_mode_options(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    for mode in &[
        StereometerInputMode::FullSpectrum,
        StereometerInputMode::MultiBand,
    ] {
        parent
            .spawn((
                Node {
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    height: px(palette::height::SLIDER_ROW_ITEM),
                    width: percent(100.),
                    border: UiRect::all(px(1)),
                    ..Default::default()
                },
                BorderColor::all(palette::BORDER),
                BackgroundColor(palette::VOID),
            ))
            .with_children(|parent| {
                spawn_body_text(parent, &mode.to_string(), NullComponent, fonts);
            })
            .observe(
                |_: On<Pointer<Click>>,
                 mut stereo_params: ResMut<StereometerParams>,
                 mut txt: Single<&mut Text, With<InputModeSelectorMenu>>| {
                    info!("clicked");
                    stereo_params.input_mode = mode.clone();
                    txt.0 = mode.to_string();
                },
            )
            .observe(on_hover_void)
            .observe(on_leave_void);
    }
}
