use bevy::{
    input_focus::InputFocus,
    prelude::*,
    text::EditableText,
    ui_widgets::{SetSliderValue, SliderDragState, SliderRange, SliderValue, SliderValueChange},
};

use crate::{
    FontBlock, palette,
    stereometer::StereometerParams,
    ui::{
        control_panel::{DropdownItem, SubmenuItem},
        menu_row_container, spawn_slider_row,
    },
};

#[derive(Component, Clone)]
pub struct ColorSubmenu;

#[derive(Component, Clone)]
pub struct HueSlider;
#[derive(Component, Clone)]
pub struct HueThumb;
#[derive(Component, Clone)]
pub struct HueAmt;

#[derive(Component, Clone)]
pub struct SaturationSlider;
#[derive(Component, Clone)]
pub struct SaturationThumb;
#[derive(Component, Clone)]
pub struct SaturationAmt;

#[derive(Component, Clone)]
pub struct LuminanceSlider;
#[derive(Component, Clone)]
pub struct LuminanceThumb;
#[derive(Component, Clone)]
pub struct LuminanceAmt;

pub fn spawn_color_submenu(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            SubmenuItem(DropdownItem::Color),
            ColorSubmenu,
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
                    spawn_slider_row(
                        parent,
                        fonts,
                        "Hue",
                        (0.0, 360.0, 0.1, 0.0, HueSlider, HueThumb),
                        (palette::width::SMALL_SELECTOR_MENU, 5, HueAmt),
                    );
                });
            parent
                .spawn(menu_row_container(JustifyContent::SpaceBetween))
                .with_children(|parent| {
                    spawn_slider_row(
                        parent,
                        fonts,
                        "Saturation",
                        (0.0, 1.0, 0.001, 1.0, SaturationSlider, SaturationThumb),
                        (palette::width::SMALL_SELECTOR_MENU, 4, SaturationAmt),
                    );
                });
            parent
                .spawn(menu_row_container(JustifyContent::SpaceBetween))
                .with_children(|parent| {
                    spawn_slider_row(
                        parent,
                        fonts,
                        "Luminance",
                        (0.0, 1.0, 0.001, 0.5, LuminanceSlider, LuminanceThumb),
                        (palette::width::SMALL_SELECTOR_MENU, 4, LuminanceAmt),
                    );
                });
        });
}

pub fn hue_text_update(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    button_input: Res<ButtonInput<KeyCode>>,
    sliders: Query<Entity, With<HueSlider>>,
    text: Query<&EditableText, With<HueAmt>>,
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
pub fn hue_slider_update(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange),
        (
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
            With<HueSlider>,
        ),
    >,
    mut thumbs: Query<(&mut Node, Has<HueThumb>), Without<HueSlider>>,
    children: Query<&Children>,
    mut amt_text: Single<&mut EditableText, With<HueAmt>>,
    mut params: ResMut<StereometerParams>,
) {
    for (slider_ent, value, range) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                amt_text.editor_mut().set_text(&format!("{:.1}", value.0));
                let position = range.thumb_position(value.0) * 100.0;
                thumb_node.left = percent(position);
                params.color.hue = value.0;
            }
        }
    }
}
pub fn saturation_text_update(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    button_input: Res<ButtonInput<KeyCode>>,
    sliders: Query<Entity, With<SaturationSlider>>,
    text: Query<&EditableText, With<SaturationAmt>>,
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
pub fn saturation_slider_update(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange),
        (
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
            With<SaturationSlider>,
        ),
    >,
    mut thumbs: Query<(&mut Node, Has<SaturationThumb>), Without<SaturationSlider>>,
    children: Query<&Children>,
    mut amt_text: Single<&mut EditableText, With<SaturationAmt>>,
    mut params: ResMut<StereometerParams>,
) {
    for (slider_ent, value, range) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                amt_text.editor_mut().set_text(&format!("{:.2}", value.0));
                let position = range.thumb_position(value.0) * 100.0;
                thumb_node.left = percent(position);
                params.color.saturation = value.0;
            }
        }
    }
}

pub fn luminance_text_update(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    button_input: Res<ButtonInput<KeyCode>>,
    sliders: Query<Entity, With<LuminanceSlider>>,
    text: Query<&EditableText, With<LuminanceAmt>>,
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
pub fn luminance_slider_update(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange),
        (
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
            With<LuminanceSlider>,
        ),
    >,
    mut thumbs: Query<(&mut Node, Has<LuminanceThumb>), Without<LuminanceSlider>>,
    children: Query<&Children>,
    mut amt_text: Single<&mut EditableText, With<LuminanceAmt>>,
    mut params: ResMut<StereometerParams>,
) {
    for (slider_ent, value, range) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                amt_text.editor_mut().set_text(&format!("{:.2}", value.0));
                let position = range.thumb_position(value.0) * 100.0;
                thumb_node.left = percent(position);
                params.color.lightness = value.0;
            }
        }
    }
}
