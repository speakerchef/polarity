use bevy::{
    input_focus::InputFocus,
    prelude::*,
    text::EditableText,
    ui_widgets::{SetSliderValue, SliderDragState, SliderRange, SliderValue, SliderValueChange},
};

use crate::{
    FontBlock, NullComponent, palette,
    stereometer::{StereometerParams, StereometerRenderMode},
    ui::{
        control_panel::{DropdownItem, SubmenuItem},
        menu_row_container, spawn_body_text, spawn_slider_row,
    },
};

#[derive(Component, Clone)]
pub struct ColorSubmenu;
#[derive(Component, Clone)]
pub struct MBColorSubmenu;

#[derive(Component, Clone)]
pub struct MainHsla;
#[derive(Component, Clone)]
pub struct LowHsla;
#[derive(Component, Clone)]
pub struct MidHsla;
#[derive(Component, Clone)]
pub struct HighHsla;

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

#[derive(Component, Clone)]
pub struct LowHueSlider;
#[derive(Component, Clone)]
pub struct LowHueThumb;
#[derive(Component, Clone)]
pub struct LowHueAmt;

#[derive(Component, Clone)]
pub struct LowSaturationSlider;
#[derive(Component, Clone)]
pub struct LowSaturationThumb;
#[derive(Component, Clone)]
pub struct LowSaturationAmt;

#[derive(Component, Clone)]
pub struct LowLuminanceSlider;
#[derive(Component, Clone)]
pub struct LowLuminanceThumb;
#[derive(Component, Clone)]
pub struct LowLuminanceAmt;

#[derive(Component, Clone)]
pub struct MidHueSlider;
#[derive(Component, Clone)]
pub struct MidHueThumb;
#[derive(Component, Clone)]
pub struct MidHueAmt;

#[derive(Component, Clone)]
pub struct MidSaturationSlider;
#[derive(Component, Clone)]
pub struct MidSaturationThumb;
#[derive(Component, Clone)]
pub struct MidSaturationAmt;

#[derive(Component, Clone)]
pub struct MidLuminanceSlider;
#[derive(Component, Clone)]
pub struct MidLuminanceThumb;
#[derive(Component, Clone)]
pub struct MidLuminanceAmt;

#[derive(Component, Clone)]
pub struct HighHueSlider;
#[derive(Component, Clone)]
pub struct HighHueThumb;
#[derive(Component, Clone)]
pub struct HighHueAmt;

#[derive(Component, Clone)]
pub struct HighSaturationSlider;
#[derive(Component, Clone)]
pub struct HighSaturationThumb;
#[derive(Component, Clone)]
pub struct HighSaturationAmt;

#[derive(Component, Clone)]
pub struct HighLuminanceSlider;
#[derive(Component, Clone)]
pub struct HighLuminanceThumb;
#[derive(Component, Clone)]
pub struct HighLuminanceAmt;

fn spawn_hsl_sliders<HS, HT, HA, SS, ST, SA, LS, LT, LA>(
    parent: &mut ChildSpawnerCommands,
    fonts: &FontBlock,
    hue: (HS, HT, HA),
    sat: (SS, ST, SA),
    lum: (LS, LT, LA),
    band: ColorBand,
) where
    HS: Component,
    HT: Component,
    HA: Component,
    SS: Component,
    ST: Component,
    SA: Component,
    LS: Component,
    LT: Component,
    LA: Component,
{
    parent
        .spawn(menu_row_container(JustifyContent::SpaceBetween, None))
        .with_children(|parent| {
            spawn_slider_row(
                parent,
                fonts,
                "Hue",
                (
                    0.0,
                    360.0,
                    0.1,
                    match band {
                        ColorBand::Full => 120.0,
                        ColorBand::Low => 0.0,
                        ColorBand::Mid => 120.0,
                        ColorBand::High => 240.0,
                    },
                    hue.0,
                    hue.1,
                ),
                (palette::width::SMALL_SELECTOR_MENU, 5, hue.2),
            );
        });
    parent
        .spawn(menu_row_container(JustifyContent::SpaceBetween, None))
        .with_children(|parent| {
            spawn_slider_row(
                parent,
                fonts,
                "Saturation",
                (0.0, 1.0, 0.001, 1.0, sat.0, sat.1),
                (palette::width::SMALL_SELECTOR_MENU, 4, sat.2),
            );
        });
    parent
        .spawn(menu_row_container(JustifyContent::SpaceBetween, None))
        .with_children(|parent| {
            spawn_slider_row(
                parent,
                fonts,
                "Luminance",
                (0.0, 1.0, 0.001, 0.5, lum.0, lum.1),
                (palette::width::SMALL_SELECTOR_MENU, 4, lum.2),
            );
        });
}
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
            spawn_hsl_sliders(
                parent,
                fonts,
                (HueSlider, HueThumb, HueAmt),
                (SaturationSlider, SaturationThumb, SaturationAmt),
                (LuminanceSlider, LuminanceThumb, LuminanceAmt),
                ColorBand::Full,
            );
        });

    parent
        .spawn((
            SubmenuItem(DropdownItem::Color),
            MBColorSubmenu,
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
                .spawn(menu_row_container(
                    JustifyContent::SpaceBetween,
                    Some(palette::SURFACE),
                ))
                .with_children(|parent| spawn_body_text(parent, "LOW BAND", NullComponent, fonts));
            spawn_hsl_sliders(
                parent,
                fonts,
                (LowHueSlider, LowHueThumb, LowHueAmt),
                (LowSaturationSlider, LowSaturationThumb, LowSaturationAmt),
                (LowLuminanceSlider, LowLuminanceThumb, LowLuminanceAmt),
                ColorBand::Low,
            );
            parent
                .spawn(menu_row_container(
                    JustifyContent::SpaceBetween,
                    Some(palette::SURFACE),
                ))
                .with_children(|parent| spawn_body_text(parent, "MID BAND", NullComponent, fonts));
            spawn_hsl_sliders(
                parent,
                fonts,
                (MidHueSlider, MidHueThumb, MidHueAmt),
                (MidSaturationSlider, MidSaturationThumb, MidSaturationAmt),
                (MidLuminanceSlider, MidLuminanceThumb, MidLuminanceAmt),
                ColorBand::Mid,
            );
            parent
                .spawn(menu_row_container(
                    JustifyContent::SpaceBetween,
                    Some(palette::SURFACE),
                ))
                .with_children(|parent| spawn_body_text(parent, "HIGH BAND", NullComponent, fonts));
            spawn_hsl_sliders(
                parent,
                fonts,
                (HighHueSlider, HighHueThumb, HighHueAmt),
                (HighSaturationSlider, HighSaturationThumb, HighSaturationAmt),
                (HighLuminanceSlider, HighLuminanceThumb, HighLuminanceAmt),
                ColorBand::High,
            );
        });
}

pub fn set_color_display_with_render_mode(
    params: Res<StereometerParams>,
    mut fscolor: Single<&mut Node, With<ColorSubmenu>>,
    mut mbcolor: Single<&mut Node, (With<MBColorSubmenu>, Without<ColorSubmenu>)>,
) {
    match params.render_mode {
        StereometerRenderMode::MultiBand => {
            if fscolor.display == Display::Flex {
                fscolor.display = Display::None;
                mbcolor.display = Display::Flex;
            }
        }
        StereometerRenderMode::FullSpectrum => {
            if mbcolor.display == Display::Flex {
                mbcolor.display = Display::None;
                fscolor.display = Display::Flex;
            }
        }
    }
}

fn local_text_update<S: Component, A: Component>(
    commands: &mut Commands,
    input_focus: &Res<InputFocus>,
    button_input: &Res<ButtonInput<KeyCode>>,
    sliders: Query<Entity, With<S>>,
    text: Query<&EditableText, With<A>>,
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

enum HslaField {
    Hue,
    Sat,
    Lum,
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn local_slider_update<S, A, T>(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange),
        (
            Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
            With<S>,
        ),
    >,
    mut thumbs: Query<(&mut Node, Has<T>), Without<S>>,
    children: &Query<&Children>,
    mut amt_text: Single<&mut EditableText, With<A>>,
    params: &mut ResMut<StereometerParams>,
    dp: usize,
    field: HslaField,
    band: ColorBand,
) where
    S: Component,
    A: Component,
    T: Component,
{
    for (slider_ent, value, range) in sliders.iter() {
        for child in children.iter_descendants(slider_ent) {
            if let Ok((mut thumb_node, is_thumb)) = thumbs.get_mut(child)
                && is_thumb
            {
                match dp {
                    1 => amt_text.editor_mut().set_text(&format!("{:.1}", value.0)),
                    2 => amt_text.editor_mut().set_text(&format!("{:.2}", value.0)),
                    _ => amt_text.editor_mut().set_text(&format!("{:.3}", value.0)),
                }
                let position = range.thumb_position(value.0) * 100.0;
                thumb_node.left = percent(position);

                match field {
                    HslaField::Hue => match band {
                        ColorBand::Full => params.color.hue = value.0,
                        ColorBand::Low => params.multiband_color.0.hue = value.0,
                        ColorBand::Mid => params.multiband_color.1.hue = value.0,
                        ColorBand::High => params.multiband_color.2.hue = value.0,
                    },
                    HslaField::Sat => match band {
                        ColorBand::Full => params.color.saturation = value.0,
                        ColorBand::Low => params.multiband_color.0.saturation = value.0,
                        ColorBand::Mid => params.multiband_color.1.saturation = value.0,
                        ColorBand::High => params.multiband_color.2.saturation = value.0,
                    },
                    HslaField::Lum => match band {
                        ColorBand::Full => params.color.lightness = value.0,
                        ColorBand::Low => params.multiband_color.0.lightness = value.0,
                        ColorBand::Mid => params.multiband_color.1.lightness = value.0,
                        ColorBand::High => params.multiband_color.2.lightness = value.0,
                    },
                }
            }
        }
    }
}

pub enum ColorBand {
    Full,
    Low,
    Mid,
    High,
}

pub trait HslaSliderUpdater {
    type HueSlider: Component;
    type HueAmt: Component;
    type HueThumb: Component;
    type SatSlider: Component;
    type SatAmt: Component;
    type SatThumb: Component;
    type LumSlider: Component;
    type LumAmt: Component;
    type LumThumb: Component;

    const COLOR_BAND: ColorBand;

    #[allow(clippy::too_many_arguments)]
    fn text_update(
        mut commands: Commands,
        input_focus: Res<InputFocus>,
        button_input: Res<ButtonInput<KeyCode>>,
        hue_sliders: Query<Entity, With<Self::HueSlider>>,
        hue_text: Query<&EditableText, With<Self::HueAmt>>,
        sat_sliders: Query<Entity, With<Self::SatSlider>>,
        sat_text: Query<&EditableText, With<Self::SatAmt>>,
        lum_sliders: Query<Entity, With<Self::LumSlider>>,
        lum_text: Query<&EditableText, With<Self::LumAmt>>,
    ) {
        local_text_update(
            &mut commands,
            &input_focus,
            &button_input,
            hue_sliders,
            hue_text,
        );
        local_text_update(
            &mut commands,
            &input_focus,
            &button_input,
            sat_sliders,
            sat_text,
        );
        local_text_update(
            &mut commands,
            &input_focus,
            &button_input,
            lum_sliders,
            lum_text,
        );
    }

    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    fn slider_update(
        hue_sliders: Query<
            (Entity, &SliderValue, &SliderRange),
            (
                Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
                With<Self::HueSlider>,
            ),
        >,
        sat_sliders: Query<
            (Entity, &SliderValue, &SliderRange),
            (
                Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
                With<Self::SatSlider>,
            ),
        >,
        lum_sliders: Query<
            (Entity, &SliderValue, &SliderRange),
            (
                Or<(Changed<SliderValue>, Changed<SliderDragState>)>,
                With<Self::LumSlider>,
            ),
        >,
        mut psthumb: ParamSet<(
            Query<(&mut Node, Has<Self::HueThumb>), Without<Self::HueSlider>>,
            Query<(&mut Node, Has<Self::SatThumb>), Without<Self::SatSlider>>,
            Query<(&mut Node, Has<Self::LumThumb>), Without<Self::LumSlider>>,
        )>,
        mut pstext: ParamSet<(
            Single<&mut EditableText, With<Self::HueAmt>>,
            Single<&mut EditableText, With<Self::SatAmt>>,
            Single<&mut EditableText, With<Self::LumAmt>>,
        )>,
        children: Query<&Children>,
        mut params: ResMut<StereometerParams>,
    ) {
        local_slider_update(
            hue_sliders,
            psthumb.p0(),
            &children,
            pstext.p0(),
            &mut params,
            1,
            HslaField::Hue,
            Self::COLOR_BAND,
        );
        local_slider_update(
            sat_sliders,
            psthumb.p1(),
            &children,
            pstext.p1(),
            &mut params,
            2,
            HslaField::Sat,
            Self::COLOR_BAND,
        );
        local_slider_update(
            lum_sliders,
            psthumb.p2(),
            &children,
            pstext.p2(),
            &mut params,
            2,
            HslaField::Lum,
            Self::COLOR_BAND,
        );
    }
}

impl HslaSliderUpdater for MainHsla {
    type HueSlider = HueSlider;
    type HueAmt = HueAmt;
    type HueThumb = HueThumb;
    type SatSlider = SaturationSlider;
    type SatAmt = SaturationAmt;
    type SatThumb = SaturationThumb;
    type LumSlider = LuminanceSlider;
    type LumAmt = LuminanceAmt;
    type LumThumb = LuminanceThumb;
    const COLOR_BAND: ColorBand = ColorBand::Full;
}
impl HslaSliderUpdater for LowHsla {
    type HueSlider = LowHueSlider;
    type HueAmt = LowHueAmt;
    type HueThumb = LowHueThumb;
    type SatSlider = LowSaturationSlider;
    type SatAmt = LowSaturationAmt;
    type SatThumb = LowSaturationThumb;
    type LumSlider = LowLuminanceSlider;
    type LumAmt = LowLuminanceAmt;
    type LumThumb = LowLuminanceThumb;
    const COLOR_BAND: ColorBand = ColorBand::Low;
}
impl HslaSliderUpdater for MidHsla {
    type HueSlider = MidHueSlider;
    type HueAmt = MidHueAmt;
    type HueThumb = MidHueThumb;
    type SatSlider = MidSaturationSlider;
    type SatAmt = MidSaturationAmt;
    type SatThumb = MidSaturationThumb;
    type LumSlider = MidLuminanceSlider;
    type LumAmt = MidLuminanceAmt;
    type LumThumb = MidLuminanceThumb;
    const COLOR_BAND: ColorBand = ColorBand::Mid;
}
impl HslaSliderUpdater for HighHsla {
    type HueSlider = HighHueSlider;
    type HueAmt = HighHueAmt;
    type HueThumb = HighHueThumb;
    type SatSlider = HighSaturationSlider;
    type SatAmt = HighSaturationAmt;
    type SatThumb = HighSaturationThumb;
    type LumSlider = HighLuminanceSlider;
    type LumAmt = HighLuminanceAmt;
    type LumThumb = HighLuminanceThumb;
    const COLOR_BAND: ColorBand = ColorBand::High;
}
