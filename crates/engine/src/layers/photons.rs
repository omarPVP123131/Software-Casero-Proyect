//! Haz y puntos de fotones, con movimiento puramente ilustrativo.

use macroquad::prelude::{draw_circle, draw_line, Color};

use crate::layers::cell::CellGeometry;
use crate::scene::{RenderQuality, SceneFrame};

pub fn spectrum_color(wavelength_nm: f32, high_contrast: bool) -> Color {
    // Saneado: NaN/inf → verde visible; el haz nunca desaparece por λ.
    let lambda = if wavelength_nm.is_finite() {
        wavelength_nm.clamp(180.0, 900.0)
    } else {
        550.0
    };
    if high_contrast {
        return if lambda < 450.0 {
            Color::from_rgba(255, 240, 0, 255)
        } else {
            Color::from_rgba(0, 255, 255, 255)
        };
    }
    match lambda {
        value if value < 380.0 => Color::from_rgba(174, 112, 255, 255),
        value if value < 450.0 => Color::from_rgba(112, 143, 255, 255),
        value if value < 510.0 => Color::from_rgba(69, 198, 235, 255),
        value if value < 570.0 => Color::from_rgba(81, 220, 163, 255),
        value if value < 610.0 => Color::from_rgba(255, 207, 92, 255),
        value if value < 700.0 => Color::from_rgba(255, 145, 87, 255),
        _ => Color::from_rgba(255, 93, 108, 255),
    }
}

pub fn beam_x_bounds(geometry: CellGeometry) -> (f32, f32) {
    (geometry.beam_start_x, geometry.cathode_x - 13.0)
}

pub fn beam_lane_y(geometry: CellGeometry, lane: usize) -> f32 {
    let fraction = lane as f32 / 4.0;
    geometry.plate_top + 18.0 + fraction * (geometry.plate_bottom - geometry.plate_top - 36.0)
}

/// El haz de fotones depende SOLO de capa activa + intensidad.
/// La longitud de onda cambia el color, jamás la visibilidad:
/// con luz tenue pero presente el haz siempre se dibuja.
fn photon_beam_visible(layers_photons: bool, intensity_percent: f32) -> bool {
    if !layers_photons {
        return false;
    }
    let intensity = if intensity_percent.is_finite() {
        intensity_percent
    } else {
        0.0
    };
    intensity > 0.1
}

fn sanitize_intensity(intensity_percent: f32) -> f32 {
    if intensity_percent.is_finite() {
        intensity_percent.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

pub fn draw_beam_highlight(frame: &SceneFrame<'_>, geometry: CellGeometry, color: Color) {
    if !photon_beam_visible(frame.layers.photons, frame.intensity_percent) {
        return;
    }
    let (start_x, end_x) = beam_x_bounds(geometry);
    for lane in 0..5 {
        let y = beam_lane_y(geometry, lane);
        draw_line(start_x, y, end_x, y, 5.0, with_alpha(color, 88));
    }
}

pub fn draw_photons(frame: &SceneFrame<'_>, geometry: CellGeometry, palette: Color) -> usize {
    if !photon_beam_visible(frame.layers.photons, frame.intensity_percent) {
        return 0;
    }
    let intensity = sanitize_intensity(frame.intensity_percent);

    let cap = match frame.quality {
        RenderQuality::Balanced => 10,
        RenderQuality::Performance => 4,
    };
    let count = (2.0 + intensity / 12.0).round().clamp(1.0, cap as f32) as usize;
    let (beam_start_x, beam_end_x) = beam_x_bounds(geometry);
    let color = palette;

    for lane in 0..5 {
        let y = beam_lane_y(geometry, lane);
        let alpha = (35.0 + intensity * 0.8).clamp(0.0, 255.0) as u8;
        draw_line(
            beam_start_x,
            y,
            beam_end_x,
            y,
            1.0,
            with_alpha(color, alpha),
        );

        for index in 0..count {
            let phase_offset = index as f32 / count as f32 + lane as f32 * 0.19;
            let time = if frame.time_seconds.is_finite() {
                frame.time_seconds as f32
            } else {
                0.0
            };
            let speed = if frame.animation_speed.is_finite() {
                frame.animation_speed.clamp(0.1, 3.0)
            } else {
                1.0
            };
            let phase = if frame.animation_running {
                (time * speed * 0.32 + phase_offset).rem_euclid(1.0)
            } else {
                phase_offset.rem_euclid(1.0)
            };
            let x = beam_start_x + phase * (beam_end_x - beam_start_x);
            let radius = if frame.quality == RenderQuality::Balanced {
                3.5
            } else {
                2.7
            };
            if frame.layers.trails && frame.quality == RenderQuality::Balanced {
                draw_line(
                    (x - 18.0).max(beam_start_x),
                    y,
                    x,
                    y,
                    1.2,
                    with_alpha(color, 82),
                );
            }
            draw_circle(x, y, radius + 2.6, with_alpha(color, 35));
            draw_circle(x, y, radius, color);
        }
    }

    count * 5
}

fn with_alpha(color: Color, alpha: u8) -> Color {
    Color::new(color.r, color.g, color.b, alpha as f32 / 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beam_visible_depends_only_on_layer_and_intensity() {
        // λ alta o voltaje invertido jamás ocultan el haz: solo la intensidad manda.
        assert!(photon_beam_visible(true, 80.0));
        assert!(photon_beam_visible(true, 100.0));
        assert!(!photon_beam_visible(true, 0.0));
        assert!(!photon_beam_visible(true, f32::NAN));
        assert!(!photon_beam_visible(false, 80.0));
    }

    #[test]
    fn spectrum_color_never_panics_and_covers_bands() {
        for lambda in [180.0, 380.0, 500.0, 700.0, 900.0, f32::NAN, f32::INFINITY] {
            let _ = spectrum_color(lambda, false);
            let _ = spectrum_color(lambda, true);
        }
    }
}
