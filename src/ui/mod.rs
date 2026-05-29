use crate::{FontBlock, palette};
use bevy::prelude::*;

pub mod control_panel;
pub mod generator_mode;
pub mod generator_visual;
pub mod interactions;

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
