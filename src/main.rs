use bevy::asset::{RenderAssetUsages, UnapprovedPathMode};
use bevy::audio::Source;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy_file_dialog::prelude::*;
use biquad::*;
use polarity::stereometer::{self, StereoFilter, Stereometer, StereometerKind, StereometerParams};
use polarity::{
    AudioFileContents, CRT_P1, CRT_P7, DrawableCursor, HISTORY_MAGENTA, HistoryDensity,
    HistoryMesh, LIVE_MAGENTA, LiveDensity, LiveMesh, MAX_WINDOW_SIZE, NUM_VERTICES, PlayingAudio,
    PreviewCanvas, TimelineScrubber, palette,
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

struct FontBlock {
    icon: FontSource,
    text: FontSource,
}

#[derive(States, Component, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum GeneratorChoice {
    #[default]
    Stereometer,
    Oscilloscope,
}

#[derive(Component, Clone)]
enum DropdownItem {
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

#[derive(Component, Clone)]
struct ModeSubmenu;

#[derive(Component, Clone)]
struct MotionSubmenu;

#[derive(Component, Clone)]
struct VisualSubmenu;

#[derive(Component, Clone)]
struct VisualDensityText;

#[derive(Component, Clone)]
struct VisualDensitySelectorMenu;

#[derive(Component, Clone)]
struct VisualDensityDropdown;

#[derive(Component, Clone)]
struct VisualPhosphorSelectorMenu;

#[derive(Component, Clone)]
struct VisualPhosphorDropdown;

#[derive(Component, Clone)]
struct VisualDensitySelectorText;

#[derive(Component, Clone)]
struct VisualPhosphorText;

#[derive(Component, Clone)]
struct HeaderDropdown;

#[derive(Component, Clone)]
struct GeneratorRootHeader;

#[derive(Component, Clone)]
struct GeneratorSubmenu;

#[derive(Component, Clone)]
struct ModifierRootHeader;

const SHADER_ASSET_PATH: &str = "shaders/custom_material.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    #[uniform(0)]
    color: LinearRgba,
    alpha_mode: AlphaMode2d,
}

impl Material2d for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        self.alpha_mode
    }
}

#[derive(Component)]
struct DurationText;

fn main() {
    App::new()
        .insert_resource(ClearColor(palette::VOID))
        .insert_resource(StereometerParams::default())
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Polarity".into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Allow,
                    ..Default::default()
                }),
        )
        .add_plugins(Material2dPlugin::<CustomMaterial>::default())
        .add_plugins(
            FileDialogPlugin::new()
                .with_save_file::<AudioFileContents>()
                .with_load_file::<AudioFileContents>(),
        )
        .init_state::<GeneratorChoice>()
        .add_systems(OnEnter(GeneratorChoice::Stereometer), spawn_stereometer)
        .add_systems(OnExit(GeneratorChoice::Stereometer), despawn_stereometer)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (file_loaded, toggle_audio_playback, update_timeline_scrubber),
        )
        .add_systems(
            Update,
            (stereometer::update, stereometer::draw)
                .chain()
                .run_if(in_state(GeneratorChoice::Stereometer)),
        )
        .run();
}

fn on_hover_bg_bright(
    event: On<Pointer<Over>>,
    mut q_bgcol: Query<&mut BackgroundColor, With<Button>>,
    mut q_textcol: Query<&mut TextColor>,
    q_children: Query<&Children>,
) {
    let target = event.event_target();
    if let Ok(mut bgcol) = q_bgcol.get_mut(target) {
        bgcol.0 = palette::TEXT;
        if let Ok(children) = q_children.get(target) {
            for child in children {
                if let Ok(mut c) = q_textcol.get_mut(*child) {
                    c.0 = palette::VOID;
                }
            }
        }
    }
}
fn on_leave_bg_bright(
    event: On<Pointer<Out>>,
    mut q_bgcol: Query<&mut BackgroundColor, With<Button>>,
    mut q_textcol: Query<&mut TextColor>,
    q_children: Query<&Children>,
) {
    let target = event.event_target();
    if let Ok(mut bgcol) = q_bgcol.get_mut(target) {
        bgcol.0 = palette::BG;
        if let Ok(children) = q_children.get(target) {
            for child in children {
                if let Ok(mut c) = q_textcol.get_mut(*child) {
                    c.0 = palette::DIM;
                }
            }
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
    // Import/Export button holder
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

fn spawn_timeline(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            // Timeline base div
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                height: px(104.),
                width: percent(100.),
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            spawn_transport_info(parent);
            spawn_waveform_view(parent);
        });
}

fn spawn_transport_info(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        // transport and file info bar
        Node {
            height: percent(37.),
            width: percent(100.),
            border: UiRect {
                top: px(palette::FRAME_WIDTH),
                bottom: px(palette::FRAME_WIDTH),
                ..Default::default()
            },
            ..Default::default()
        },
        BackgroundColor(palette::BG_MED),
        BorderColor::all(palette::BORDER),
    ));
}

fn spawn_waveform_view(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            // track waveform view section
            Node {
                height: percent(100.),
                width: percent(100.),
                padding: UiRect::all(px(4.)),
                ..Default::default()
            },
            BackgroundColor(palette::BG_MED),
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("Song Duration: , "), DurationText));
            parent.spawn((Text::new("Time Elapsed: 0.0000s"), TimelineScrubber(None)));
        });
}

fn spawn_preview_canvas(parent: &mut ChildSpawnerCommands) {
    // Preview Canvas
    parent.spawn((
        PreviewCanvas,
        Node {
            height: percent(100.),
            min_height: vw(30.),
            flex_grow: 1.,
            min_width: px(0),
            aspect_ratio: Some(16. / 9.),
            ..Default::default()
        },
    ));
}

fn generator_submenu_onclick(
    e: On<Pointer<Click>>,
    mut dropdown_items: Query<&DropdownItem>,
    mut mode_submenu: Query<&mut Node, (With<ModeSubmenu>, Without<VisualSubmenu>)>,
    // mut motion_submenu: Single<&mut Visibility, With<MotionSubmenu>>,
    mut visual_submenu: Query<&mut Node, (With<VisualSubmenu>, Without<MotionSubmenu>)>,
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

fn spawn_mode_submenu(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            ModeSubmenu,
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: percent(100.),
                border: UiRect::horizontal(px(1)).with_top(px(1)),
                padding: UiRect::all(px(palette::APP_PADDING)),
                ..Default::default()
            },
            BackgroundColor(palette::BG),
            BorderColor::all(palette::BORDER),
        ))
        .with_children(|parent| {
            (1..=4).for_each(|i| {
                parent
                    .spawn((
                        Node {
                            display: Display::Flex,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            height: px(60.),
                            width: px(60.),
                            border: UiRect::all(px(palette::FRAME_WIDTH)),
                            ..Default::default()
                        },
                        BackgroundColor(palette::BG),
                        BorderColor::all(palette::BORDER),
                    ))
                    .with_children(|parent| {
                        parent.spawn((Text::new(i.to_string()),));
                    })
                    .observe(on_hover_bg)
                    .observe(on_leave_bg)
                    .observe(match i {
                        1 => |_: On<Pointer<Click>>, mut params: ResMut<StereometerParams>| {
                            params.kind = StereometerKind::LinearBipolar;
                        },
                        2 => |_: On<Pointer<Click>>, mut params: ResMut<StereometerParams>| {
                            params.kind = StereometerKind::ScaledBipolar;
                        },
                        3 => |_: On<Pointer<Click>>, mut params: ResMut<StereometerParams>| {
                            params.kind = StereometerKind::LinearLissajous;
                        },
                        4 => |_: On<Pointer<Click>>, mut params: ResMut<StereometerParams>| {
                            params.kind = StereometerKind::ScaledLissajous;
                        },
                        _ => unreachable!(),
                    });
            });
        });
}

fn spawn_selector_with_size<'a>(
    parent: &'a mut ChildSpawnerCommands,
    sz: f32,
    text: &str,
    marker: impl Component + Clone,
    font: &FontBlock,
) -> bevy::prelude::EntityCommands<'a> {
    let mut parent_spawner = parent.spawn((Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        margin: UiRect::horizontal(px(16)),
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
            .with_children(|parent| spawn_body_text(parent, text, marker, font));
    });
    parent_spawner
}

fn visual_density_on_click(
    _: On<Pointer<Click>>,
    mut vis_phosphor: Single<
        &mut Node,
        (With<VisualPhosphorDropdown>, Without<VisualDensityDropdown>),
    >,
    mut vis_density: Single<
        &mut Node,
        (With<VisualDensityDropdown>, Without<VisualPhosphorDropdown>),
    >,
) {
    if vis_density.display != Display::Flex {
        vis_density.display = Display::Flex;
        if vis_phosphor.display == Display::Flex {
            vis_phosphor.display = Display::None;
        }
    } else {
        vis_density.display = Display::None;
    }
}

fn visual_phosphor_on_click(
    _: On<Pointer<Click>>,
    mut vis_phosphor: Single<
        &mut Node,
        (With<VisualPhosphorDropdown>, Without<VisualDensityDropdown>),
    >,
    mut vis_density: Single<
        &mut Node,
        (With<VisualDensityDropdown>, Without<VisualPhosphorDropdown>),
    >,
) {
    if vis_phosphor.display != Display::Flex {
        vis_phosphor.display = Display::Flex;
        if vis_density.display == Display::Flex {
            vis_density.display = Display::None;
        }
    } else {
        vis_phosphor.display = Display::None;
    }
}

fn spawn_body_text(
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

fn spawn_visual_submenu(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            VisualSubmenu,
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
            (1..=2).for_each(|i| {
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
                            spawn_body_text(parent, "Density", VisualDensityText, fonts);
                            spawn_selector_with_size(
                                parent,
                                palette::width::MED_SELECTOR_MENU,
                                &Into::<String>::into(LiveDensity::default()),
                                VisualDensitySelectorMenu,
                                fonts,
                            )
                            .with_children(|parent| {
                                parent
                                    .spawn((
                                        VisualDensityDropdown,
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
                                        for level in LiveDensity::all() {
                                            parent
                                                .spawn((
                                                    Node {
                                                        display: Display::Flex,
                                                        align_items: AlignItems::Center,
                                                        justify_content: JustifyContent::Center,
                                                        height: px(
                                                            palette::height::SLIDER_ROW_ITEM,
                                                        ),
                                                        width: percent(100.),
                                                        border: UiRect::all(px(1)),
                                                        ..Default::default()
                                                    },
                                                    BorderColor::all(palette::BORDER),
                                                    BackgroundColor(palette::VOID),
                                                ))
                                                .with_children(|parent| {
                                                    spawn_body_text(
                                                        parent,
                                                        &Into::<String>::into(level.clone()),
                                                        VisualDensityText,
                                                        fonts,
                                                    );
                                                })
                                                .observe(|_: On<Pointer<Click>>, mut stereo_params: ResMut<StereometerParams>, mut txt: Single<&mut Text, With<VisualDensitySelectorMenu>>| {
                                                        info!("clicked");
                                                        stereo_params.live_density = level.clone();
                                                        txt.0 = Into::<String>::into(level.clone());
                                                });
                                        }
                                    });
                            })
                            .observe(visual_density_on_click)
                            .observe(on_hover_void)
                            .observe(on_leave_void);
                        }
                        2 => {
                            spawn_body_text(parent, "Phosphor", VisualPhosphorText, fonts);
                            spawn_selector_with_size(
                                parent,
                                palette::width::MED_SELECTOR_MENU,
                                &Into::<String>::into(HistoryDensity::default()),
                                VisualPhosphorSelectorMenu,
                                fonts,
                            )
                            .with_children(|parent| {
                                parent
                                    .spawn((
                                        VisualPhosphorDropdown,
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
                                        for level in HistoryDensity::all() {
                                            parent
                                                .spawn((
                                                    Node {
                                                        display: Display::Flex,
                                                        align_items: AlignItems::Center,
                                                        justify_content: JustifyContent::Center,
                                                        height: px(
                                                            palette::height::SLIDER_ROW_ITEM,
                                                        ),
                                                        width: percent(100.),
                                                        border: UiRect::all(px(1)),
                                                        ..Default::default()
                                                    },
                                                    BorderColor::all(palette::BORDER),
                                                    BackgroundColor(palette::VOID),
                                                ))
                                                .with_children(|parent| {
                                                    spawn_body_text(
                                                        parent,
                                                        &Into::<String>::into(level.clone()),
                                                        VisualPhosphorText,
                                                        fonts,
                                                    );
                                                })
                                                .observe(|_: On<Pointer<Click>>, mut stereo_params: ResMut<StereometerParams>, mut txt: Single<&mut Text, With<VisualPhosphorSelectorMenu>>| {
                                                        info!("clicked");
                                                        stereo_params.history_density = level.clone();
                                                        txt.0 = Into::<String>::into(level.clone());
                                                });
                                        }
                                    });
                            })
                            .observe(visual_phosphor_on_click)
                            .observe(on_hover_void)
                            .observe(on_leave_void);
                        }
                        _ => unreachable!(),
                    });
            })
        });
}

fn on_hover_surface(e: On<Pointer<Over>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::SURFACE
    {
        item.0 = palette::SURFACE_HOVER;
    }
}
fn on_leave_surface(e: On<Pointer<Out>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::SURFACE_HOVER
    {
        item.0 = palette::SURFACE;
    }
}

fn on_hover_bg(e: On<Pointer<Over>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::BG
    {
        item.0 = palette::SURFACE;
    }
}
fn on_leave_bg(e: On<Pointer<Out>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::SURFACE
    {
        item.0 = palette::BG;
    }
}

fn on_hover_void(e: On<Pointer<Over>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::VOID
    {
        item.0 = palette::SURFACE;
    }
}
fn on_leave_void(e: On<Pointer<Out>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::SURFACE
    {
        item.0 = palette::VOID;
    }
}

fn on_click_toggle_bright_surface(
    e: On<Pointer<Click>>,
    mut bgcol: Query<&mut BackgroundColor>,
    mut textcol: Query<&mut TextColor>,
    children: Query<&Children>,
) {
    if let Ok(mut item) = bgcol.get_mut(e.entity) {
        if item.0 != palette::TEXT {
            item.0 = palette::TEXT;
        } else {
            item.0 = palette::SURFACE;
        }

        for child in children.iter_descendants(e.entity) {
            if let Ok(mut textcol) = textcol.get_mut(child) {
                if textcol.0 != palette::VOID {
                    textcol.0 = palette::VOID;
                } else {
                    textcol.0 = palette::BRIGHT;
                }
            }
        }
    }
}

fn on_click_toggle_bright_bg(
    e: On<Pointer<Click>>,
    mut bgcol: Query<&mut BackgroundColor>,
    mut textcol: Query<&mut TextColor>,
    children: Query<&Children>,
) {
    if let Ok(mut item) = bgcol.get_mut(e.entity) {
        if item.0 != palette::TEXT {
            item.0 = palette::TEXT;
        } else {
            item.0 = palette::BG;
        }

        for child in children.iter_descendants(e.entity) {
            if let Ok(mut textcol) = textcol.get_mut(child) {
                if textcol.0 != palette::VOID {
                    textcol.0 = palette::VOID;
                } else {
                    textcol.0 = palette::BRIGHT;
                }
            }
        }
    }
}

fn toggle_visibility_with_marker<T: Component>(
    _: On<Pointer<Click>>,
    mut q_submenu: Single<&mut Visibility, With<T>>,
) {
    q_submenu.toggle_inherited_hidden();
}

fn spawn_submenu_items(
    parent: &mut ChildSpawnerCommands,
    fonts: &FontBlock,
    dropdownitem: &DropdownItem,
    i: usize,
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
                    HeaderDropdown,
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
            items.iter().enumerate().for_each(|(i, item)| {
                spawn_submenu_items(parent, fonts, item, i);
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

fn spawn_control_panel(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
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

fn export_file(mut commands: Commands) {
    // TODO: Load file byte contents
    // commands.dialog().save_file::<AudioFileContents>();
}

fn file_loaded(
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

fn toggle_audio_playback(
    press: Res<ButtonInput<KeyCode>>,
    q_audio: Query<&AudioSink, With<PlayingAudio>>,
) {
    if press.just_pressed(KeyCode::Space)
        && let Ok(sink) = q_audio.single()
    {
        sink.toggle_playback();
    }
}

fn update_timeline_scrubber(
    q_playing_audio: Single<&AudioSink, With<PlayingAudio>>,
    audio: Single<&AudioFileContents>,
    mut q_progress_text: Single<&mut Text, With<TimelineScrubber>>,
) {
    let pos = q_playing_audio.position().as_secs_f64() % audio.duration;
    q_progress_text.0 = format!("Time Elapsed: {:.2}s", pos,);
}

fn spawn_stereometer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let mut live_mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let mut history_mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let live_zeros: Vec<[f32; 3]> = vec![[0., 0., 10.]; MAX_WINDOW_SIZE * NUM_VERTICES];
    let hist_zeros: Vec<[f32; 3]> = vec![[0., 0., 0.]; MAX_WINDOW_SIZE * NUM_VERTICES];
    let hist_colors: Vec<[f32; 4]> = (0..MAX_WINDOW_SIZE)
        .flat_map(|i| {
            let alpha = (i as f32 / MAX_WINDOW_SIZE as f32).powf(2.);
            let c = HISTORY_MAGENTA.with_alpha(alpha).to_f32_array();
            std::iter::repeat_n(c, 6)
        })
        .collect();
    live_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, live_zeros);
    live_mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![LIVE_MAGENTA.to_f32_array(); MAX_WINDOW_SIZE * NUM_VERTICES],
    );
    history_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, hist_zeros);
    history_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, hist_colors);

    let goni_id = commands.spawn_empty().id();

    commands.entity(goni_id).insert((
        Stereometer {
            live_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),
            history_buffer: VecDeque::from([Vec2::ZERO; MAX_WINDOW_SIZE]),
            last_sample_idx: 0,
            id: goni_id,
            filterbank: None,
        },
        DrawableCursor,
    ));
    commands.spawn((
        LiveMesh,
        Mesh2d(meshes.add(live_mesh)),
        MeshMaterial2d(materials.add(CustomMaterial {
            color: LinearRgba::default(),
            alpha_mode: AlphaMode2d::Blend,
        })),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
    commands.spawn((
        HistoryMesh,
        Mesh2d(meshes.add(history_mesh)),
        MeshMaterial2d(materials.add(CustomMaterial {
            color: LinearRgba::default(),
            alpha_mode: AlphaMode2d::Blend,
        })),
    ));
    info!("Spawned Goniometer");
}

fn despawn_stereometer(
    mut commands: Commands,
    q_goniometer: Single<&Stereometer, With<DrawableCursor>>,
) {
    commands.entity(q_goniometer.id).despawn();
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let fonts = FontBlock {
        icon: FontSource::Handle(
            asset_server.load("fonts/material-symbols/MaterialSymbolsOutlined.ttf"),
        ),
        text: FontSource::Handle(asset_server.load("fonts/inter/InterVariable.ttf")),
    };
    commands.spawn((Camera2d, Bloom::default()));
    commands
        .spawn((Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            height: percent(100.),
            width: percent(100.),
            padding: UiRect::all(px(palette::APP_PADDING)),
            ..Default::default()
        },))
        .with_children(|parent| {
            // main app container
            parent
                .spawn((
                    Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        height: percent(100.),
                        width: percent(100.),
                        border: UiRect::all(px(palette::FRAME_WIDTH)),
                        ..Default::default()
                    },
                    BorderColor::all(palette::BORDER),
                ))
                .with_children(|parent| {
                    // Top section holder for canvas + control panel
                    parent
                        .spawn((Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            height: percent(100.),
                            width: percent(100.),
                            ..Default::default()
                        },))
                        .with_children(|parent| {
                            spawn_preview_canvas(parent);
                            spawn_control_panel(parent, &fonts);
                        });
                    // Bottom section holder for timeline
                    parent
                        .spawn((Node {
                            display: Display::Flex,
                            height: px(104.),
                            flex_shrink: 0.,
                            width: percent(100.),
                            ..Default::default()
                        },))
                        .with_children(spawn_timeline);
                });
        });
}
