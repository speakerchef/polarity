use crate::LiveDensity;
use crate::palette;
use crate::stereometer::StereometerParams;
use crate::ui::interactions::*;
use crate::ui::spawn_body_text;
use crate::{FontBlock, HistoryDensity};
use bevy::prelude::*;
#[derive(Component, Clone)]
pub struct VisualSubmenu;

#[derive(Component, Clone)]
pub struct VisualDensityText;

#[derive(Component, Clone)]
pub struct VisualDensitySelectorMenu;

#[derive(Component, Clone)]
pub struct VisualDensityDropdown;

#[derive(Component, Clone)]
pub struct VisualPhosphorSelectorMenu;

#[derive(Component, Clone)]
pub struct VisualPhosphorDropdown;

#[derive(Component, Clone)]
pub struct VisualDensitySelectorText;

#[derive(Component, Clone)]
pub struct VisualPhosphorText;

pub fn spawn_selector_with_size<'a>(
    parent: &'a mut ChildSpawnerCommands,
    sz: f32,
    text: &str,
    marker: impl Component + Clone,
    font: &FontBlock,
) -> bevy::prelude::EntityCommands<'a> {
    let mut parent_spawner = parent.spawn((Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        width: px(sz),
        ..Default::default()
    },));
    parent_spawner.with_children(|parent| {
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
            .with_children(|parent| spawn_body_text(parent, text, marker, font))
            .observe(on_hover_void)
            .observe(on_leave_void);
    });
    parent_spawner
}

pub fn visual_density_on_click(
    _: On<Pointer<Click>>,
    mut vis_phosphor: Single<
        &mut Node,
        (With<VisualPhosphorDropdown>, Without<VisualDensityDropdown>),
    >,
    mut vis_density: Single<
        &mut Node,
        (With<VisualDensityDropdown>, Without<VisualPhosphorDropdown>),
    >,
) {
    if vis_density.display != Display::Flex {
        vis_density.display = Display::Flex;
        if vis_phosphor.display == Display::Flex {
            vis_phosphor.display = Display::None;
        }
    } else {
        vis_density.display = Display::None;
    }
}

pub fn visual_phosphor_on_click(
    _: On<Pointer<Click>>,
    mut vis_phosphor: Single<
        &mut Node,
        (With<VisualPhosphorDropdown>, Without<VisualDensityDropdown>),
    >,
    mut vis_density: Single<
        &mut Node,
        (With<VisualDensityDropdown>, Without<VisualPhosphorDropdown>),
    >,
) {
    if vis_phosphor.display != Display::Flex {
        vis_phosphor.display = Display::Flex;
        if vis_density.display == Display::Flex {
            vis_density.display = Display::None;
        }
    } else {
        vis_phosphor.display = Display::None;
    }
}

pub fn spawn_visual_submenu(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            VisualSubmenu,
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
            (1..=2).for_each(|i| {
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
                    .with_children(|parent| match i {
                        1 => {
                            spawn_body_text(parent, "Density", VisualDensityText, fonts);
                            spawn_selector_with_size(
                                parent,
                                palette::width::MED_SELECTOR_MENU,
                                &Into::<String>::into(LiveDensity::default()),
                                VisualDensitySelectorMenu,
                                fonts,
                            )
                            .with_children(|parent| {
                                parent
                                    .spawn((
                                        VisualDensityDropdown,
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
                                        for level in LiveDensity::all() {
                                            parent
                                                .spawn((
                                                    Node {
                                                        display: Display::Flex,
                                                        align_items: AlignItems::Center,
                                                        justify_content: JustifyContent::Center,
                                                        height: px(
                                                            palette::height::SLIDER_ROW_ITEM,
                                                        ),
                                                        width: percent(100.),
                                                        border: UiRect::all(px(1)),
                                                        ..Default::default()
                                                    },
                                                    BorderColor::all(palette::BORDER),
                                                    BackgroundColor(palette::VOID),
                                                ))
                                                .with_children(|parent| {
                                                    spawn_body_text(
                                                        parent,
                                                        &Into::<String>::into(level.clone()),
                                                        VisualDensityText,
                                                        fonts,
                                                    );
                                                })
                                                .observe(|_: On<Pointer<Click>>, mut stereo_params: ResMut<StereometerParams>, mut txt: Single<&mut Text, With<VisualDensitySelectorMenu>>| {
                                                        info!("clicked");
                                                        stereo_params.live_density = level.clone();
                                                        txt.0 = Into::<String>::into(level.clone());
                                                })
                                                .observe(on_hover_void)
                                                .observe(on_leave_void);
                                        }
                                    });
                            })
                            .observe(visual_density_on_click);
                        }
                        2 => {
                            spawn_body_text(parent, "Phosphor", VisualPhosphorText, fonts);
                            spawn_selector_with_size(
                                parent,
                                palette::width::MED_SELECTOR_MENU,
                                &Into::<String>::into(HistoryDensity::default()),
                                VisualPhosphorSelectorMenu,
                                fonts,
                            )
                            .with_children(|parent| {
                                parent
                                    .spawn((
                                        VisualPhosphorDropdown,
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
                                        for level in HistoryDensity::all() {
                                            parent
                                                .spawn((
                                                    Node {
                                                        display: Display::Flex,
                                                        align_items: AlignItems::Center,
                                                        justify_content: JustifyContent::Center,
                                                        height: px(
                                                            palette::height::SLIDER_ROW_ITEM,
                                                        ),
                                                        width: percent(100.),
                                                        border: UiRect::all(px(1)),
                                                        ..Default::default()
                                                    },
                                                    BorderColor::all(palette::BORDER),
                                                    BackgroundColor(palette::VOID),
                                                ))
                                                .with_children(|parent| {
                                                    spawn_body_text(
                                                        parent,
                                                        &Into::<String>::into(level.clone()),
                                                        VisualPhosphorText,
                                                        fonts,
                                                    );
                                                })
                                                .observe(|_: On<Pointer<Click>>, mut stereo_params: ResMut<StereometerParams>, mut txt: Single<&mut Text, With<VisualPhosphorSelectorMenu>>| {
                                                        info!("clicked");
                                                        stereo_params.history_density = level.clone();
                                                        txt.0 = Into::<String>::into(level.clone());
                                                })

                                                .observe(on_hover_void)
                                                .observe(on_leave_void);
                                        }
                                    });
                            })
                            .observe(visual_phosphor_on_click);
                        }
                        _ => unreachable!(),
                    });
            })
        });
}
