//! Paleta visual del laboratorio.

use egui_macroquad::egui::{self, Color32, Stroke, Vec2};

pub const BACKGROUND: Color32 = Color32::from_rgb(10, 15, 28);
pub const PANEL: Color32 = Color32::from_rgb(17, 25, 43);
pub const PANEL_RAISED: Color32 = Color32::from_rgb(23, 33, 55);
pub const BORDER: Color32 = Color32::from_rgb(40, 55, 80);
pub const TEXT: Color32 = Color32::from_rgb(231, 239, 250);
pub const MUTED: Color32 = Color32::from_rgb(139, 157, 181);
pub const CYAN: Color32 = Color32::from_rgb(67, 211, 231);
pub const BLUE: Color32 = Color32::from_rgb(94, 150, 255);
pub const GREEN: Color32 = Color32::from_rgb(65, 211, 151);
pub const AMBER: Color32 = Color32::from_rgb(255, 186, 82);
pub const VIOLET: Color32 = Color32::from_rgb(165, 135, 255);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = PANEL;
    visuals.override_text_color = Some(TEXT);
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.inactive.bg_fill = PANEL_RAISED;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(31, 48, 75);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, CYAN);
    visuals.widgets.active.bg_fill = Color32::from_rgb(34, 68, 86);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, CYAN);
    visuals.selection.bg_fill = Color32::from_rgb(27, 86, 103);
    visuals.selection.stroke = Stroke::new(1.0_f32, CYAN);
    visuals.hyperlink_color = CYAN;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(9.0, 9.0);
    style.spacing.button_padding = Vec2::new(10.0, 7.0);
    ctx.set_style(style);
}
