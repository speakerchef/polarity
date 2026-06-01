use bevy::prelude::*;

use crate::{
    FontBlock, NullComponent, palette,
    stereometer::{StereometerParams, StereometerRenderMode},
    ui::{
        control_panel::{DropdownItem, SubmenuItem},
        interactions::*,
        menu_row_container, spawn_body_text, spawn_dropdown_row,
    },
};

#[derive(Component, Clone)]
pub struct RenderModeSubmenu;
#[derive(Component, Clone)]
pub struct RenderModeSelectorMenu;
#[derive(Component, Clone)]
pub struct RenderModeDropdown;

pub fn spawn_render_mode_submenu(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    parent
        .spawn((
            SubmenuItem(DropdownItem::Render),
            RenderModeSubmenu,
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
                        (
                            &StereometerRenderMode::default().to_string(),
                            RenderModeSelectorMenu,
                        ),
                        RenderModeDropdown,
                        spawn_render_mode_options,
                    )
                    .observe(render_mode_on_click);
                });
        });
}
fn render_mode_on_click(
    _: On<Pointer<Click>>,
    mut vis: Single<&mut Node, With<RenderModeDropdown>>,
) {
    if vis.display != Display::Flex {
        vis.display = Display::Flex;
    } else {
        vis.display = Display::None;
    }
}

fn spawn_render_mode_options(parent: &mut ChildSpawnerCommands, fonts: &FontBlock) {
    for mode in &[
        StereometerRenderMode::FullSpectrum,
        StereometerRenderMode::MultiBand,
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
                 mut stereo_params: ResMut<StereometerParams>,
                 mut txt: Single<&mut Text, With<RenderModeSelectorMenu>>| {
                    info!("clicked");
                    stereo_params.render_mode = mode.clone();
                    txt.0 = mode.to_string();
                },
            )
            .observe(on_hover_void)
            .observe(on_leave_void);
    }
}

pub fn watch_render_mode(
    params: Res<StereometerParams>,
    filter: Query<(&mut Node, &DropdownItem)>,
) {
    for (mut node, item) in filter {
        if matches!(item, DropdownItem::Filtering) {
            if matches!(params.render_mode, StereometerRenderMode::MultiBand) {
                node.display = match node.display {
                    Display::Flex => Display::None,
                    _ => return,
                }
            } else {
                node.display = match node.display {
                    Display::None => Display::Flex,
                    _ => return,
                }
            }
        }
    }
}
