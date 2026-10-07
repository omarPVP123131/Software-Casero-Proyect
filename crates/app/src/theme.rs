//! Paletas y tipografía del laboratorio.

use egui_macroquad::egui::{self, Color32, Stroke, Vec2};

use crate::state::ThemeChoice;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Color32,
    pub panel: Color32,
    pub raised: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub cyan: Color32,
    pub blue: Color32,
    pub green: Color32,
    pub amber: Color32,
    pub violet: Color32,
    pub red: Color32,
}

pub fn palette(theme: ThemeChoice) -> Palette {
    match theme {
        ThemeChoice::Midnight => Palette {
            background: Color32::from_rgb(10, 15, 28),
            panel: Color32::from_rgb(17, 25, 43),
            raised: Color32::from_rgb(23, 33, 55),
            border: Color32::from_rgb(40, 55, 80),
            text: Color32::from_rgb(231, 239, 250),
            muted: Color32::from_rgb(139, 157, 181),
            cyan: Color32::from_rgb(67, 211, 231),
            blue: Color32::from_rgb(94, 150, 255),
            green: Color32::from_rgb(65, 211, 151),
            amber: Color32::from_rgb(255, 186, 82),
            violet: Color32::from_rgb(165, 135, 255),
            red: Color32::from_rgb(255, 113, 128),
        },
        ThemeChoice::Slate => Palette {
            background: Color32::from_rgb(22, 27, 34),
            panel: Color32::from_rgb(31, 38, 47),
            raised: Color32::from_rgb(41, 50, 62),
            border: Color32::from_rgb(70, 83, 100),
            text: Color32::from_rgb(240, 243, 247),
            muted: Color32::from_rgb(170, 183, 198),
            cyan: Color32::from_rgb(80, 218, 221),
            blue: Color32::from_rgb(121, 171, 255),
            green: Color32::from_rgb(82, 215, 152),
            amber: Color32::from_rgb(255, 198, 104),
            violet: Color32::from_rgb(185, 159, 255),
            red: Color32::from_rgb(255, 123, 135),
        },
        ThemeChoice::HighContrast => Palette {
            background: Color32::from_rgb(0, 0, 0),
            panel: Color32::from_rgb(12, 12, 12),
            raised: Color32::from_rgb(28, 28, 28),
            border: Color32::from_rgb(230, 230, 230),
            text: Color32::WHITE,
            muted: Color32::from_rgb(220, 220, 220),
            cyan: Color32::from_rgb(0, 255, 255),
            blue: Color32::from_rgb(100, 185, 255),
            green: Color32::from_rgb(0, 255, 100),
            amber: Color32::from_rgb(255, 230, 0),
            violet: Color32::from_rgb(255, 80, 240),
            red: Color32::from_rgb(255, 70, 70),
        },
    }
}

pub fn apply(ctx: &egui::Context, choice: ThemeChoice, font_scale: f32) {
    let colors = palette(choice);
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = colors.background;
    visuals.window_fill = colors.panel;
    visuals.window_corner_radius = egui::CornerRadius::same(14);
    visuals.window_stroke = Stroke::new(1.0_f32, colors.border);
    visuals.menu_corner_radius = egui::CornerRadius::same(10);
    visuals.override_text_color = Some(colors.text);
    visuals.widgets.noninteractive.bg_fill = colors.panel;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, colors.text);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.inactive.bg_fill = colors.raised;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, colors.border);
    visuals.widgets.hovered.bg_fill = colors.border;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, colors.cyan);
    visuals.widgets.active.bg_fill = colors.cyan.gamma_multiply(0.35);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, colors.cyan);
    visuals.selection.bg_fill = colors.cyan.gamma_multiply(0.32);
    visuals.selection.stroke = Stroke::new(1.0_f32, colors.cyan);
    visuals.hyperlink_color = colors.cyan;

    let mut style = egui::Style {
        visuals,
        ..Default::default()
    };
    ctx.set_zoom_factor(font_scale.clamp(0.8, 1.5));
    style.spacing.item_spacing = Vec2::new(9.0, 8.0);
    style.spacing.button_padding = Vec2::new(9.0, 6.0);
    style.animation_time = 0.14;
    ctx.set_style(style);
}
