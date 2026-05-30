use crate::{FontBlock, palette, ui::interactions::*};
use bevy::{
    prelude::*,
    ui_widgets::{Slider, SliderRange, SliderThumb, SliderValue, TrackClick},
};

pub mod control_panel;
pub mod generator_mode;
pub mod generator_visual;
pub mod interactions;
pub mod postfx_bloom;

pub fn spawn_body_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    cmp: impl Component + Clone,
    font: &FontBlock,
) {
    parent.spawn((
        cmp.clone(),
        Text::new(text),
        TextFont {
            font: font.text.clone(),
            font_size: FontSize::Px(palette::font_size::BIG),
            weight: FontWeight(palette::font_weight::BODY),
            ..Default::default()
        },
        TextColor(palette::BRIGHT),
        LetterSpacing::Px(palette::letter_spacing::BASE),
    ));
}

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

pub fn horizontal_slider(slider_marker: impl Component, thumb: impl Component) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            margin: UiRect::horizontal(px(8)).with_left(px(16)),
            flex_grow: 1.0,
            ..default()
        },
        slider_marker,
        Slider {
            track_click: TrackClick::Snap,
            ..Default::default()
        },
        SliderValue(50.0),
        SliderRange::new(0.0, 100.0),
        Children::spawn((
            // slider track
            Spawn((
                Node {
                    height: px(4),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BorderColor::all(palette::BORDER),
                BackgroundColor(palette::VOID),
            )),
            // slider thumb
            Spawn((
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Start,
                    left: px(-4),
                    right: px(12.),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                children![(
                    thumb,
                    SliderThumb,
                    Node {
                        display: Display::Flex,
                        width: px(16),
                        height: px(16),
                        position_type: PositionType::Absolute,
                        left: percent(0),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(palette::BRIGHT),
                )],
            )),
        )),
    )
}
