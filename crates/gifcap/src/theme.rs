//! “Blue breeze” visuals + monospace-first typography.

use eframe::egui::{
    self, Color32, FontFamily, FontId, Rounding, Shadow, Stroke, Style, TextStyle, Visuals,
};

use crate::icon_bitmap;

pub fn app_icon() -> egui::IconData {
    const S: u32 = 64;
    egui::IconData {
        rgba: icon_bitmap::icon_rgba(S),
        width: S,
        height: S,
    }
}

pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::light();
    visuals.dark_mode = false;
    visuals.override_text_color = None;

    // —— Palette (blue breeze) ——
    let mist = Color32::from_rgb(232, 248, 255);
    let panel = Color32::from_rgb(214, 240, 252);
    let panel_deep = Color32::from_rgb(198, 230, 248);
    let accent = Color32::from_rgb(14, 165, 233);
    let accent_h = Color32::from_rgb(56, 189, 248);
    let accent_a = Color32::from_rgb(2, 132, 199);
    let text = Color32::from_rgb(15, 65, 85);
    let text_muted = Color32::from_rgb(55, 115, 140);
    let stroke = Color32::from_rgb(140, 200, 225);
    let warn = Color32::from_rgb(217, 119, 87);
    let err = Color32::from_rgb(200, 75, 90);

    visuals.window_fill = mist;
    visuals.window_stroke = Stroke::new(1.0, stroke);
    visuals.window_rounding = Rounding::same(10.0);
    visuals.window_shadow = Shadow {
        offset: egui::vec2(0.0, 4.0),
        blur: 18.0,
        spread: 0.0,
        color: Color32::from_black_alpha(28),
    };

    visuals.panel_fill = panel;
    visuals.extreme_bg_color = panel_deep;
    visuals.faint_bg_color = mist;
    visuals.code_bg_color = panel_deep;

    visuals.hyperlink_color = accent_a;
    visuals.warn_fg_color = warn;
    visuals.error_fg_color = err;

    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_muted);
    visuals.widgets.noninteractive.bg_fill = panel;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, stroke);
    visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text);
    visuals.widgets.inactive.bg_fill = mist;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, stroke);
    visuals.widgets.inactive.rounding = Rounding::same(6.0);

    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.bg_fill = accent_h;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, accent_a);
    visuals.widgets.hovered.rounding = Rounding::same(6.0);

    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.bg_fill = accent_a;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, accent_a);
    visuals.widgets.active.rounding = Rounding::same(6.0);

    visuals.widgets.open.fg_stroke = Stroke::new(1.0, text);
    visuals.widgets.open.bg_fill = panel_deep;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, accent);
    visuals.widgets.open.rounding = Rounding::same(6.0);

    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(14, 165, 233, 90);
    visuals.selection.stroke = Stroke::new(1.0, accent_a);

    ctx.set_visuals(visuals);

    let mut style = Style::default();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(20.0, FontFamily::Monospace)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Monospace)),
        (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Monospace)),
        (TextStyle::Small, FontId::new(12.0, FontFamily::Monospace)),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(10.0, 6.0);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);
    style.spacing.window_margin = egui::Margin::same(10.0);
    ctx.set_style(style);
}

/// Toolbar strip: sea-breeze panel, blue border (not system gray), minimal vertical padding.
pub fn toolbar_frame(_ctx: &egui::Context, at_top: bool) -> egui::Frame {
    let fill = Color32::from_rgb(206, 236, 252);
    let stroke_c = Color32::from_rgb(90, 170, 210);
    let rounding = if at_top {
        Rounding {
            nw: 0.0,
            ne: 0.0,
            sw: 8.0,
            se: 8.0,
        }
    } else {
        Rounding {
            nw: 8.0,
            ne: 8.0,
            sw: 0.0,
            se: 0.0,
        }
    };
    egui::Frame::default()
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke_c))
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .rounding(rounding)
}

pub fn primary_button(label: impl Into<String>) -> egui::Button<'static> {
    let accent = Color32::from_rgb(14, 165, 233);
    egui::Button::new(
        egui::RichText::new(label.into())
            .color(Color32::WHITE)
            .strong(),
    )
    .fill(accent)
    .stroke(Stroke::new(1.0, Color32::from_rgb(2, 132, 199)))
    .rounding(Rounding::same(6.0))
}

pub fn secondary_button(label: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(label.into()).color(Color32::from_rgb(15, 65, 85)),
    )
    .fill(Color32::from_rgb(232, 248, 255))
    .stroke(Stroke::new(1.0, Color32::from_rgb(90, 170, 210)))
    .rounding(Rounding::same(6.0))
}

/// Destructive action (e.g. discard recording).
pub fn danger_button(label: impl Into<String>) -> egui::Button<'static> {
    let fill = Color32::from_rgb(200, 75, 90);
    let stroke = Color32::from_rgb(160, 45, 58);
    egui::Button::new(
        egui::RichText::new(label.into())
            .color(Color32::WHITE)
            .strong(),
    )
    .fill(fill)
    .stroke(Stroke::new(1.0, stroke))
    .rounding(Rounding::same(6.0))
}

/// Compact control strip (toolbar row).
pub fn apply_toolbar_spacing(ui: &mut egui::Ui) {
    let s = ui.style_mut();
    s.spacing.item_spacing = egui::vec2(6.0, 2.0);
    s.spacing.button_padding = egui::vec2(10.0, 3.0);
}
