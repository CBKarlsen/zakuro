//! egui dressed to look like the rest of macOS: the system font, its
//! controls' sizes and corners, grey surfaces and the blue accent, in the
//! light and the dark look alike.

use egui::epaint::Shadow;
use egui::style::{Selection, WidgetVisuals, Widgets};
use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke, Style, TextStyle, Theme, vec2};
use std::sync::Arc;

/// what the look needs of one of the two schemes.
struct Palette {
    window: Color32,
    panel: Color32,
    field: Color32,
    faint: Color32,
    button: Color32,
    hovered: Color32,
    pressed: Color32,
    line: Color32,
    text: Color32,
    strong: Color32,
    accent: Color32,
}

const LIGHT: Palette = Palette {
    window: Color32::from_rgb(246, 246, 246),
    panel: Color32::from_rgb(236, 236, 236),
    field: Color32::WHITE,
    faint: Color32::from_rgb(242, 242, 242),
    button: Color32::WHITE,
    hovered: Color32::from_rgb(244, 244, 245),
    pressed: Color32::from_rgb(224, 224, 226),
    line: Color32::from_rgb(210, 210, 214),
    text: Color32::from_rgb(60, 60, 67),
    strong: Color32::from_rgb(29, 29, 31),
    accent: Color32::from_rgb(0, 122, 255),
};

const DARK: Palette = Palette {
    window: Color32::from_rgb(44, 44, 46),
    panel: Color32::from_rgb(30, 30, 30),
    field: Color32::from_rgb(26, 26, 26),
    faint: Color32::from_rgb(38, 38, 38),
    button: Color32::from_rgb(63, 63, 66),
    hovered: Color32::from_rgb(76, 76, 80),
    pressed: Color32::from_rgb(94, 94, 98),
    line: Color32::from_rgb(58, 58, 60),
    text: Color32::from_rgb(214, 214, 219),
    strong: Color32::WHITE,
    accent: Color32::from_rgb(10, 132, 255),
};

/// the system's font in front of the others, for text and for code, which
/// stay behind it for what it lacks. a Mac that hasn't the files keeps
/// egui's own.
pub fn fonts(definitions: &mut FontDefinitions) {
    for (family, path) in [(FontFamily::Proportional, "/System/Library/Fonts/SFNS.ttf"), (FontFamily::Monospace, "/System/Library/Fonts/SFNSMono.ttf")] {
        let Ok(bytes) = std::fs::read(path) else { continue };
        let name = format!("system {path}");
        definitions.font_data.insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
        definitions.families.entry(family).or_default().insert(0, name);
    }
}

/// the look for both schemes, the system's pick of them is the one shown.
pub fn style(ctx: &egui::Context) {
    for (theme, palette) in [(Theme::Light, &LIGHT), (Theme::Dark, &DARK)] {
        ctx.style_mut_of(theme, |style| apply(style, palette));
    }
}

fn apply(style: &mut Style, palette: &Palette) {
    style.text_styles = [
        (TextStyle::Heading, FontId::proportional(18.0)),
        (TextStyle::Body, FontId::proportional(14.0)),
        (TextStyle::Button, FontId::proportional(14.0)),
        (TextStyle::Small, FontId::proportional(12.0)),
        (TextStyle::Monospace, FontId::monospace(13.0)),
    ]
    .into();

    let spacing = &mut style.spacing;
    spacing.item_spacing = vec2(8.0, 7.0);
    spacing.button_padding = vec2(12.0, 4.0);
    spacing.interact_size.y = 24.0;
    spacing.window_margin = Margin::same(16);
    spacing.menu_margin = Margin::same(6);

    let visuals = &mut style.visuals;
    visuals.panel_fill = palette.panel;
    visuals.window_fill = palette.window;
    visuals.extreme_bg_color = palette.field;
    visuals.faint_bg_color = palette.faint;
    visuals.hyperlink_color = palette.accent;
    visuals.window_stroke = Stroke::new(1.0, palette.line);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.window_shadow = Shadow { offset: [0, 10], blur: 28, spread: 0, color: Color32::from_black_alpha(70) };
    visuals.popup_shadow = visuals.window_shadow;
    visuals.selection = Selection { bg_fill: palette.accent, stroke: Stroke::new(1.0, Color32::WHITE) };
    visuals.widgets = widgets(palette);
}

fn widgets(palette: &Palette) -> Widgets {
    let radius = CornerRadius::same(6);
    let button = |fill: Color32, text: Color32| WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, palette.line),
        corner_radius: radius,
        fg_stroke: Stroke::new(1.0, text),
        expansion: 0.0,
    };
    Widgets {
        noninteractive: WidgetVisuals {
            bg_fill: palette.panel,
            weak_bg_fill: palette.panel,
            bg_stroke: Stroke::new(1.0, palette.line),
            corner_radius: radius,
            fg_stroke: Stroke::new(1.0, palette.text),
            expansion: 0.0,
        },
        inactive: button(palette.button, palette.text),
        hovered: button(palette.hovered, palette.strong),
        active: button(palette.pressed, palette.strong),
        open: button(palette.pressed, palette.strong),
    }
}
