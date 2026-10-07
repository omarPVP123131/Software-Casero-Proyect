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

/// Semilla de la variabilidad visual derivada de λ: cada longitud de onda
/// reordena el patrón de llegada. Determinista y compartida con la UI.
pub fn arrival_seed(wavelength_nm: f32) -> u64 {
    let bits = if wavelength_nm.is_finite() {
        wavelength_nm.clamp(180.0, 900.0).to_bits() as u64
    } else {
        550.0_f32.to_bits() as u64
    };
    bits.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(0x8BAD_F00D_C0FF_EE00)
}

fn hash01(mut z: u64) -> f64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    // 53 bits de precisión en [0, 1).
    ((z >> 11) as f64) / ((1u64 << 53) as f64)
}

/// Fluctuación del conteo: el número visible oscila ±1 alrededor de la media
/// (Poisson visual). Cuantizada cada 0.5 s; con tiempo congelado no cambia.
/// Garantía: base 0 siempre devuelve 0; el resultado nunca supera 12.
pub fn stochastic_count(base: usize, time_seconds: f64, seed: u64) -> usize {
    if base == 0 {
        return 0;
    }
    let t = if time_seconds.is_finite() && time_seconds >= 0.0 {
        time_seconds
    } else {
        0.0
    };
    let quantum = (t * 2.0).floor() as u64;
    let noise = hash01(seed.wrapping_add(quantum.wrapping_mul(0xC2B2_AE3D_27D4_EB4F)));
    let delta = ((noise * 3.0).floor() as i32) - 1; // -1, 0 o +1
    (base as i32 + delta).clamp(0, 12) as usize
}

/// Desfase propio de cada partícula en ±0.06 de fase: rompe la rejilla
/// perfectamente uniforme. Estático por (índice, semilla), sin saltos.
pub fn phase_jitter(index: usize, seed: u64) -> f32 {
    let h = hash01(seed ^ (index as u64).wrapping_mul(0x1656_67B1_9E37_79B9));
    (h as f32 - 0.5) * 0.12
}
/// Posiciones de partículas con llegada estocástica (Poisson visual).
///
/// UI y renderer comparten este muestreo para que la selección coincida con
/// lo dibujado: el conteo fluctúa ±1 dos veces por segundo y cada partícula
/// lleva un desfase propio. Todo es determinista en `(tiempo, semilla)`;
/// en pausa el tiempo se congela y la escena queda fija.
#[allow(clippy::too_many_arguments)]
pub fn visual_electrons(
    geometry: CellGeometry,
    time_seconds: f64,
    animation_running: bool,
    animation_speed: f32,
    quality: RenderQuality,
    speed_scale: f32,
    count: usize,
    seed: u64,
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
    let count = stochastic_count(count.min(12), time_seconds, seed);
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
            let offset = index as f32 * 0.23 + phase_jitter(index, seed);
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
        arrival_seed(frame.wavelength_nm),
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
    fn stochastic_arrival_is_deterministic_and_bounded() {
        // Misma (base, tiempo, semilla) → mismo resultado, siempre.
        let a = stochastic_count(5, 12.3, 99);
        let b = stochastic_count(5, 12.3, 99);
        assert_eq!(a, b);
        assert!((4..=6).contains(&a), "fluctúa ±1 alrededor de 5");
        // Base 0 jamás produce electrones de la nada.
        assert_eq!(stochastic_count(0, 12.3, 99), 0);
        // El desfase es estable por partícula y acotado.
        let j1 = phase_jitter(2, 99);
        assert_eq!(j1, phase_jitter(2, 99));
        assert!(j1.abs() <= 0.06 + f32::EPSILON);
        // Distintas λ dan distintas semillas.
        assert_ne!(arrival_seed(300.0), arrival_seed(700.0));
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
            0,
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
            0,
        );
        // Conteo estocástico: 5 ± 1, siempre finito.
        assert!((4..=6).contains(&particles.len()));
        for p in &particles {
            assert!(p.x.is_finite() && p.y.is_finite());
        }
        assert_eq!(electron_count(f32::NAN, true, RenderQuality::Balanced), 0);
        assert!(!electrons_visible(Some(true), Some(true), f32::NAN, true));
        assert_eq!(electron_speed_scale(Some(f64::NAN)), 1.0);
        assert_eq!(electron_speed_scale(Some(f64::INFINITY)), 1.0);
    }
}
