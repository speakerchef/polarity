use crate::{
    AudioFileContents, DrawableCursor, FilteringMode, FontBlock, NullComponent, palette,
    stereometer::{StereoFilter, Stereometer, StereometerParams},
    ui::{
        horizontal_slider, interactions::*, spawn_body_text, spawn_selector_with_size,
        spawn_textbox,
    },
};
use bevy::{
    input_focus::InputFocus,
    prelude::*,
    text::EditableText,
    ui_widgets::{
        SetSliderValue, SliderDragState, SliderRange, SliderValue, SliderValueChange,
        slider_self_update,
    },
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
                .with_children(|parent| {
                    spawn_body_text(parent, "Mode", NullComponent, fonts);
                    spawn_selector_with_size(
                        parent,
                        palette::width::MED_SELECTOR_MENU,
                        "Off",
                        FilterModeSelectorMenu,
                        fonts,
                    )
                    .with_children(|parent| {
                        parent
                            .spawn((
                                FilterModeDropdown,
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
                                spawn_filtering_mode_options(parent, fonts);
                            });
                    })
                    .observe(filtering_mode_on_click);
                });
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
                .with_children(|parent| {
                    spawn_body_text(parent, "Freq", NullComponent, fonts);
                    parent
                        .spawn(horizontal_slider(FilterFreqSlider, FilterFreqThumb, 1.0))
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
                                FilterFreqAmt,
                                5.,
                                5,
                            );
                            spawn_body_text(parent, "Hz", NullComponent, fonts);
                        });
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
                    let scale = 100. / 20000.;
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
                                    change: SliderValueChange::Absolute(300. * scale),
                                });
                            }
                            FilteringMode::Bpf => {
                                commands.trigger(SetSliderValue {
                                    entity: e,
                                    change: SliderValueChange::Absolute(1000. * scale),
                                });
                            }
                            FilteringMode::Hpf => {
                                commands.trigger(SetSliderValue {
                                    entity: e,
                                    change: SliderValueChange::Absolute(3000. * scale),
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
    mut amt_text: Single<&mut Text, With<FilterFreqAmt>>,
) {
    for (slider_ent, value, range) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                amt_text.0 = format!("{}", (value.0 * 200.0).round());
                let position = range.thumb_position(value.0) * 100.0;
                thumb_node.left = percent(position);
            }
        }
    }
}

pub fn update_filter_freq(
    mut params: ResMut<StereometerParams>,
    audio: Single<&AudioFileContents>,
    mut goniometer: Single<&mut Stereometer, With<DrawableCursor>>,
    text: Single<&Text, With<FilterFreqAmt>>,
) {
    if let Ok(val) = text.0.parse::<f32>()
        && params.freq != val as u32
    {
        params.freq = val as u32;
        info!("{}", params.freq);
        let lpf_coeffs = Coefficients::<f32>::from_params(
            Type::LowPass,
            audio.sample_rate.hz(),
            params.freq.clamp(1, 20000).hz(),
            Q_BUTTERWORTH_F32,
        )
        .unwrap();
        let bpf_coeffs = Coefficients::<f32>::from_params(
            Type::BandPass,
            audio.sample_rate.hz(),
            params.freq.clamp(1, 20000).hz(),
            Q_BUTTERWORTH_F32,
        )
        .unwrap();
        let hpf_coeffs = Coefficients::<f32>::from_params(
            Type::HighPass,
            audio.sample_rate.hz(),
            params.freq.clamp(1, 20000).hz(),
            Q_BUTTERWORTH_F32,
        )
        .unwrap();
        goniometer.live_filterbank = Some((
            StereoFilter::new(lpf_coeffs),
            StereoFilter::new(bpf_coeffs),
            StereoFilter::new(hpf_coeffs),
        ));
        goniometer.history_filterbank = Some((
            StereoFilter::new(lpf_coeffs),
            StereoFilter::new(bpf_coeffs),
            StereoFilter::new(hpf_coeffs),
        ));
    }
}
