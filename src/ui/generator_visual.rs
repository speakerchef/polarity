use crate::LiveDensity;
use crate::palette;
use crate::stereometer::StereometerParams;
use crate::ui::interactions::*;
use crate::ui::spawn_body_text;
use crate::{FontBlock, HistoryDensity};
use bevy::ecs::observer;
use bevy::ecs::relationship::RelationshipSourceCollection;
use bevy::input;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus;
use bevy::input_focus::AutoFocus;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::text::TextCursorStyle;

#[derive(Component, Clone)]
pub struct RedColorSelector;

#[derive(Component, Clone)]
pub struct BlueColorSelector;

#[derive(Component, Clone)]
pub struct GreenColorSelector;

#[derive(Component, Clone)]
pub struct RedSelectorMarker;

#[derive(Component, Clone)]
pub struct GreenSelectorMarker;

#[derive(Component, Clone)]
pub struct BlueSelectorMarker;

#[derive(Component, Clone)]
pub struct VisualSubmenu;

#[derive(Component, Clone)]
pub struct VisualDensityMarker;

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
pub struct VisualPhosphorMarker;

#[derive(Component, Clone)]
pub struct VisualPhosphorText;

#[derive(Component, Clone)]
pub struct VisualColorMarker;

#[derive(Component, Clone)]
pub struct VisualColorText;

#[derive(Component, Clone)]
pub struct VisualColorSelectorMenu;

#[derive(Component, Clone)]
pub struct VisualColorPicker;

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

pub fn spawn_color_input_picker<'a, T: Component>(
    parent: &'a mut ChildSpawnerCommands,
    sz: f32,
    marker: T,
    font: &FontBlock,
) -> bevy::prelude::EntityCommands<'a> {
    let mut parent_spawner = parent.spawn((
        Node {
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            height: px(palette::height::INNER),
            border: UiRect::all(px(1)),
            width: px(sz),
            ..Default::default()
        },
        AutoFocus,
        BorderColor::all(palette::BORDER),
        BackgroundColor(palette::VOID),
    ));
    parent_spawner
        .with_children(|parent| {
            parent.spawn((
                ColorPicker,
                marker,
                EditableText {
                    visible_width: Some(3.),
                    max_characters: Some(3),
                    allow_newlines: false,

                    ..Default::default()
                },
                TextLayout::no_wrap(),
                TextFont {
                    font: font.text.clone(),
                    font_size: FontSize::Px(palette::font_size::BIG),
                    weight: FontWeight(palette::font_weight::BODY),
                    ..default()
                },
                TextCursorStyle::default(),
            ));
        })
        .observe(on_hover_void)
        .observe(on_leave_void);
    parent_spawner
}

#[derive(Component)]
pub struct ColorPicker;

pub fn watch_color_input_edit(
    mut input_focus: ResMut<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    color_picker: Query<&ColorPicker>,
    mut paramset: ParamSet<(
        Single<&mut EditableText, With<RedColorSelector>>,
        Single<&mut EditableText, With<GreenColorSelector>>,
        Single<&mut EditableText, With<BlueColorSelector>>,
    )>,
    mut params: ResMut<StereometerParams>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter)
        && let Some(focused_entity) = input_focus.get()
    {
        input_focus.clear();
        if color_picker.get(focused_entity).is_ok() {
            if let Ok(mut value) = paramset.p0().value().to_string().parse::<i128>() {
                value = value.clamp(0, 255);
                let red = value as f32 / u8::MAX as f32;
                params.color.red = red;
                paramset.p0().editor_mut().set_text(&value.to_string());
                info!("Entered red: {}", paramset.p0().value());
            }
            if let Ok(mut value) = paramset.p1().value().to_string().parse::<i128>() {
                value = value.clamp(0, 255);
                let green = value as f32 / u8::MAX as f32;
                params.color.green = green;
                paramset.p1().editor_mut().set_text(&value.to_string());
                info!("Entered green: {}", paramset.p1().value());
            }
            if let Ok(mut value) = paramset.p2().value().to_string().parse::<i128>() {
                value = value.clamp(0, 255);
                let blue = value as f32 / u8::MAX as f32;
                params.color.blue = blue;
                paramset.p2().editor_mut().set_text(&value.to_string());
                info!("Entered blue: {}", paramset.p2().value());
            }
        }
    }
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

fn spawn_history_density_options(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    for level in HistoryDensity::all() {
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
                spawn_body_text(
                    parent,
                    &Into::<String>::into(level.clone()),
                    VisualPhosphorText,
                    fonts,
                );
            })
            .observe(
                |_: On<Pointer<Click>>,
                 mut stereo_params: ResMut<StereometerParams>,
                 mut txt: Single<&mut Text, With<VisualPhosphorSelectorMenu>>| {
                    info!("clicked");
                    stereo_params.history_density = level.clone();
                    txt.0 = Into::<String>::into(level.clone());
                },
            )
            .observe(on_hover_void)
            .observe(on_leave_void);
    }
}

fn spawn_point_density_options(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    for level in LiveDensity::all() {
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
                spawn_body_text(
                    parent,
                    &Into::<String>::into(level.clone()),
                    VisualDensityText,
                    fonts,
                );
            })
            .observe(
                |_: On<Pointer<Click>>,
                 mut stereo_params: ResMut<StereometerParams>,
                 mut txt: Single<&mut Text, With<VisualDensitySelectorMenu>>| {
                    info!("clicked");
                    stereo_params.live_density = level.clone();
                    txt.0 = Into::<String>::into(level.clone());
                },
            )
            .observe(on_hover_void)
            .observe(on_leave_void);
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
            (1..=3).for_each(|i| {
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
                            spawn_body_text(parent, "Density", VisualDensityMarker, fonts);
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
                                        spawn_point_density_options(parent, fonts);
                                    });
                            })
                            .observe(visual_density_on_click);
                        }
                        2 => {
                            spawn_body_text(parent, "Phosphor", VisualPhosphorMarker, fonts);
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
                                        spawn_history_density_options(parent, fonts);
                                    });
                            })
                            .observe(visual_phosphor_on_click);
                        }
                        3 => {
                            spawn_body_text(parent, "Color", VisualColorMarker, fonts);
                            parent
                                .spawn((
                                    Node {
                                        display: Display::Flex,
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::SpaceBetween,
                                        column_gap: px(8),
                                        ..Default::default()
                                    },
                                    BackgroundColor(palette::BG),
                                ))
                                .with_children(|parent| {
                                    parent
                                        .spawn((Node {
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::SpaceBetween,
                                            column_gap: px(2),
                                            ..Default::default()
                                        },))
                                        .with_children(|parent| {
                                            spawn_body_text(parent, "R:", RedSelectorMarker, fonts);
                                            spawn_color_input_picker(
                                                parent,
                                                palette::width::SMALL_SELECTOR_MENU,
                                                RedColorSelector,
                                                fonts,
                                            );
                                        });
                                    parent
                                        .spawn((Node {
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::SpaceBetween,
                                            column_gap: px(4),
                                            ..Default::default()
                                        },))
                                        .with_children(|parent| {
                                            spawn_body_text(
                                                parent,
                                                "G:",
                                                GreenSelectorMarker,
                                                fonts,
                                            );
                                            spawn_color_input_picker(
                                                parent,
                                                palette::width::SMALL_SELECTOR_MENU,
                                                GreenColorSelector,
                                                fonts,
                                            );
                                        });
                                    parent
                                        .spawn((Node {
                                            display: Display::Flex,
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::SpaceBetween,
                                            column_gap: px(4),
                                            ..Default::default()
                                        },))
                                        .with_children(|parent| {
                                            spawn_body_text(
                                                parent,
                                                "B:",
                                                BlueSelectorMarker,
                                                fonts,
                                            );
                                            spawn_color_input_picker(
                                                parent,
                                                palette::width::SMALL_SELECTOR_MENU,
                                                BlueColorSelector,
                                                fonts,
                                            );
                                        });
                                });
                        }
                        _ => unreachable!(),
                    });
            })
        });
}
