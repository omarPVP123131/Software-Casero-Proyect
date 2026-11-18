//! Catálogo de cátodos: Φ, f₀, λ₀ calculados por `fotoelectrico-physics`.
//!
//! La UI pinta tarjetas por material con el mini-espectro y la marca del umbral;
//! todos los números vienen de aquí (nunca se escriben a mano en TS).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::session::MaterialChoice;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export, export_to = "MaterialInfo.ts")]
pub struct MaterialInfo {
    pub id: String,
    pub name_es: String,
    pub name_en: String,
    pub symbol: String,
    pub work_function_ev: f64,
    pub threshold_frequency_hz: f64,
    pub threshold_wavelength_nm: f64,
    /// Nota breve de la fuente del valor de Φ (tooltip).
    pub source: String,
}

/// Todos los materiales seleccionables, en el orden de la UI.
pub fn material_catalog() -> Vec<MaterialInfo> {
    MaterialChoice::ALL
        .iter()
        .filter(|m| **m != MaterialChoice::Custom)
        .map(|&choice| {
            let material = choice.to_physics(0.0);
            MaterialInfo {
                id: choice.id().to_owned(),
                name_es: choice.name_es().to_owned(),
                name_en: choice.name_en().to_owned(),
                symbol: choice.symbol().to_owned(),
                work_function_ev: material.work_function_ev(),
                threshold_frequency_hz: material.threshold_frequency_hz(),
                threshold_wavelength_nm: fotoelectrico_physics::constants::meters_to_nm(
                    material.threshold_wavelength_m(),
                ),
                source: choice.source().to_owned(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_matches_physics_engine() {
        let catalog = material_catalog();
        assert_eq!(catalog.len(), 6);
        let sodium = catalog.iter().find(|m| m.id == "sodium").unwrap();
        assert!((sodium.work_function_ev - 2.36).abs() < 1e-12);
        // λ₀(Na) ≈ 525 nm (h·c/Φ).
        assert!((sodium.threshold_wavelength_nm - 525.0).abs() < 1.0);
        // Todos los umbrales caen en el rango del slider.
        for m in &catalog {
            assert!(m.threshold_wavelength_nm > 180.0 && m.threshold_wavelength_nm < 900.0);
            assert!(m.threshold_frequency_hz > 0.0);
        }
    }
}
