use crate::NullComponent;
use crate::stereometer::StereometerParams;
use crate::ui::{horizontal_slider, spawn_body_text, spawn_textbox};
use bevy::input_focus::InputFocus;
use bevy::ui_widgets::SetSliderValue;
use bevy::ui_widgets::SliderDragState;
use bevy::ui_widgets::SliderRange;
use bevy::ui_widgets::SliderValue;
use bevy::ui_widgets::SliderValueChange;
use bevy::ui_widgets::slider_self_update;
use bevy::{prelude::*, text::EditableText};

use crate::{FontBlock, palette};

#[derive(Component, Clone)]
pub struct PostFxSparkleMarker;

#[derive(Component, Clone)]
pub struct PostFxSparkleSlider;

#[derive(Component, Clone)]
pub struct PostFxSparkleThumb;

#[derive(Component, Clone)]
pub struct PostFxSparkleAmt;

#[derive(Component, Clone)]
pub struct PostFxSparkleText;

#[derive(Component, Clone)]
pub struct SparkleSubmenu;

pub fn spawn_sparkle_submenu(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            SparkleSubmenu,
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
            (1..=1).for_each(|i| {
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
                            spawn_body_text(parent, "Amount", PostFxSparkleMarker, fonts);
                            parent
                                .spawn(horizontal_slider(
                                    PostFxSparkleSlider,
                                    PostFxSparkleThumb,
                                    0.1,
                                ))
                                .observe(slider_self_update);
                            parent
                                .spawn(Node {
                                    column_gap: px(4),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    spawn_textbox(
                                        parent,
                                        palette::width::SMALL_SELECTOR_MENU,
                                        fonts,
                                        PostFxSparkleAmt,
                                        4.,
                                        4,
                                    );
                                    spawn_body_text(parent, "%", NullComponent, fonts);
                                });
                        }
                        _ => unreachable!(),
                    });
            })
        });
}

pub fn sparkle_text_update(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    button_input: Res<ButtonInput<KeyCode>>,
    sliders: Query<Entity, With<PostFxSparkleSlider>>,
    text: Query<&EditableText, With<PostFxSparkleAmt>>,
) {
    if button_input.just_pressed(KeyCode::Enter)
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
pub fn sparkle_slider_update(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange),
        (
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
            With<PostFxSparkleSlider>,
        ),
    >,
    mut thumbs: Query<(&mut Node, Has<PostFxSparkleThumb>), Without<PostFxSparkleSlider>>,
    children: Query<&Children>,
    mut amt_text: Single<&mut Text, With<PostFxSparkleAmt>>,
    mut params: ResMut<StereometerParams>,
) {
    for (slider_ent, value, range) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                amt_text.0 = format!("{:.1}", value.0);
                let position = range.thumb_position(value.0) * 100.0;
                thumb_node.left = percent(position);
                params.color = params
                    .color
                    .with_alpha(((value.0 / 100.) * 10.0).clamp(1.0, 10.0));
            }
        }
    }
}
