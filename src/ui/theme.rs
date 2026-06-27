use std::collections::BTreeMap;

use eframe::egui::{
    self, CornerRadius, FontData, FontDefinitions, FontFamily, Margin, Stroke, Vec2,
};

use crate::ui::palette;

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // Load fonts
    fonts.font_data.insert(
        "inter_regular".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/inter/InterRegular.ttf")).into(),
    );
    fonts.font_data.insert(
        "inter_medium".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/inter/InterMedium.ttf")).into(),
    );
    fonts.font_data.insert(
        "inter_bold".to_owned(),
        FontData::from_static(include_bytes!("../../assets/fonts/inter/InterBold.ttf")).into(),
    );
    fonts.font_data.insert(
        "mono".to_owned(),
        FontData::from_static(include_bytes!(
            "../../assets/fonts/mono/JetBrainsMonoNL-Regular.ttf"
        ))
        .into(),
    );
    fonts.font_data.insert(
        "icons".into(),
        FontData::from_static(include_bytes!(
            "../../assets/fonts/material-symbols/MaterialSymbolsOutlined.ttf"
        ))
        .into(),
    );

    let mut newfam = BTreeMap::new();
    newfam.insert(
        FontFamily::Name("inter_regular".into()),
        vec!["inter_regular".to_owned()],
    );
    fonts.families.append(&mut newfam);
    let mut newfam = BTreeMap::new();
    newfam.insert(
        FontFamily::Name("inter_medium".into()),
        vec!["inter_medium".to_owned()],
    );
    fonts.families.append(&mut newfam);
    let mut newfam = BTreeMap::new();
    newfam.insert(
        FontFamily::Name("inter_bold".into()),
        vec!["inter_bold".to_owned()],
    );
    fonts.families.append(&mut newfam);

    let mut newfam = BTreeMap::new();
    newfam.insert(FontFamily::Name("mono".into()), vec!["mono".to_owned()]);
    fonts.families.append(&mut newfam);

    let mut newfam = BTreeMap::new();
    newfam.insert(FontFamily::Name("icons".into()), vec!["icons".to_owned()]);
    fonts.families.append(&mut newfam);

    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &egui::Context, dark: bool) {
    use palette as p;

    let bdr = Stroke::new(p::FRAME_WIDTH, p::BORDER(dark));
    let mut v = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    v.panel_fill = p::BG(dark);
    v.window_fill = p::BG_DARK(dark);
    v.extreme_bg_color = p::VOID(dark);
    v.faint_bg_color = p::SURFACE(dark);
    v.override_text_color = Some(p::TEXT);
    v.window_stroke = bdr;
    v.window_corner_radius = CornerRadius::ZERO;
    v.selection.bg_fill = p::SURFACE_HOVER(dark);
    v.selection.stroke = Stroke::new(p::FRAME_WIDTH, p::BRIGHT);

    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.bg_stroke = bdr;
        w.corner_radius = CornerRadius::ZERO;
        w.expansion = 0.0;
        w.fg_stroke = Stroke::new(1.0, p::TEXT);
    }
    v.widgets.noninteractive.bg_fill = p::BG(dark);
    v.widgets.inactive.bg_fill = p::SURFACE(dark);
    v.widgets.inactive.weak_bg_fill = p::VOID(dark);
    v.widgets.hovered.bg_fill = p::SURFACE_HOVER(dark);
    v.widgets.hovered.weak_bg_fill = p::SURFACE_HOVER(dark);

    ctx.set_visuals(v);

    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = Vec2::ZERO;
    style.spacing.window_margin = Margin::same(p::APP_PADDING as i8);
    ctx.set_global_style(style);
}
