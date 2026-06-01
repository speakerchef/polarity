use crate::LiveDensity;
use crate::palette;
use crate::stereometer::StereometerParams;
use crate::ui::control_panel::DropdownItem;
use crate::ui::control_panel::SubmenuItem;
use crate::ui::interactions::*;
use crate::ui::menu_row_container;
use crate::ui::spawn_body_text;
use crate::ui::spawn_dropdown_row;
use crate::ui::spawn_slider_row;
use crate::{FontBlock, TraceDensity};
use bevy::input_focus::AutoFocus;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::text::TextCursorStyle;
use bevy::ui_widgets::SetSliderValue;
use bevy::ui_widgets::SliderDragState;
use bevy::ui_widgets::SliderRange;
use bevy::ui_widgets::SliderValue;
use bevy::ui_widgets::SliderValueChange;

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
pub struct VisualTraceSelectorMenu;

#[derive(Component, Clone)]
pub struct VisualTraceDropdown;

#[derive(Component, Clone)]
pub struct VisualDensitySelectorText;

#[derive(Component, Clone)]
pub struct VisualTraceMarker;

#[derive(Component, Clone)]
pub struct VisualTraceText;

#[derive(Component, Clone)]
pub struct VisualColorMarker;

#[derive(Component, Clone)]
pub struct VisualColorText;

#[derive(Component, Clone)]
pub struct VisualColorSelectorMenu;

#[derive(Component, Clone)]
pub struct VisualColorPicker;

pub fn spawn_color_input_picker<'a, T: Component>(
    parent: &'a mut ChildSpawnerCommands,
    default_text: &str,
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
                Text::new(default_text),
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

fn set_color<T: Component>(mut p: Single<&mut EditableText, With<T>>, mut v: i128, c: &mut f32) {
    v = v.clamp(0, 255);
    let col = v as f32 / u8::MAX as f32;
    *c = col;
    p.editor_mut().set_text(&v.to_string());
}

#[allow(clippy::type_complexity)]
pub fn watch_color_input_edit(
    mut input_focus: ResMut<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    color_picker: Query<&ColorPicker>,
    mut paramset: ParamSet<(
        Single<&mut EditableText, With<RedColorSelector>>,
        Single<&mut EditableText, With<GreenColorSelector>>,
        Single<&mut EditableText, With<BlueColorSelector>>,
    )>,
    ps_color_textbox: ParamSet<(
        Single<&mut Text, With<RedColorSelector>>,
        Single<&mut Text, With<GreenColorSelector>>,
        Single<&mut Text, With<BlueColorSelector>>,
    )>,
    mut params: ResMut<StereometerParams>,
) {
    if (keyboard_input.just_pressed(KeyCode::Enter)
        || keyboard_input.just_pressed(KeyCode::NumpadEnter))
        && let Some(focused_entity) = input_focus.get()
    {
        input_focus.clear();
        if color_picker.get(focused_entity).is_ok() {
            if let Ok(value) = paramset.p0().value().to_string().parse::<i128>() {
                set_color(paramset.p0(), value, &mut params.color.red);
            }
            if let Ok(value) = paramset.p1().value().to_string().parse::<i128>() {
                set_color(paramset.p1(), value, &mut params.color.green);
            }
            if let Ok(value) = paramset.p2().value().to_string().parse::<i128>() {
                set_color(paramset.p2(), value, &mut params.color.blue);
            }
            update_color_picker_textbox(ps_color_textbox, &mut params.color);
        }
    }
}

pub fn visual_density_on_click(
    _: On<Pointer<Click>>,
    mut vis_trace: Single<&mut Node, (With<VisualTraceDropdown>, Without<VisualDensityDropdown>)>,
    mut vis_density: Single<&mut Node, (With<VisualDensityDropdown>, Without<VisualTraceDropdown>)>,
) {
    if vis_density.display != Display::Flex {
        vis_density.display = Display::Flex;
        if vis_trace.display == Display::Flex {
            vis_trace.display = Display::None;
        }
    } else {
        vis_density.display = Display::None;
    }
}

pub fn visual_trace_on_click(
    _: On<Pointer<Click>>,
    mut vis_trace: Single<&mut Node, (With<VisualTraceDropdown>, Without<VisualDensityDropdown>)>,
    mut vis_density: Single<&mut Node, (With<VisualDensityDropdown>, Without<VisualTraceDropdown>)>,
) {
    if vis_trace.display != Display::Flex {
        vis_trace.display = Display::Flex;
        if vis_density.display == Display::Flex {
            vis_density.display = Display::None;
        }
    } else {
        vis_trace.display = Display::None;
    }
}

fn spawn_trace_density_options(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    for level in TraceDensity::all() {
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
                    VisualTraceText,
                    fonts,
                );
            })
            .observe(
                |_: On<Pointer<Click>>,
                 mut stereo_params: ResMut<StereometerParams>,
                 mut txt: Single<&mut Text, With<VisualTraceSelectorMenu>>| {
                    info!("clicked");
                    stereo_params.trace_density = level.clone();
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
#[derive(Component, Clone)]
pub struct VisualScaleSlider;
#[derive(Component, Clone)]
pub struct VisualScaleThumb;
#[derive(Component, Clone)]
pub struct VisualScaleAmt;

#[derive(Component, Clone)]
pub struct VisualDotSizeSlider;
#[derive(Component, Clone)]
pub struct VisualDotSizeThumb;
#[derive(Component, Clone)]
pub struct VisualDotSizeAmt;

pub fn spawn_visual_submenu(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            SubmenuItem(DropdownItem::Visual),
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
            parent
                .spawn(menu_row_container(JustifyContent::SpaceBetween))
                .with_children(|parent| {
                    spawn_dropdown_row(
                        parent,
                        fonts,
                        "Density",
                        (
                            &Into::<String>::into(LiveDensity::default()),
                            VisualDensitySelectorMenu,
                        ),
                        VisualDensityDropdown,
                        spawn_point_density_options,
                    )
                    .observe(visual_density_on_click);
                });
            parent
                .spawn(menu_row_container(JustifyContent::SpaceBetween))
                .with_children(|parent| {
                    spawn_dropdown_row(
                        parent,
                        fonts,
                        "Trace",
                        (
                            &Into::<String>::into(TraceDensity::default()),
                            VisualTraceSelectorMenu,
                        ),
                        VisualTraceDropdown,
                        spawn_trace_density_options,
                    )
                    .observe(visual_trace_on_click);
                });
            parent
                .spawn(menu_row_container(JustifyContent::Default))
                .with_children(|parent| {
                    spawn_slider_row(
                        parent,
                        fonts,
                        "Scale(%)",
                        (
                            100.0,
                            500.0,
                            1.0,
                            250.0,
                            VisualScaleSlider,
                            VisualScaleThumb,
                        ),
                        (palette::width::SMALL_SELECTOR_MENU, 4, VisualScaleAmt),
                    )
                });
            parent
                .spawn(menu_row_container(JustifyContent::Default))
                .with_children(|parent| {
                    spawn_slider_row(
                        parent,
                        fonts,
                        "Pixel(Px)",
                        (
                            0.20,
                            2.00,
                            0.01,
                            0.75,
                            VisualDotSizeSlider,
                            VisualDotSizeThumb,
                        ),
                        (palette::width::SMALL_SELECTOR_MENU, 4, VisualDotSizeAmt),
                    )
                });
            parent
                .spawn(menu_row_container(JustifyContent::SpaceBetween))
                .with_children(|parent| {
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
                                        "255",
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
                                    spawn_body_text(parent, "G:", GreenSelectorMarker, fonts);
                                    spawn_color_input_picker(
                                        parent,
                                        "255",
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
                                    spawn_body_text(parent, "B:", BlueSelectorMarker, fonts);
                                    spawn_color_input_picker(
                                        parent,
                                        "255",
                                        palette::width::SMALL_SELECTOR_MENU,
                                        BlueColorSelector,
                                        fonts,
                                    );
                                });
                        });
                });
        });
}

pub fn scale_amt_text_update(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    button_input: Res<ButtonInput<KeyCode>>,
    sliders: Query<Entity, With<VisualScaleSlider>>,
    text: Query<&EditableText, With<VisualScaleAmt>>,
) {
    if (button_input.just_pressed(KeyCode::Enter)
        || button_input.just_pressed(KeyCode::NumpadEnter))
        && let Some(focused_ent) = input_focus.get()
        && let Ok(text_entity) = text.get(focused_ent)
        && let Ok(text_to_f32) = text_entity.value().to_string().parse::<f32>()
    {
        for e in sliders {
            commands.trigger(SetSliderValue {
                entity: e,
                change: SliderValueChange::Absolute(text_to_f32),
            })
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn scale_slider_update(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange),
        (
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
            With<VisualScaleSlider>,
        ),
    >,
    mut thumbs: Query<(&mut Node, Has<VisualScaleThumb>), Without<VisualScaleSlider>>,
    children: Query<&Children>,
    mut amt_text: Single<&mut EditableText, With<VisualScaleAmt>>,
    mut params: ResMut<StereometerParams>,
) {
    for (slider_ent, value, range) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                amt_text.editor_mut().set_text(&value.0.round().to_string());
                params.scale_factor = value.0.round();
                let position = range.thumb_position(value.0) * 100.0;
                thumb_node.left = percent(position);
            }
        }
    }
}

pub fn dot_size_amt_text_update(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    button_input: Res<ButtonInput<KeyCode>>,
    sliders: Query<Entity, With<VisualDotSizeSlider>>,
    text: Query<&EditableText, With<VisualDotSizeAmt>>,
) {
    if (button_input.just_pressed(KeyCode::Enter)
        || button_input.just_pressed(KeyCode::NumpadEnter))
        && let Some(focused_ent) = input_focus.get()
        && let Ok(text_entity) = text.get(focused_ent)
        && let Ok(text_to_f32) = text_entity.value().to_string().parse::<f32>()
    {
        for e in sliders {
            commands.trigger(SetSliderValue {
                entity: e,
                change: SliderValueChange::Absolute(text_to_f32),
            })
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn dot_size_slider_update(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange),
        (
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
            With<VisualDotSizeSlider>,
        ),
    >,
    mut thumbs: Query<(&mut Node, Has<VisualDotSizeThumb>), Without<VisualDotSizeSlider>>,
    children: Query<&Children>,
    mut amt_text: Single<&mut EditableText, With<VisualDotSizeAmt>>,
    mut params: ResMut<StereometerParams>,
) {
    for (slider_ent, value, range) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                amt_text.editor_mut().set_text(&format!("{:.2}", value.0));
                params.dot_size = value.0;
                let position = range.thumb_position(value.0) * 100.0;
                thumb_node.left = percent(position);
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_color_picker_textbox(
    mut ps: ParamSet<(
        Single<&mut Text, With<RedColorSelector>>,
        Single<&mut Text, With<GreenColorSelector>>,
        Single<&mut Text, With<BlueColorSelector>>,
    )>,
    color: &mut LinearRgba,
) {
    ps.p0().0 = ((color.red * u8::MAX as f32) as u8).to_string();
    ps.p1().0 = ((color.green * u8::MAX as f32) as u8).to_string();
    ps.p2().0 = ((color.blue * u8::MAX as f32) as u8).to_string();
}
