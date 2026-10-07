//! Flechas de polaridad dibujadas según el control visual, no cálculo de campo.

use macroquad::prelude::{draw_line, Color};

use crate::layers::cell::CellGeometry;
use crate::scene::SceneFrame;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowSegment {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
}

/// Segmentos de flecha compartidos por el dibujo, los hit-tests y el HUD.
pub fn arrow_segments(geometry: CellGeometry, visual_voltage_v: f32) -> Vec<ArrowSegment> {
    if visual_voltage_v.abs() <= 0.15 {
        return Vec::new();
    }
    let direction = visual_voltage_v.signum();
    let center_x = (geometry.cathode_x + geometry.anode_x) * 0.5;
    let mut segments = Vec::with_capacity(9);
    for fraction in [-0.52_f32, 0.0, 0.52] {
        let y = geometry.mid_y + (geometry.plate_bottom - geometry.mid_y) * fraction;
        let start_x = if direction > 0.0 {
            center_x - 22.0
        } else {
            center_x + 22.0
        };
        let end_x = start_x + direction * 42.0;
        segments.push(ArrowSegment {
            start_x,
            start_y: y,
            end_x,
            end_y: y,
        });
        let back = if direction > 0.0 { -7.0 } else { 7.0 };
        segments.push(ArrowSegment {
            start_x: end_x,
            start_y: y,
            end_x: end_x + back,
            end_y: y - 4.0,
        });
        segments.push(ArrowSegment {
            start_x: end_x,
            start_y: y,
            end_x: end_x + back,
            end_y: y + 4.0,
        });
    }
    segments
}

pub fn draw_field_arrows(frame: &SceneFrame<'_>, geometry: CellGeometry, color: Color) {
    if !frame.layers.arrows {
        return;
    }
    for segment in arrow_segments(geometry, frame.visual_voltage_v) {
        draw_line(
            segment.start_x,
            segment.start_y,
            segment.end_x,
            segment.end_y,
            1.5,
            color,
        );
    }
}

pub fn draw_field_highlight(frame: &SceneFrame<'_>, geometry: CellGeometry, color: Color) {
    if !frame.layers.arrows {
        return;
    }
    for segment in arrow_segments(geometry, frame.visual_voltage_v) {
        draw_line(
            segment.start_x,
            segment.start_y,
            segment.end_x,
            segment.end_y,
            5.0,
            Color::new(color.r, color.g, color.b, 0.45),
        );
    }
}
