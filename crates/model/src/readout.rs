//! Adaptador de lectura: controles → `fotoelectrico-physics` → `Readout`.
//!
//! Replica exactamente los supuestos documentados del proyecto original
//! (`docs/physics-model.md`): flujo = P/E, rampa de colección lineal,
//! fotocorriente I = e·Φ·A·QE·g(V). La física fundamental (Kmax, V₀, v)
//! sale tal cual del motor `fotoelectrico-physics`.

use fotoelectrico_physics::{constants, Material};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::constants::{CURVE_MAX_NM, CURVE_MIN_NM, CURVE_POINTS};
use crate::session::{ExperimentControls, MaterialChoice, SpeedScaleMode};

/// Punto de la curva Kmax(λ) para gráficas y caches.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export, export_to = "CurvePoint.ts")]
pub struct CurvePoint {
    pub wavelength_nm: f64,
    pub k_max_ev: f64,
}

/// Todas las magnitudes que la UI puede mostrar, ya en unidades de pantalla.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export, export_to = "Readout.ts")]
pub struct Readout {
    pub emits: bool,
    pub blocked: bool,
    pub frequency_hz: f64,
    pub photon_energy_ev: f64,
    pub photon_energy_j: f64,
    pub work_function_ev: f64,
    pub threshold_frequency_hz: f64,
    pub threshold_wavelength_nm: f64,
    pub k_max_ev: f64,
    pub k_max_j: f64,
    pub stopping_potential_v: f64,
    pub electron_max_speed_m_s: f64,
    /// Escala de animación 0..∞ según preferencia (comprimida o real-log).
    pub speed_scale: f64,
    pub photon_flux_per_m2_s: f64,
    pub photocurrent_a: f64,
    pub collection_factor: f64,
    /// Energía (eV) que aún le falta al fotón para superar Φ (0 si ya emite).
    pub deficit_ev: f64,
}

/// Respuesta completa de `evaluate`: lecturas del cátodo principal y del
/// material de comparación + curva cacheable por material.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export, export_to = "EvaluateResponse.ts")]
pub struct EvaluateResponse {
    pub readout: Readout,
    pub comparison: Readout,
    /// 61 puntos Kmax(λ) del material principal; la UI la cachea y solo
    /// cambia cuando cambia el material (ver `evaluate`).
    pub curve: Vec<CurvePoint>,
    pub comparison_curve: Vec<CurvePoint>,
    /// Material efectivo tras sanear (la UI puede mostrar Φ custom clampeada).
    pub effective_phi_ev: f64,
    pub comparison_effective_phi_ev: f64,
}

pub fn material_for(choice: MaterialChoice, custom_phi_ev: f64) -> Material {
    choice.to_physics(custom_phi_ev)
}

/// Flujo de fotones incidentes: `flujo = irradiancia / E_fotón`.
/// 100 % = 100 W/m². Garantía: jamás NaN/inf.
pub fn photon_flux_density_per_m2_s(intensity_percent: f64, photon_energy_j: f64) -> f64 {
    let intensity = if intensity_percent.is_finite() {
        intensity_percent.clamp(0.0, 100.0)
    } else {
        0.0
    };
    if !photon_energy_j.is_finite() || photon_energy_j <= 0.0 || intensity <= 0.1 {
        return 0.0;
    }
    let irradiance = intensity / 100.0 * crate::constants::REFERENCE_IRRADIANCE_W_M2;
    let flux = irradiance / photon_energy_j;
    if flux.is_finite() && flux >= 0.0 {
        flux
    } else {
        0.0
    }
}

/// ¿Llegan electrones al ánodo? Binario para la animación.
pub fn is_collected(
    emits: bool,
    stopping_potential_v: f64,
    applied_voltage_v: f64,
    intensity_percent: f64,
) -> bool {
    let intensity = intensity_percent.clamp(0.0, 100.0);
    let applied = if applied_voltage_v.is_finite() {
        applied_voltage_v.clamp(-30.0, 30.0)
    } else {
        0.0
    };
    let v0 = if stopping_potential_v.is_finite() && stopping_potential_v >= 0.0 {
        stopping_potential_v
    } else {
        0.0
    };
    emits && intensity > 0.1 && applied >= -v0 - 1e-9
}

/// Rampa de colección continua 0..1 (la curva I–V).
pub fn collection_factor(
    emits: bool,
    stopping_potential_v: f64,
    applied_voltage_v: f64,
    intensity_percent: f64,
) -> f64 {
    let intensity = intensity_percent.clamp(0.0, 100.0);
    let applied = if applied_voltage_v.is_finite() {
        applied_voltage_v.clamp(-30.0, 30.0)
    } else {
        0.0
    };
    let v0 = if stopping_potential_v.is_finite() && stopping_potential_v > 0.0 {
        stopping_potential_v
    } else {
        0.0
    };
    if !emits || intensity <= 0.1 {
        return 0.0;
    }
    if applied >= 0.0 {
        return 1.0;
    }
    if v0 <= 0.0 {
        return 0.0;
    }
    if applied <= -v0 {
        0.0
    } else {
        ((applied + v0) / v0).clamp(0.0, 1.0)
    }
}

/// Fotocorriente `I = e·Φ·A·QE·g(V)` en amperios.
pub fn photocurrent_a(
    photon_flux_per_m2_s: f64,
    emits: bool,
    stopping_potential_v: f64,
    applied_voltage_v: f64,
    intensity_percent: f64,
    cathode_area_cm2: f64,
    quantum_efficiency_percent: f64,
) -> f64 {
    if !photon_flux_per_m2_s.is_finite() || photon_flux_per_m2_s <= 0.0 {
        return 0.0;
    }
    let area_m2 = if cathode_area_cm2.is_finite() {
        cathode_area_cm2.clamp(0.05, 25.0)
    } else {
        1.0
    } * 1e-4;
    let qe = if quantum_efficiency_percent.is_finite() {
        quantum_efficiency_percent.clamp(0.0, 50.0)
    } else {
        1.0
    } / 100.0;
    let g = collection_factor(
        emits,
        stopping_potential_v,
        applied_voltage_v,
        intensity_percent,
    );
    let current = constants::ELEMENTARY_CHARGE_E * photon_flux_per_m2_s * area_m2 * qe * g;
    if current.is_finite() && current >= 0.0 {
        current
    } else {
        0.0
    }
}

/// Escala visual de velocidad con las dos leyes pedidas.
///
/// - `Compressed`: √(Kmax/1 eV) acotada a [0.5, 2.2] — comprime el contraste
///   Na (≈0.9) vs Pt (≈0.4 y sin emisión a λ largas). Documentada en leyenda.
/// - `RealLog`: log₁₀ de la velocidad real m/s mapeada a [0.5, 2.2] con la
///   leyenda "escala real logarítmica".
///
/// La animación SIEMPRE integra con dt (nunca por frame); esta escala solo
/// multiplica la velocidad nominal.
pub fn speed_scale_for(mode: SpeedScaleMode, k_max_ev: f64, speed_m_s: f64) -> f64 {
    let k = if k_max_ev.is_finite() && k_max_ev > 1e-9 {
        k_max_ev
    } else {
        return 0.5;
    };
    match mode {
        SpeedScaleMode::Compressed => (k / crate::constants::SPEED_SCALE_K_REF_EV).sqrt().clamp(
            crate::constants::SPEED_SCALE_MIN,
            crate::constants::SPEED_SCALE_MAX,
        ),
        SpeedScaleMode::RealLog => {
            let v = if speed_m_s.is_finite() && speed_m_s > 0.0 {
                speed_m_s
            } else {
                return 0.5;
            };
            // 1e5 m/s → 0.5; 1e7 m/s → 2.2 (log10 de 5 a 7).
            let t = ((v.log10() - 5.0) / 2.0).clamp(0.0, 1.0);
            0.5 + t * (crate::constants::SPEED_SCALE_MAX - crate::constants::SPEED_SCALE_MIN)
        }
    }
}

fn readout_for(
    material: &Material,
    controls: &ExperimentControls,
    lambda_nm: f64,
    speed_mode: SpeedScaleMode,
) -> Readout {
    let physics = fotoelectrico_physics::get_physics_readout(material, lambda_nm);
    let frequency_hz = constants::SPEED_OF_LIGHT_C / constants::nm_to_meters(lambda_nm);
    let flux = photon_flux_density_per_m2_s(controls.intensity_percent, physics.photon_energy_j);
    let factor = collection_factor(
        physics.emits,
        physics.stopping_potential_v,
        controls.applied_voltage_v,
        controls.intensity_percent,
    );
    let current = photocurrent_a(
        flux,
        physics.emits,
        physics.stopping_potential_v,
        controls.applied_voltage_v,
        controls.intensity_percent,
        controls.cathode_area_cm2,
        controls.quantum_efficiency_percent,
    );
    let blocked = physics.emits
        && controls.intensity_percent > 0.1
        && !is_collected(
            physics.emits,
            physics.stopping_potential_v,
            controls.applied_voltage_v,
            controls.intensity_percent,
        );
    let deficit = (physics.work_function_ev - physics.photon_energy_ev).max(0.0);
    Readout {
        emits: physics.emits,
        blocked,
        frequency_hz: finite(frequency_hz),
        photon_energy_ev: finite(physics.photon_energy_ev),
        photon_energy_j: finite(physics.photon_energy_j),
        work_function_ev: finite(physics.work_function_ev),
        threshold_frequency_hz: finite(physics.threshold_frequency_hz),
        threshold_wavelength_nm: finite(physics.threshold_wavelength_nm),
        k_max_ev: finite(physics.k_max_ev),
        k_max_j: finite(physics.k_max_j),
        stopping_potential_v: finite(physics.stopping_potential_v),
        electron_max_speed_m_s: finite(physics.electron_max_speed_m_s),
        speed_scale: speed_scale_for(speed_mode, physics.k_max_ev, physics.electron_max_speed_m_s),
        photon_flux_per_m2_s: finite(flux),
        photocurrent_a: finite(current),
        collection_factor: finite(factor),
        deficit_ev: finite(deficit),
    }
}

fn finite(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

/// Curva Kmax(λ) muestreada en `points` puntos entre 180 y 900 nm.
/// La UI la cachea y solo la recalcula cuando cambia el material.
pub fn curve_for(material: &Material, points: usize) -> Vec<CurvePoint> {
    let n = points.clamp(2, 241);
    let mut curve = Vec::with_capacity(n);
    for i in 0..n {
        let fraction = i as f64 / (n - 1) as f64;
        let lambda_nm = CURVE_MIN_NM + fraction * (CURVE_MAX_NM - CURVE_MIN_NM);
        let result = fotoelectrico_physics::photon::calculate_effect(material, lambda_nm);
        curve.push(CurvePoint {
            wavelength_nm: lambda_nm,
            k_max_ev: finite(constants::joules_to_ev(result.k_max_j)),
        });
    }
    curve
}

/// Punto principal de la API. `curve_points` permite 61 (UI) o más (export).
pub fn evaluate(controls: &ExperimentControls, curve_points: usize) -> EvaluateResponse {
    let clean = controls.sanitized();
    let material = material_for(clean.material, clean.custom_phi_ev);
    let comparison = material_for(clean.comparison_material, clean.comparison_custom_phi_ev);
    EvaluateResponse {
        readout: readout_for(
            &material,
            &clean,
            clean.wavelength_nm,
            SpeedScaleMode::Compressed,
        ),
        comparison: readout_for(
            &comparison,
            &clean,
            clean.wavelength_nm,
            SpeedScaleMode::Compressed,
        ),
        curve: curve_for(&material, curve_points),
        comparison_curve: Vec::new(),
        effective_phi_ev: material.work_function_ev(),
        comparison_effective_phi_ev: comparison.work_function_ev(),
    }
}

/// Variante con preferencia de escala de velocidad y curvas de ambos
/// materiales (la de comparación se genera bajo demanda).
pub fn evaluate_full(
    controls: &ExperimentControls,
    preferences: &crate::session::UiPreferences,
    curve_points: usize,
    with_comparison_curve: bool,
) -> EvaluateResponse {
    let clean = controls.sanitized();
    let material = material_for(clean.material, clean.custom_phi_ev);
    let comparison = material_for(clean.comparison_material, clean.comparison_custom_phi_ev);
    EvaluateResponse {
        readout: readout_for(
            &material,
            &clean,
            clean.wavelength_nm,
            preferences.speed_scale,
        ),
        comparison: readout_for(
            &comparison,
            &clean,
            clean.wavelength_nm,
            preferences.speed_scale,
        ),
        curve: curve_for(&material, curve_points),
        comparison_curve: if with_comparison_curve {
            curve_for(&comparison, curve_points)
        } else {
            Vec::new()
        },
        effective_phi_ev: material.work_function_ev(),
        comparison_effective_phi_ev: comparison.work_function_ev(),
    }
}

pub const DEFAULT_CURVE_POINTS: usize = CURVE_POINTS;

#[cfg(test)]
mod tests {
    use super::*;

    fn sodium() -> ExperimentControls {
        ExperimentControls {
            material: MaterialChoice::Sodium,
            wavelength_nm: 400.0,
            intensity_percent: 55.0,
            applied_voltage_v: 0.0,
            ..Default::default()
        }
    }

    #[test]
    fn reads_real_physics_values() {
        let r = evaluate(&sodium(), 61);
        assert!(r.readout.emits);
        // Sodio 400 nm → E ≈ 3.10 eV, Kmax ≈ 0.74 eV, V₀ ≈ 0.74 V.
        assert!((r.readout.photon_energy_ev - 3.10).abs() < 0.02);
        assert!((r.readout.k_max_ev - 0.74).abs() < 0.02);
        assert!((r.readout.stopping_potential_v - 0.74).abs() < 0.02);
        assert!((r.readout.threshold_wavelength_nm - 525.0).abs() < 1.0);
        assert_eq!(r.curve.len(), 61);
        assert!(r.readout.photocurrent_a > 0.0);
        assert!(r.readout.electron_max_speed_m_s > 5.0e5);
    }

    #[test]
    fn red_light_no_emission_but_flux_exists() {
        let mut c = sodium();
        c.wavelength_nm = 700.0;
        let r = evaluate(&c, 61);
        assert!(!r.readout.emits);
        assert_eq!(r.readout.k_max_ev, 0.0);
        assert_eq!(r.readout.photocurrent_a, 0.0);
        assert!(r.readout.photon_flux_per_m2_s > 0.0);
        assert!((r.readout.deficit_ev - (2.36 - 1.771)).abs() < 0.01);
    }

    #[test]
    fn blocking_keeps_kmax_and_zeroes_current() {
        let free = evaluate(&sodium(), 61);
        let mut c = sodium();
        c.applied_voltage_v = -2.0;
        let blocked = evaluate(&c, 61);
        assert!(blocked.readout.emits);
        assert!(blocked.readout.blocked);
        assert_eq!(blocked.readout.k_max_ev, free.readout.k_max_ev);
        assert_eq!(blocked.readout.photocurrent_a, 0.0);
        assert_eq!(blocked.readout.collection_factor, 0.0);
    }

    #[test]
    fn intensity_changes_current_not_energy() {
        let dim = evaluate(
            &ExperimentControls {
                intensity_percent: 10.0,
                ..sodium()
            },
            61,
        );
        let bright = evaluate(
            &ExperimentControls {
                intensity_percent: 90.0,
                ..sodium()
            },
            61,
        );
        assert_eq!(dim.readout.k_max_ev, bright.readout.k_max_ev);
        assert!(bright.readout.photocurrent_a > dim.readout.photocurrent_a * 8.0);
    }

    #[test]
    fn speed_scale_is_bounded_and_documented() {
        assert!((speed_scale_for(SpeedScaleMode::Compressed, 1.0, 0.0) - 1.0).abs() < 1e-12);
        assert_eq!(speed_scale_for(SpeedScaleMode::Compressed, 30.0, 0.0), 2.2);
        assert_eq!(speed_scale_for(SpeedScaleMode::Compressed, 0.0, 0.0), 0.5);
        let slow = speed_scale_for(SpeedScaleMode::RealLog, 0.1, 1.0e5);
        let fast = speed_scale_for(SpeedScaleMode::RealLog, 3.0, 1.0e7);
        assert!((slow - 0.5).abs() < 1e-9);
        assert!((fast - 2.2).abs() < 1e-9);
    }

    #[test]
    fn custom_material_changes_readout() {
        let mut c = sodium();
        c.material = MaterialChoice::Custom;
        c.custom_phi_ev = 5.0;
        let r = evaluate(&c, 61);
        assert!(!r.readout.emits, "Φ=5.0 eV > E(400nm)=3.1 eV");
        assert!((r.effective_phi_ev - 5.0).abs() < 1e-12);
    }

    #[test]
    fn never_nan_on_extreme_inputs() {
        for weird in [f64::NAN, f64::INFINITY, -50.0, 0.0, 1e9] {
            let c = ExperimentControls {
                wavelength_nm: weird,
                intensity_percent: weird,
                applied_voltage_v: weird,
                cathode_area_cm2: weird,
                quantum_efficiency_percent: weird,
                ..Default::default()
            };
            let r = evaluate(&c, 61);
            for v in [
                r.readout.frequency_hz,
                r.readout.photon_energy_ev,
                r.readout.k_max_ev,
                r.readout.stopping_potential_v,
                r.readout.photocurrent_a,
                r.readout.collection_factor,
                r.readout.speed_scale,
            ] {
                assert!(v.is_finite());
                assert!(v >= 0.0);
            }
        }
    }

    #[test]
    fn collection_ramp_is_linear() {
        assert!((collection_factor(true, 1.0, -0.5, 50.0) - 0.5).abs() < 1e-12);
        assert_eq!(collection_factor(true, 1.0, 0.0, 50.0), 1.0);
        assert_eq!(collection_factor(true, 1.0, -1.1, 50.0), 0.0);
    }
}
