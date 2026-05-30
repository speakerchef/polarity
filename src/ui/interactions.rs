use crate::palette;
use bevy::prelude::*;

pub fn on_hover_surface(e: On<Pointer<Over>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::SURFACE
    {
        item.0 = palette::SURFACE_HOVER;
    }
}
pub fn on_leave_surface(e: On<Pointer<Out>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::SURFACE_HOVER
    {
        item.0 = palette::SURFACE;
    }
}

pub fn on_hover_bg(e: On<Pointer<Over>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::BG
    {
        item.0 = palette::SURFACE;
    }
}
pub fn on_leave_bg(e: On<Pointer<Out>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::SURFACE
    {
        item.0 = palette::BG;
    }
}

pub fn on_hover_void(e: On<Pointer<Over>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::VOID
    {
        item.0 = palette::SURFACE;
    }
}
pub fn on_leave_void(e: On<Pointer<Out>>, mut submenu: Query<&mut BackgroundColor>) {
    if let Ok(mut item) = submenu.get_mut(e.entity)
        && item.0 == palette::SURFACE
    {
        item.0 = palette::VOID;
    }
}

pub fn on_click_toggle_bright_surface(
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

pub fn on_click_toggle_bright_bg(
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

pub fn toggle_visibility_with_marker<T: Component>(
    _: On<Pointer<Click>>,
    mut node: Single<&mut Node, With<T>>,
) {
    if node.display == Display::Flex {
        node.display = Display::None;
    } else {
        node.display = Display::Flex;
    }
}

pub fn on_hover_bg_bright(
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
pub fn on_leave_bg_bright(
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
