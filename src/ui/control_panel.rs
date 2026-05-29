use crate::ui::generator_mode::{ModeSubmenu, spawn_mode_submenu};
use crate::ui::generator_visual::VisualSubmenu;
use bevy_file_dialog::prelude::*;
use biquad::*;
use std::collections::HashMap;
use std::sync::Arc;

use bevy::audio::{Source, prelude::*};
use bevy::prelude::*;

use crate::stereometer::{StereoFilter, Stereometer, StereometerKind, StereometerParams};
use crate::ui::generator_visual::spawn_visual_submenu;
use crate::{AudioFileContents, FontBlock, palette, ui::interactions::*};
use crate::{DurationText, PlayingAudio, TimelineScrubber};

#[derive(Component, Clone)]
struct GeneratorRootHeader;

#[derive(Component, Clone)]
struct GeneratorSubmenu;

#[derive(Component, Clone)]
struct ModifierRootHeader;

#[derive(Component, Clone)]
pub struct MotionSubmenu;

#[derive(Component, Clone)]
pub enum DropdownItem {
    // GENERATOR
    Mode,
    Motion,
    Visual,
    // POST FX
}

impl From<DropdownItem> for String {
    fn from(value: DropdownItem) -> Self {
        match value {
            DropdownItem::Mode => "MODE".to_string(),
            DropdownItem::Motion => "MOTION".to_string(),
            DropdownItem::Visual => "VISUAL".to_string(),
        }
    }
}
fn spawn_file_handle_button(
    parent: &mut ChildSpawnerCommands,
    fonts: &FontBlock,
    button_text: &str,
    arrow_glyph: &str,
    on_click: fn(On<Pointer<Click>>, Commands),
    add_border: bool,
) {
    parent
        .spawn((
            Button,
            Node {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                height: percent(100.),
                flex_grow: 1.0,
                border: if add_border {
                    UiRect {
                        right: px(palette::FRAME_WIDTH),
                        ..Default::default()
                    }
                } else {
                    UiRect::default()
                },
                ..Default::default()
            },
            BackgroundColor(palette::BG),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(arrow_glyph),
                TextColor(palette::DIM),
                TextFont {
                    font: fonts.icon.clone(),
                    font_size: FontSize::Px(palette::font_size::ICON),
                    ..Default::default()
                },
                LetterSpacing::Px(palette::letter_spacing::BASE),
            ));
            parent.spawn((
                Text::new(button_text),
                TextColor(palette::DIM),
                TextFont {
                    font: fonts.text.clone(),
                    font_size: FontSize::Px(palette::font_size::MED),
                    weight: FontWeight(palette::font_weight::HEAVY),
                    ..Default::default()
                },
                LetterSpacing::Px(palette::letter_spacing::BASE),
            ));
        })
        .observe(on_hover_bg_bright)
        .observe(on_leave_bg_bright)
        .observe(on_click);
}

fn spawn_import_export_button(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            Node {
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                height: px(42),
                flex_shrink: 0.,
                width: percent(100.),
                border: UiRect::left(px(palette::FRAME_WIDTH * 2.))
                    .with_bottom(px(palette::FRAME_WIDTH)),
                ..Default::default()
            },
            BackgroundColor(palette::BG),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            spawn_file_handle_button(
                parent,
                fonts,
                "IMPORT",
                "\u{e5db}",
                |_: On<Pointer<Click>>, commands| import_file(commands),
                true,
            );
            spawn_file_handle_button(
                parent,
                fonts,
                "EXPORT",
                "\u{e5d8}",
                |_: On<Pointer<Click>>, commands| export_file(commands),
                false,
            );
        });
}

fn spawn_submenu_items(
    parent: &mut ChildSpawnerCommands,
    fonts: &FontBlock,
    dropdownitem: &DropdownItem,
) {
    parent
        .spawn((
            dropdownitem.clone(),
            Node {
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                height: px(palette::height::DROPDOWN_ITEM),
                width: percent(100.),
                padding: UiRect::left(px(12)).with_right(px(12)),
                border: UiRect::top(px(2)),
                ..Default::default()
            },
            BorderColor::all(palette::BORDER),
            BackgroundColor(palette::SURFACE),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(dropdownitem.clone()),
                TextFont {
                    font: fonts.text.clone(),
                    font_size: FontSize::Px(palette::font_size::BODY),
                    weight: FontWeight(palette::font_weight::BODY),
                    ..Default::default()
                },
                TextColor(palette::BRIGHT),
                LetterSpacing::Px(palette::letter_spacing::BASE),
            ));
        })
        .observe(on_hover_surface)
        .observe(on_leave_surface)
        .observe(on_click_toggle_bright_surface)
        .observe(generator_submenu_onclick);

    match dropdownitem {
        DropdownItem::Mode => spawn_mode_submenu(parent),
        DropdownItem::Visual => spawn_visual_submenu(parent, fonts),
        _ => info!("No children for this yet"),
    }
}

fn spawn_primary_dropdown_header<TRoot: Component + Clone, TSubmenu: Component>(
    parent: &mut ChildSpawnerCommands,
    name: &str,
    fonts: &FontBlock,
    root_cmp: TRoot,
    item_index: usize,
) {
    parent
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                width: percent(100.),
                border: if item_index != 0 {
                    UiRect::top(px(1))
                } else {
                    default()
                },
                ..Default::default()
            },
            BackgroundColor(palette::SURFACE),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    root_cmp.clone(),
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        height: px(palette::height::ROWHEAD),
                        width: percent(100.),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Node {
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                height: percent(100.),
                                width: px(46),
                                border: UiRect::right(px(1)),
                                ..Default::default()
                            },
                            BorderColor::all(palette::BORDER),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(format!("0{}", item_index + 1)),
                                TextFont {
                                    font: fonts.text.clone(),
                                    font_size: FontSize::Px(palette::font_size::BIG),
                                    weight: FontWeight(palette::font_weight::HEAVY),
                                    ..Default::default()
                                },
                                TextColor(palette::BRIGHT),
                                LetterSpacing::Px(palette::letter_spacing::SPACED),
                            ));
                        });

                    parent
                        .spawn((Node {
                            display: Display::Flex,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::FlexStart,
                            height: px(palette::height::ROWHEAD),
                            width: percent(100.),
                            padding: UiRect::left(px(12)).with_right(px(12)),
                            ..Default::default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(name.to_string()),
                                TextFont {
                                    font: fonts.text.clone(),
                                    font_size: FontSize::Px(palette::font_size::BIG),
                                    weight: FontWeight(palette::font_weight::HEAVY),

                                    ..Default::default()
                                },
                                TextColor(palette::BRIGHT),
                                LetterSpacing::Px(palette::letter_spacing::SPACED),
                            ));
                        });
                });
        })
        .observe(on_hover_surface)
        .observe(on_leave_surface)
        .observe(on_click_toggle_bright_surface)
        .observe(toggle_visibility_with_marker::<TSubmenu>);
}

fn spawn_submenu(
    parent: &mut ChildSpawnerCommands,
    fonts: &FontBlock,
    items: &[DropdownItem],
    marker: impl Component,
) {
    parent
        .spawn(
            // main container
            (
                marker,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    width: percent(100.),
                    ..Default::default()
                },
                Visibility::Hidden,
            ),
        )
        .with_children(|parent| {
            items.iter().for_each(|item| {
                spawn_submenu_items(parent, fonts, item);
            });
        });
}

fn spawn_control_panel_menus(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            Node {
                display: Display::Flex,
                align_items: AlignItems::Start,
                flex_direction: FlexDirection::Column,
                height: percent(100.),
                width: percent(100.),
                border: UiRect::left(px(palette::FRAME_WIDTH * 2.)),
                padding: UiRect::all(px(12.)),
                ..Default::default()
            },
            BackgroundColor(palette::BG),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            spawn_primary_dropdown_header::<GeneratorRootHeader, GeneratorSubmenu>(
                parent,
                "GENERATOR",
                fonts,
                GeneratorRootHeader,
                0,
            );

            spawn_submenu(
                parent,
                fonts,
                &[
                    DropdownItem::Mode,
                    DropdownItem::Visual,
                    DropdownItem::Motion,
                ],
                GeneratorSubmenu,
            );
        });
}

pub fn spawn_control_panel(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    // control panel base
    parent
        .spawn((
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                height: percent(100.),
                width: px(360.),
                flex_shrink: 0.,
                padding: UiRect::left(px(palette::spacing::S1)),
                border: UiRect::left(px(palette::FRAME_WIDTH)),
                ..Default::default()
            },
            BackgroundColor(palette::BG),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            spawn_import_export_button(parent, fonts);
            spawn_control_panel_menus(parent, fonts);
        });
}

fn import_file(mut commands: Commands) {
    commands
        .dialog()
        .set_directory("~")
        .add_filter("Audio", &["wav", "mp3", "m4a", "ogg", "flac"])
        .load_file::<AudioFileContents>();
}

fn export_file(mut _commands: Commands) {
    // TODO: Load file byte contents
    // commands.dialog().save_file::<AudioFileContents>();
}

pub fn file_loaded(
    mut file_events: MessageReader<DialogFileLoaded<AudioFileContents>>,
    existing_files: Query<Entity, With<AudioFileContents>>,
    mut timeline_scrubber: Single<&mut TimelineScrubber>,
    mut decoder_text: Single<&mut Text, With<DurationText>>,
    mut stereometer: Single<&mut Stereometer>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    for f in file_events.read() {
        // Cleanup old file contents
        for e in existing_files {
            if let Ok(mut file) = commands.get_entity(e) {
                file.despawn();
            }
        }

        let bytes: Arc<[u8]> = f.contents.clone().into();
        let audio_src = AudioSource { bytes };
        let duration = audio_src.decoder().total_duration();
        decoder_text.0 = format!("Song Duration: {:.2}s, ", duration.unwrap().as_secs_f64());
        timeline_scrubber.0 = duration;

        let file_contents = AudioFileContents {
            duration: audio_src.decoder().total_duration().unwrap().as_secs_f64(),
            sample_rate: audio_src.decoder().sample_rate().into(),
            num_channels: Into::<u16>::into(audio_src.decoder().channels()) as usize,
            samples: audio_src.decoder().collect(),
        };
        let fs = file_contents.sample_rate.hz();
        let lpf_coeffs =
            Coefficients::<f32>::from_params(Type::LowPass, fs, 300.hz(), Q_BUTTERWORTH_F32)
                .unwrap();
        let bpf_coeffs =
            Coefficients::<f32>::from_params(Type::BandPass, fs, 1.khz(), Q_BUTTERWORTH_F32)
                .unwrap();
        let hpf_coeffs =
            Coefficients::<f32>::from_params(Type::HighPass, fs, 3.khz(), Q_BUTTERWORTH_F32)
                .unwrap();
        stereometer.filterbank = Some(HashMap::from([
            ("lpf".into(), StereoFilter::new(lpf_coeffs)),
            ("bpf".into(), StereoFilter::new(bpf_coeffs)),
            ("hpf".into(), StereoFilter::new(hpf_coeffs)),
        ]));

        commands.spawn((
            PlayingAudio,
            AudioPlayer::new(asset_server.load(f.path.clone())),
            PlaybackSettings {
                paused: true,
                mode: bevy::audio::PlaybackMode::Loop,
                ..Default::default()
            },
            file_contents,
        ));
        info!("Opened file `{}`", f.file_name);
    }
}

pub fn generator_submenu_onclick(
    e: On<Pointer<Click>>,
    mut dropdown_items: Query<&DropdownItem>,
    mode_submenu: Query<&mut Node, (With<ModeSubmenu>, Without<VisualSubmenu>)>,
    // mut motion_submenu: Single<&mut Visibility, With<MotionSubmenu>>,
    visual_submenu: Query<&mut Node, (With<VisualSubmenu>, Without<MotionSubmenu>)>,
) {
    info!("clicked");
    if let Ok(item) = dropdown_items.get_mut(e.entity) {
        match item {
            DropdownItem::Mode => {
                info!("Mode");
                for mut m in mode_submenu {
                    if m.display == Display::Flex {
                        m.display = Display::None;
                    } else {
                        m.display = Display::Flex;
                    }
                }
            }
            DropdownItem::Visual => {
                info!("Visual");
                for mut m in visual_submenu {
                    if m.display == Display::Flex {
                        m.display = Display::None;
                    } else {
                        m.display = Display::Flex;
                    }
                }
            }
            _ => info!("Not implemented this submenu item"),
        }
    }
}
