use crate::{FontBlock, NullComponent, palette, ui::interactions::*};
use bevy::{
    input_focus::AutoFocus,
    prelude::*,
    text::{EditableText, TextCursorStyle},
    ui_widgets::{
        Slider, SliderRange, SliderStep, SliderThumb, SliderValue, TrackClick, slider_self_update,
    },
};

pub mod control_panel;
pub mod generator_color;
pub mod generator_filtering;
pub mod generator_mode;
pub mod generator_render;
pub mod generator_visual;
pub mod interactions;
pub mod postfx_sparkle;

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
            font_size: FontSize::Px(palette::font_size::BODY),
            weight: FontWeight(palette::font_weight::MED),
            ..Default::default()
        },
        TextLayout::no_wrap().with_justify(Justify::Left),
        TextColor(palette::BRIGHT),
        LetterSpacing::Px(palette::letter_spacing::MINIMAL),
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

pub fn horizontal_slider(
    slider_marker: impl Component,
    thumb: impl Component,
    min: f32,
    max: f32,
    step_amt: f32,
    def: f32,
) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            width: px(palette::width::SLIDER),
            ..default()
        },
        slider_marker,
        Slider {
            track_click: TrackClick::Snap,
            ..Default::default()
        },
        SliderValue(def),
        SliderRange::new(min, max),
        SliderStep(step_amt),
        Children::spawn((
            // slider track
            Spawn((
                Node {
                    height: px(6),
                    border: UiRect::all(px(1)),
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
                        border: UiRect::all(px(1)),
                        width: px(16),
                        height: px(16),
                        position_type: PositionType::Absolute,
                        left: percent(0),
                        ..default()
                    },
                    BackgroundColor(palette::BRIGHT),
                    BorderColor::all(palette::BORDER)
                )],
            )),
        )),
    )
}

pub fn spawn_textbox<'a, T: Component>(
    parent: &'a mut ChildSpawnerCommands,
    sz: f32,
    font: &FontBlock,
    marker: T,
    vis_width: f32,
    max_char: usize,
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
                Node {
                    width: percent(100.0),
                    height: percent(100.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::all(px(4)),
                    ..default()
                },
                EditableText {
                    visible_width: Some(vis_width),
                    max_characters: Some(max_char),
                    allow_newlines: false,

                    ..Default::default()
                },
                TextLayout::no_wrap().with_justify(Justify::Right),
                TextFont {
                    font: font.text.clone(),
                    font_size: FontSize::Px(palette::font_size::MED),
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

pub fn menu_row_container(justify_content: JustifyContent) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            align_items: AlignItems::Center,
            justify_content,
            height: px(palette::height::MENU_ITEM),
            width: percent(100.),
            padding: UiRect::horizontal(px(10)),
            border: UiRect::bottom(px(1)),
            ..Default::default()
        },
        BorderColor::all(palette::BORDER),
    )
}

pub fn spawn_dropdown_row<'a, S, D>(
    parent: &'a mut ChildSpawnerCommands,
    fonts: &FontBlock,
    label_text: &str,
    selector: (&str, S),
    dropdown_marker: D,
    dropdown_spawner: fn(&mut ChildSpawnerCommands, &FontBlock),
) -> EntityCommands<'a>
where
    S: Component + Clone,
    D: Component,
{
    spawn_body_text(parent, label_text, NullComponent, fonts);
    let mut parent_spawner = spawn_selector_with_size(
        parent,
        palette::width::LARGE_SELECTOR_MENU,
        selector.0,
        selector.1,
        fonts,
    );
    parent_spawner.with_children(|parent| {
        parent
            .spawn((
                dropdown_marker,
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
                dropdown_spawner(parent, fonts);
            });
    });
    parent_spawner
}

pub fn spawn_slider_row<S, T, A>(
    parent: &mut ChildSpawnerCommands,
    fonts: &FontBlock,
    label_text: &str,
    slider: (f32, f32, f32, f32, S, T),
    textbox: (f32, usize, A),
) where
    S: Component,
    T: Component,
    A: Component,
{
    parent
        .spawn((Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            width: px(100),
            ..Default::default()
        },))
        .with_children(|parent| spawn_body_text(parent, label_text, NullComponent, fonts));
    let (min, max, step, def, slider_marker, thumb_marker) = slider;
    parent
        .spawn(Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: px(palette::width::SLIDER),
            // flex_grow: 2.0,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(horizontal_slider(
                    slider_marker,
                    thumb_marker,
                    min,
                    max,
                    step,
                    def,
                ))
                .observe(slider_self_update);
        });
    parent
        .spawn(Node {
            column_gap: px(4),
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            width: px(80.),
            ..Default::default()
        })
        .with_children(|parent| {
            let (width, max_chars, textbox_marker) = textbox;
            spawn_textbox(
                parent,
                width,
                fonts,
                textbox_marker,
                max_chars as f32,
                max_chars,
            );
            // spawn_body_text(parent, textbox_postfix_text, NullComponent, fonts);
        });
}
