use crate::NullComponent;
use crate::stereometer::StereometerParams;
use crate::ui::horizontal_slider;
use crate::ui::spawn_body_text;
use bevy::input;
use bevy::input_focus::AutoFocus;
use bevy::input_focus::InputFocus;
use bevy::ui_widgets::SetSliderValue;
use bevy::ui_widgets::SliderDragState;
use bevy::ui_widgets::SliderPrecision;
use bevy::ui_widgets::SliderRange;
use bevy::ui_widgets::SliderValue;
use bevy::ui_widgets::SliderValueChange;
use bevy::ui_widgets::slider_self_update;
use bevy::{
    prelude::*,
    text::{EditableText, TextCursorStyle},
};

use crate::{FontBlock, palette, ui::interactions::*};

#[derive(Component, Clone)]
pub struct PostFxBloomMarker;

#[derive(Component, Clone)]
pub struct PostFxBloomAmtSlider;

#[derive(Component, Clone)]
pub struct PostFxBloomSliderThumb;

#[derive(Component, Clone)]
pub struct PostFxBloomAmtValue;

#[derive(Component, Clone)]
pub struct PostFxBloomText;

#[derive(Component, Clone)]
pub struct BloomSubmenu;
pub fn spawn_textbox<'a, T: Component>(
    parent: &'a mut ChildSpawnerCommands,
    sz: f32,
    font: &FontBlock,
    marker: T,
) -> bevy::prelude::EntityCommands<'a> {
    let mut parent_spawner = parent.spawn((
        Node {
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            height: px(palette::height::INNER),
            border: UiRect::all(px(1)),
            width: px(sz),
            column_gap: px(2),
            ..Default::default()
        },
        AutoFocus,
        BorderColor::all(palette::BORDER),
        BackgroundColor(palette::VOID),
    ));
    parent_spawner
        .with_children(|parent| {
            parent.spawn((
                marker,
                EditableText {
                    visible_width: Some(4.),
                    max_characters: Some(4),
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
                Text::new("0"),
                TextCursorStyle::default(),
            ));
        })
        .observe(on_hover_void)
        .observe(on_leave_void);
    parent_spawner
}

pub fn spawn_bloom_submenu(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            BloomSubmenu,
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
                            spawn_body_text(parent, "Amount", PostFxBloomMarker, fonts);
                            parent
                                .spawn(horizontal_slider(
                                    PostFxBloomAmtSlider,
                                    PostFxBloomSliderThumb,
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
                                        PostFxBloomAmtValue,
                                    );
                                    spawn_body_text(parent, "%", NullComponent, fonts);
                                });
                        }
                        _ => unreachable!(),
                    });
            })
        });
}

pub fn bloom_text_update(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    button_input: Res<ButtonInput<KeyCode>>,
    sliders: Query<Entity, With<PostFxBloomAmtSlider>>,
    text: Query<&EditableText, With<PostFxBloomAmtValue>>,
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

pub fn bloom_slider_update(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange, &SliderDragState),
        (
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
            With<PostFxBloomAmtSlider>,
        ),
    >,
    mut thumbs: Query<
        (&mut Node, &mut BackgroundColor, Has<PostFxBloomSliderThumb>),
        Without<PostFxBloomAmtSlider>,
    >,
    children: Query<&Children>,
    mut amt_text: Single<&mut Text, (With<PostFxBloomAmtValue>)>,
    mut params: ResMut<StereometerParams>,
) {
    for (slider_ent, value, range, drag_state) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, mut thumb_bg, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                amt_text.0 = format!("{:.1}", value.0);
                let position = range.thumb_position(value.0) * 100.0;
                thumb_node.left = percent(position);
                info!("Alpha Raw: {}", value.0);
                params.color = params.color.with_alpha((value.0 / 100.) * 10.0);
            }
        }
    }
}
