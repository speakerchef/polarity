use crate::{
    AudioFileContents, DrawableCursor, FilteringMode, FontBlock, NullComponent, palette,
    stereometer::{StereoFilter, Stereometer, StereometerParams},
    ui::{
        control_panel::{DropdownItem, SubmenuItem},
        interactions::*,
        menu_row_container, spawn_body_text, spawn_dropdown_row, spawn_slider_row,
    },
};
use bevy::{
    input_focus::InputFocus,
    prelude::*,
    text::EditableText,
    ui_widgets::{SetSliderValue, SliderDragState, SliderRange, SliderValue, SliderValueChange},
};
use biquad::*;

#[derive(Component, Clone)]
pub struct FilterSubmenu;
#[derive(Component, Clone)]
pub struct FilterModeSelectorMenu;
#[derive(Component, Clone)]
pub struct FilterModeDropdown;
#[derive(Component, Clone)]
pub struct FilterFreqSlider;
#[derive(Component, Clone)]
pub struct FilterFreqThumb;
#[derive(Component, Clone)]
pub struct FilterFreqAmt;

pub fn spawn_filtering_submenu(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            SubmenuItem(DropdownItem::Filtering),
            FilterSubmenu,
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
                        "Mode",
                        ("Off", FilterModeSelectorMenu),
                        FilterModeDropdown,
                        spawn_filtering_mode_options,
                    )
                    .observe(filtering_mode_on_click);
                });

            // Freq slider
            let (min, max, step, def) = (20.0, 20000.0, 1.0, 20.0);
            parent
                .spawn(menu_row_container(JustifyContent::SpaceBetween))
                .with_children(|parent| {
                    spawn_slider_row(
                        parent,
                        fonts,
                        "Frequency",
                        (min, max, step, def, FilterFreqSlider, FilterFreqThumb),
                        (palette::width::MED_SELECTOR_MENU, 5, FilterFreqAmt),
                    )
                });
        });
}
fn filtering_mode_on_click(
    _: On<Pointer<Click>>,
    mut vis: Single<&mut Node, With<FilterModeDropdown>>,
) {
    if vis.display != Display::Flex {
        vis.display = Display::Flex;
    } else {
        vis.display = Display::None;
    }
}

fn spawn_filtering_mode_options(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    for mode in &[
        FilteringMode::Off,
        FilteringMode::Lpf,
        FilteringMode::Bpf,
        FilteringMode::Hpf,
    ] {
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
                spawn_body_text(parent, &mode.to_string(), NullComponent, fonts);
            })
            .observe(
                |_: On<Pointer<Click>>,
                 mut commands: Commands,
                 sliders: Query<Entity, With<FilterFreqSlider>>,
                 mut params: ResMut<StereometerParams>,
                 mut txt: Single<&mut Text, With<FilterModeSelectorMenu>>| {
                    info!("clicked");
                    params.filtering_mode = mode.clone();
                    for e in sliders {
                        match mode {
                            FilteringMode::Off => {
                                commands.trigger(SetSliderValue {
                                    entity: e,
                                    change: SliderValueChange::Absolute(0.),
                                });
                            }
                            FilteringMode::Lpf => {
                                commands.trigger(SetSliderValue {
                                    entity: e,
                                    change: SliderValueChange::Absolute(300.),
                                });
                            }
                            FilteringMode::Bpf => {
                                commands.trigger(SetSliderValue {
                                    entity: e,
                                    change: SliderValueChange::Absolute(1000.),
                                });
                            }
                            FilteringMode::Hpf => {
                                commands.trigger(SetSliderValue {
                                    entity: e,
                                    change: SliderValueChange::Absolute(3000.),
                                });
                            }
                        }
                    }
                    txt.0 = mode.to_string();
                },
            )
            .observe(on_hover_void)
            .observe(on_leave_void);
    }
}
pub fn freq_amt_text_update(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    button_input: Res<ButtonInput<KeyCode>>,
    sliders: Query<Entity, With<FilterFreqSlider>>,
    text: Query<&EditableText, With<FilterFreqAmt>>,
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
pub fn freq_slider_update(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange),
        (
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
            With<FilterFreqSlider>,
        ),
    >,
    mut thumbs: Query<(&mut Node, Has<FilterFreqThumb>), Without<FilterFreqSlider>>,
    children: Query<&Children>,
    mut amt_text: Single<&mut EditableText, With<FilterFreqAmt>>,
    mut params: ResMut<StereometerParams>,
    audio: Single<&AudioFileContents>,
    mut stereometer: Single<&mut Stereometer, With<DrawableCursor>>,
) {
    for (slider_ent, value, range) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                amt_text.editor_mut().set_text(&value.0.round().to_string());
                let position = range.thumb_position(value.0) * 100.0;
                thumb_node.left = percent(position);

                // update filters
                if params.freq != value.0.round() {
                    params.freq = value.0.round();
                    stereometer.live_filterbank = Some((
                        StereoFilter::from_coeffs_butterworth(
                            Type::LowPass,
                            params.freq,
                            audio.sample_rate,
                        ),
                        StereoFilter::from_coeffs_butterworth(
                            Type::BandPass,
                            params.freq,
                            audio.sample_rate,
                        ),
                        StereoFilter::from_coeffs_butterworth(
                            Type::HighPass,
                            params.freq,
                            audio.sample_rate,
                        ),
                    ));
                    stereometer.trace_filterbank = Some((
                        StereoFilter::from_coeffs_butterworth(
                            Type::LowPass,
                            params.freq,
                            audio.sample_rate,
                        ),
                        StereoFilter::from_coeffs_butterworth(
                            Type::BandPass,
                            params.freq,
                            audio.sample_rate,
                        ),
                        StereoFilter::from_coeffs_butterworth(
                            Type::HighPass,
                            params.freq,
                            audio.sample_rate,
                        ),
                    ));
                }
            }
        }
    }
}
