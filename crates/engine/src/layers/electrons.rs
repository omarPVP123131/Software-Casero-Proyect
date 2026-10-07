//! Fotoelectrones guiados por el motor físico.
//!
//! La cantidad escala con la intensidad y la colección (bloqueo por
//! frenado); la velocidad visual escala con sqrt(Kmax). Sin colección
//! no se dibuja ningún electrón.

use macroquad::prelude::{draw_circle, draw_line, Color};

use crate::layers::cell::CellGeometry;
use crate::scene::{RenderQuality, SceneFrame};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElectronVisual {
    pub x: f32,
    pub y: f32,
    pub trail_y: f32,
}

/// Escala visual de velocidad a partir de Kmax (v ∝ sqrt(K), Kref = 1 eV).
/// Garantía: NaN/inf/<=0 → 1.0 (nunca falla ni se congela).
pub fn electron_speed_scale(k_max_ev: Option<f64>) -> f32 {
    match k_max_ev {
        Some(k) if k.is_finite() && k > 1e-9 => (k / 1.0).sqrt().clamp(0.5, 2.2) as f32,
        _ => 1.0,
    }
}

/// ¿Deben verse electrones? Colección (no bloqueados) + intensidad.
/// `None` conserva el fallback demo para arranque sin lectura.
/// Garantía: intensidad inválida → sin electrones, sin pánico.
pub fn electrons_visible(
    collection_possible: Option<bool>,
    emission_possible: Option<bool>,
    intensity_percent: f32,
    demo_electrons: bool,
) -> bool {
    let intensity = if intensity_percent.is_finite() {
        intensity_percent
    } else {
        0.0
    };
    if let Some(collected) = collection_possible {
        return collected && intensity > 0.1;
    }
    if let Some(emission) = emission_possible {
        return emission && intensity > 0.1;
    }
    demo_electrons
}

/// Cantidad de electrones: escala con intensidad cuando hay colección.
/// Garantía: siempre 0..=8, jamás 0 por error numérico cuando debe verse.
pub fn electron_count(intensity_percent: f32, visible: bool, quality: RenderQuality) -> usize {
    let intensity = if intensity_percent.is_finite() {
        intensity_percent.clamp(0.0, 100.0)
    } else {
        0.0
    };
    if !visible || intensity <= 0.1 {
        return 0;
    }
    let strength = intensity / 100.0;
    match quality {
        RenderQuality::Balanced => (2.0 + strength * 5.0).round().clamp(1.0, 7.0) as usize,
        RenderQuality::Performance => (1.0 + strength * 3.0).round().clamp(1.0, 4.0) as usize,
    }
}

/// Posiciones de partículas. UI y renderer comparten este muestreo para que
/// la selección coincida con lo dibujado.
#[allow(clippy::too_many_arguments)]
pub fn visual_electrons(
    geometry: CellGeometry,
    time_seconds: f64,
    animation_running: bool,
    animation_speed: f32,
    quality: RenderQuality,
    speed_scale: f32,
    count: usize,
) -> Vec<ElectronVisual> {
    let _ = quality;
    // Saneamiento total: geometría degenerada o tiempo inválido jamás rompen el frame.
    // Si la geometría no es finita, no hay dónde dibujar: 0 partículas (sin pánico).
    if !geometry.cathode_x.is_finite()
        || !geometry.anode_x.is_finite()
        || !geometry.mid_y.is_finite()
    {
        return Vec::new();
    }
    let count = count.min(12);
    if count == 0 {
        return Vec::new();
    }
    let span = (geometry.anode_x - geometry.cathode_x - 30.0).max(1.0);
    let time = if time_seconds.is_finite() {
        time_seconds as f32
    } else {
        0.0
    };
    let anim = if animation_speed.is_finite() {
        animation_speed.clamp(0.1, 3.0)
    } else {
        1.0
    };
    let speed = if speed_scale.is_finite() {
        speed_scale.clamp(0.4, 2.5)
    } else {
        1.0
    };
    (0..count)
        .map(|index| {
            let offset = index as f32 * 0.23;
            let effective_speed = anim * speed;
            let phase = if animation_running {
                (time * effective_speed * 0.21 + offset).rem_euclid(1.0)
            } else {
                0.48
            };
            let x = geometry.cathode_x + 15.0 + phase * span;
            let lane = (index as f32 - (count as f32 - 1.0) * 0.5) * 15.0;
            let y = geometry.mid_y + lane + (phase * std::f32::consts::TAU).sin() * 4.0;
            ElectronVisual {
                x,
                y,
                trail_y: geometry.mid_y + lane,
            }
        })
        .collect()
}

pub fn draw_electrons(frame: &SceneFrame<'_>, geometry: CellGeometry, color: Color) -> usize {
    if !frame.layers.electrons {
        return 0;
    }
    let visible = electrons_visible(
        frame.collection_possible,
        frame.emission_possible,
        frame.intensity_percent,
        frame.demo_electrons,
    );
    if !visible {
        return 0;
    }

    let speed = electron_speed_scale(frame.k_max_ev);
    let count = electron_count(frame.intensity_percent, true, frame.quality);
    let particles = visual_electrons(
        geometry,
        frame.time_seconds,
        frame.animation_running,
        frame.animation_speed,
        frame.quality,
        speed,
        count,
    );
    for particle in &particles {
        if frame.layers.trails && frame.quality == RenderQuality::Balanced {
            draw_line(
                geometry.cathode_x + 12.0,
                particle.trail_y,
                particle.x,
                particle.y,
                1.1,
                with_alpha(color, 75),
            );
        }
        draw_circle(particle.x, particle.y, 7.0, with_alpha(color, 35));
        draw_circle(particle.x, particle.y, 4.0, color);
    }
    particles.len()
}

fn with_alpha(color: Color, alpha: u8) -> Color {
    Color::new(color.r, color.g, color.b, alpha as f32 / 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_collection_draws_no_electrons() {
        assert!(!electrons_visible(Some(false), Some(true), 80.0, true));
        assert_eq!(electron_count(80.0, false, RenderQuality::Balanced), 0);
    }

    #[test]
    fn count_grows_with_intensity_when_collected() {
        let low = electron_count(15.0, true, RenderQuality::Balanced);
        let high = electron_count(90.0, true, RenderQuality::Balanced);
        assert!(high > low);
    }

    #[test]
    fn speed_grows_with_kmax() {
        assert!(electron_speed_scale(Some(2.0)) > electron_speed_scale(Some(0.5)));
    }

    #[test]
    fn engine_never_panics_on_garbage_inputs() {
        use crate::layers::cell::CellGeometry;
        // Geometría degenerada → 0 partículas en vez de posiciones NaN/inf.
        let bad_geometry = CellGeometry {
            beam_start_x: 0.0,
            cathode_x: 100.0,
            anode_x: 100.0, // vano degenerado
            plate_top: 0.0,
            plate_bottom: 0.0,
            plate_width: f32::NAN,
            mid_y: f32::INFINITY,
        };
        let none = visual_electrons(
            bad_geometry,
            f64::NAN,
            true,
            f32::INFINITY,
            RenderQuality::Balanced,
            f32::NAN,
            5,
        );
        assert!(none.is_empty());
        // Geometría sana + tiempo/velocidad basura → posiciones finitas.
        let good_geometry = CellGeometry {
            beam_start_x: 0.0,
            cathode_x: 100.0,
            anode_x: 300.0,
            plate_top: 0.0,
            plate_bottom: 200.0,
            plate_width: 12.0,
            mid_y: 100.0,
        };
        let particles = visual_electrons(
            good_geometry,
            f64::NAN,
            true,
            f32::INFINITY,
            RenderQuality::Balanced,
            f32::NAN,
            5,
        );
        assert_eq!(particles.len(), 5);
        for p in &particles {
            assert!(p.x.is_finite() && p.y.is_finite());
        }
        assert_eq!(electron_count(f32::NAN, true, RenderQuality::Balanced), 0);
        assert!(!electrons_visible(Some(true), Some(true), f32::NAN, true));
        assert_eq!(electron_speed_scale(Some(f64::NAN)), 1.0);
        assert_eq!(electron_speed_scale(Some(f64::INFINITY)), 1.0);
    }
}
