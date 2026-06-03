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
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::EditableText;
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
                .spawn(menu_row_container(JustifyContent::SpaceBetween, None))
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
                .spawn(menu_row_container(JustifyContent::SpaceBetween, None))
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
                .spawn(menu_row_container(JustifyContent::Default, None))
                .with_children(|parent| {
                    spawn_slider_row(
                        parent,
                        fonts,
                        "Scale %",
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
                .spawn(menu_row_container(JustifyContent::Default, None))
                .with_children(|parent| {
                    spawn_slider_row(
                        parent,
                        fonts,
                        "Pixel Size",
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
