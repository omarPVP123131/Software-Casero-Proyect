//! Cuadrícula, regla y rótulos de la escena.

use macroquad::prelude::{
    draw_line, draw_rectangle, draw_rectangle_lines, draw_text, measure_text, Color, Rect,
};

use crate::layers::cell::CellGeometry;
use crate::scene::{SceneFrame, Viewport};

pub fn draw_grid(viewport: Viewport, visible: bool, color: Color) {
    if !visible {
        return;
    }
    let step = 28.0;
    let mut x = viewport.x + 8.0;
    while x < viewport.x + viewport.width - 8.0 {
        draw_line(
            x,
            viewport.y + 8.0,
            x,
            viewport.y + viewport.height - 8.0,
            0.6,
            color,
        );
        x += step;
    }
    let mut y = viewport.y + 8.0;
    while y < viewport.y + viewport.height - 8.0 {
        draw_line(
            viewport.x + 8.0,
            y,
            viewport.x + viewport.width - 8.0,
            y,
            0.6,
            color,
        );
        y += step;
    }
}

pub fn draw_ruler(viewport: Viewport, visible: bool, color: Color, font_scale: f32) {
    if !visible {
        return;
    }
    let y = viewport.y + viewport.height - 15.0;
    let left = viewport.x + 20.0;
    let right = viewport.x + viewport.width - 20.0;
    draw_line(left, y, right, y, 1.0, color);
    for index in 0..=10 {
        let x = left + (right - left) * index as f32 / 10.0;
        let tick_height = if index % 5 == 0 { 7.0 } else { 4.0 };
        draw_line(x, y - tick_height, x, y + tick_height, 1.0, color);
        if index % 5 == 0 {
            draw_text(
                format!("{}%", index * 10),
                x - 9.0,
                y - 9.0,
                10.0 * font_scale,
                color,
            );
        }
    }
}

pub fn draw_labels(frame: &SceneFrame<'_>, geometry: CellGeometry, color: Color) {
    if !frame.layers.labels {
        return;
    }
    let viewport = frame.viewport;
    let font_size = 11.0 * frame.font_scale.clamp(0.8, 1.5);
    let mut occupied = Vec::new();
    draw_callout(
        viewport,
        format!("λ · {:.0} nm", frame.wavelength_nm),
        (viewport.x + 92.0, viewport.y + 27.0),
        font_size,
        color,
        frame.high_contrast,
        &mut occupied,
    );
    draw_callout(
        viewport,
        format!("CÁTODO · {}", frame.material_label),
        (geometry.cathode_x, geometry.plate_top + 5.0),
        font_size,
        color,
        frame.high_contrast,
        &mut occupied,
    );
    draw_callout(
        viewport,
        "ÁNODO".to_owned(),
        (geometry.anode_x, geometry.plate_top + 5.0),
        font_size,
        color,
        frame.high_contrast,
        &mut occupied,
    );
    let status = match (frame.emission_possible, frame.collection_possible) {
        (Some(true), Some(true)) => "MOTOR · EMISIÓN + COLECCIÓN",
        (Some(true), _) => "MOTOR · EMISIÓN BLOQUEADA (V < -V0)",
        (Some(false), _) => "MOTOR · SIN EMISIÓN (λ > λ0)",
        (None, _) if frame.demo_electrons && frame.layers.electrons => {
            "DEMO · PARTÍCULAS ILUSTRATIVAS"
        }
        (None, _) => "MOTOR · CALCULANDO…",
    };
    draw_callout(
        viewport,
        status.to_owned(),
        (viewport.x + 112.0, viewport.y + viewport.height - 24.0),
        font_size * 0.9,
        color,
        frame.high_contrast,
        &mut occupied,
    );
    if frame.visual_voltage_v.abs() > 0.15 {
        draw_callout(
            viewport,
            format!("V APLICADO · {:+.1} V", frame.visual_voltage_v),
            (
                (geometry.cathode_x + geometry.anode_x) * 0.5,
                geometry.mid_y - 26.0,
            ),
            font_size * 0.9,
            color,
            frame.high_contrast,
            &mut occupied,
        );
    }
}

fn draw_callout(
    viewport: Viewport,
    text: String,
    anchor: (f32, f32),
    font_size: f32,
    text_color: Color,
    high_contrast: bool,
    occupied: &mut Vec<Rect>,
) {
    let dimensions = measure_text(&text, None, font_size.round() as u16, 1.0);
    let width = (dimensions.width + 18.0).min((viewport.width - 16.0).max(90.0));
    let height = font_size + 12.0;
    let (ax, ay) = anchor;
    let raw_candidates = [
        Rect::new(ax - width * 0.5, ay - height - 12.0, width, height),
        Rect::new(ax + 14.0, ay - height * 0.5, width, height),
        Rect::new(ax - width * 0.5, ay + 12.0, width, height),
        Rect::new(ax - width - 14.0, ay - height * 0.5, width, height),
    ];
    let min_x = viewport.x + 8.0;
    let max_x = (viewport.x + viewport.width - width - 8.0).max(min_x);
    let min_y = viewport.y + 8.0;
    let max_y = (viewport.y + viewport.height - height - 8.0).max(min_y);
    let mut best = raw_candidates[0];
    let mut best_score = f32::INFINITY;
    for candidate in raw_candidates {
        let clamped = Rect::new(
            candidate.x.clamp(min_x, max_x),
            candidate.y.clamp(min_y, max_y),
            width,
            height,
        );
        let clamp_distance = (candidate.x - clamped.x).abs() + (candidate.y - clamped.y).abs();
        let center_x = clamped.x + width * 0.5;
        let center_y = clamped.y + height * 0.5;
        let anchor_distance = (center_x - ax).abs() + (center_y - ay).abs();
        let overlap = occupied
            .iter()
            .map(|other| intersection_area(clamped, *other))
            .sum::<f32>();
        let score = overlap * 100.0 + clamp_distance * 4.0 + anchor_distance;
        if score < best_score {
            best_score = score;
            best = clamped;
        }
    }

    let target_x = ax.clamp(best.x, best.x + best.w);
    let target_y = ay.clamp(best.y, best.y + best.h);
    let leader = if high_contrast {
        Color::from_rgba(255, 255, 255, 190)
    } else {
        Color::new(text_color.r, text_color.g, text_color.b, 0.48)
    };
    draw_line(ax, ay, target_x, target_y, 1.0, leader);
    let fill = if high_contrast {
        Color::from_rgba(0, 0, 0, 240)
    } else {
        Color::from_rgba(14, 23, 39, 232)
    };
    let border = if high_contrast {
        Color::from_rgba(245, 245, 245, 235)
    } else {
        Color::from_rgba(65, 93, 125, 220)
    };
    draw_rectangle(best.x, best.y, best.w, best.h, fill);
    draw_rectangle_lines(best.x, best.y, best.w, best.h, 1.0, border);
    draw_text(
        text,
        best.x + 9.0,
        best.y + best.h - 5.0,
        font_size,
        text_color,
    );
    occupied.push(best);
}

fn intersection_area(left: Rect, right: Rect) -> f32 {
    let width = (left.x + left.w).min(right.x + right.w) - left.x.max(right.x);
    let height = (left.y + left.h).min(right.y + right.h) - left.y.max(right.y);
    width.max(0.0) * height.max(0.0)
}
