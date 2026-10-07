// crates/physics/src/photon.rs

use crate::constants::{nm_to_meters, PLANCK_H, SPEED_OF_LIGHT_C};
use crate::material::Material;

/// Estructura que representa el resultado del efecto fotoeléctrico
#[derive(Debug, Clone)]
pub struct PhotoelectricEffectResult {
    pub emits_electron: bool,
    pub photon_energy_j: f64,
    pub k_max_j: f64,
    pub stopping_potential_v: f64,
}

/// Límites físicos aceptados por el motor. Fuera de aquí se satura,
/// nunca se devuelve NaN/inf ni se "desconecta".
pub const MIN_WAVELENGTH_NM: f64 = 10.0;
pub const MAX_WAVELENGTH_NM: f64 = 10_000.0;
/// Diferencias de energía menores a esto se consideran umbral (Kmax = 0).
pub const EMISSION_EPS_J: f64 = 1e-25;

/// Sanea una longitud de onda: finita y dentro de rango. Nunca NaN/inf.
pub fn sanitize_wavelength_nm(wavelength_nm: f64) -> f64 {
    if !wavelength_nm.is_finite() {
        return 550.0;
    }
    wavelength_nm.clamp(MIN_WAVELENGTH_NM, MAX_WAVELENGTH_NM)
}

/// Calcula lo que ocurre cuando incide luz de cierta longitud de onda (en nm) sobre un material.
/// Garantía: jamás devuelve NaN/inf; con entradas inválidas satura a un valor seguro.
pub fn calculate_effect(material: &Material, wavelength_nm: f64) -> PhotoelectricEffectResult {
    let clean_nm = sanitize_wavelength_nm(wavelength_nm);
    let wavelength_m = nm_to_meters(clean_nm);

    // Frecuencia f = c / λ (wavelength_m > 0 siempre por saneamiento)
    let frequency_hz = SPEED_OF_LIGHT_C / wavelength_m;

    // Energía del fotón E = h * f
    let photon_energy_j = PLANCK_H * frequency_hz;
    let work_j = if material.work_function_j.is_finite() && material.work_function_j > 0.0 {
        material.work_function_j
    } else {
        // Material inválido: sin emisión pero con energía válida (no NaN)
        return PhotoelectricEffectResult {
            emits_electron: false,
            photon_energy_j: finite_or_zero(photon_energy_j),
            k_max_j: 0.0,
            stopping_potential_v: 0.0,
        };
    };

    // ¿Supera la función de trabajo? (E > Φ + eps para evitar parpadeo en el umbral)
    let diff_j = photon_energy_j - work_j;
    if diff_j > EMISSION_EPS_J {
        // K_max = hf - Φ
        let k_max_j = diff_j;

        // V0 = K_max / e
        let stopping_potential_v = k_max_j / crate::constants::ELEMENTARY_CHARGE_E;

        PhotoelectricEffectResult {
            emits_electron: true,
            photon_energy_j: finite_or_zero(photon_energy_j),
            k_max_j: finite_or_zero(k_max_j),
            stopping_potential_v: finite_or_zero(stopping_potential_v),
        }
    } else {
        // No hay emisión (f <= f0): Kmax y V0 son exactamente 0, nunca NaN
        PhotoelectricEffectResult {
            emits_electron: false,
            photon_energy_j: finite_or_zero(photon_energy_j),
            k_max_j: 0.0,
            stopping_potential_v: 0.0,
        }
    }
}

fn finite_or_zero(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

// --- SUITE COMPLETA DE PRUEBAS UNITARIAS (FASE 3 ROADMAP) ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{meters_to_nm, SPEED_OF_LIGHT_C};
    use crate::material::Material;

    #[test]
    fn test_no_emission_below_threshold() {
        let potassium = Material::new("Potasio", 2.29);
        // Luz roja (700 nm) -> frecuencia menor que f0
        let result = calculate_effect(&potassium, 700.0);

        assert!(!result.emits_electron);
        assert_eq!(result.k_max_j, 0.0);
        assert_eq!(result.stopping_potential_v, 0.0);
    }

    #[test]
    fn test_emission_above_threshold() {
        let potassium = Material::new("Potasio", 2.29);
        // UV (300 nm) -> frecuencia mayor que f0
        let result = calculate_effect(&potassium, 300.0);

        assert!(result.emits_electron);
        assert!(result.k_max_j > 0.0);
        assert!(result.stopping_potential_v > 0.0);
    }

    #[test]
    fn test_kmax_increases_with_frequency() {
        let sodium = Material::new("Sodio", 2.36);

        // 400 nm vs 300 nm (300 nm tiene mayor frecuencia)
        let res_low_freq = calculate_effect(&sodium, 400.0);
        let res_high_freq = calculate_effect(&sodium, 300.0);

        assert!(res_high_freq.k_max_j > res_low_freq.k_max_j);
        assert!(res_high_freq.stopping_potential_v > res_low_freq.stopping_potential_v);
    }

    #[test]
    fn test_exact_threshold_frequency() {
        let copper = Material::new("Cobre", 4.70);
        let f0 = copper.threshold_frequency_hz();

        // Convertimos f0 a longitud de onda en nm
        let lambda_m = SPEED_OF_LIGHT_C / f0;
        let lambda_nm = meters_to_nm(lambda_m);

        let result = calculate_effect(&copper, lambda_nm);

        // En el umbral K_max es prácticamente 0 (margen por precisión flotante)
        assert!(result.k_max_j < 1e-20);
    }

    #[test]
    fn test_wavelength_frequency_consistency() {
        let lambda_input = 500.0; // nm
        let lambda_m = crate::constants::nm_to_meters(lambda_input);
        let freq = SPEED_OF_LIGHT_C / lambda_m;
        let lambda_reconstructed = meters_to_nm(SPEED_OF_LIGHT_C / freq);

        assert!((lambda_input - lambda_reconstructed).abs() < 1e-9);
    }

    #[test]
    fn test_never_returns_nan_or_inf_on_extreme_inputs() {
        let material = Material::new("Sodio", 2.36);
        for weird in [
            0.0,
            -100.0,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            180.0,
            900.0,
            10_000.0,
        ] {
            let result = calculate_effect(&material, weird);
            assert!(
                result.photon_energy_j.is_finite(),
                "photon_energy finita con λ={weird}"
            );
            assert!(result.k_max_j.is_finite(), "k_max finita con λ={weird}");
            assert!(
                result.stopping_potential_v.is_finite(),
                "V0 finita con λ={weird}"
            );
            assert!(
                result.k_max_j >= 0.0 && result.stopping_potential_v >= 0.0,
                "Kmax/V0 no negativos con λ={weird}"
            );
            let readout = crate::get_physics_readout(&material, weird);
            assert!(readout.photon_energy_j.is_finite());
            assert!(readout.k_max_j.is_finite());
            assert!(readout.stopping_potential_v.is_finite());
            assert!(readout.electron_max_speed_m_s.is_finite());
            assert!(readout.electron_max_speed_m_s >= 0.0);
        }
    }

    #[test]
    fn test_invalid_material_never_breaks_engine() {
        // Φ inválida se satura a 2.36 eV: el motor sigue devolviendo valores
        // finitos y coherentes (a 400 nm sí hay emisión con Φ saturada).
        for weird_phi in [0.0, -5.0, f64::NAN, f64::INFINITY] {
            let material = Material::new("Raro", weird_phi);
            assert!((material.work_function_ev() - 2.36).abs() < 1e-12);
            let result = calculate_effect(&material, 400.0);
            assert!(result.photon_energy_j.is_finite());
            assert!(result.k_max_j.is_finite());
            assert!(result.stopping_potential_v.is_finite());
            assert!(result.k_max_j >= 0.0);
        }
    }

    #[test]
    fn test_all_app_materials_emit_at_180nm_and_none_at_900nm() {
        // 180 nm → E ≈ 6.89 eV > Pt 5.65: todos emiten. 900 nm → E ≈ 1.38 eV: ninguno.
        for phi in [2.29, 2.36, 2.87, 4.30, 4.70, 5.65] {
            let material = Material::new("M", phi);
            assert!(
                calculate_effect(&material, 180.0).emits_electron,
                "Φ={phi} debe emitir a 180 nm"
            );
            assert!(
                !calculate_effect(&material, 900.0).emits_electron,
                "Φ={phi} no debe emitir a 900 nm"
            );
        }
    }
}
