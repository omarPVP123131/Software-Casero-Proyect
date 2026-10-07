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

/// Calcula lo que ocurre cuando incide luz de cierta longitud de onda (en nm) sobre un material
pub fn calculate_effect(material: &Material, wavelength_nm: f64) -> PhotoelectricEffectResult {
    let wavelength_m = nm_to_meters(wavelength_nm);
    
    // Frecuencia f = c / λ
    let frequency_hz = SPEED_OF_LIGHT_C / wavelength_m;
    
    // Energía del fotón E = h * f
    let photon_energy_j = PLANCK_H * frequency_hz;
    
    // ¿Supera la función de trabajo? (E > Φ)
    if photon_energy_j > material.work_function_j {
        // K_max = hf - Φ
        let k_max_j = photon_energy_j - material.work_function_j;
        
        // V0 = K_max / e
        let stopping_potential_v = k_max_j / crate::constants::ELEMENTARY_CHARGE_E;
        
        PhotoelectricEffectResult {
            emits_electron: true,
            photon_energy_j,
            k_max_j,
            stopping_potential_v,
        }
    } else {
        // No hay emisión (f < f0)
        PhotoelectricEffectResult {
            emits_electron: false,
            photon_energy_j,
            k_max_j: 0.0,
            stopping_potential_v: 0.0,
        }
    }
}

// --- SUITE COMPLETA DE PRUEBAS UNITARIAS (FASE 3 ROADMAP) ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Material;
    use crate::constants::{SPEED_OF_LIGHT_C, meters_to_nm};

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
}