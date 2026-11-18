// crates/physics/src/material.rs

use crate::constants::{ev_to_joules, PLANCK_H};

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    /// Función de trabajo en Joules (J)
    pub work_function_j: f64,
}

impl Material {
    /// Crea un nuevo material ingresando la función de trabajo en eV.
    /// Garantía: Φ inválida (NaN/inf/<=0) se satura a 2.36 eV (sodio) en vez de fallar.
    pub fn new(name: &str, work_function_ev: f64) -> Self {
        let clean_ev = if work_function_ev.is_finite() && work_function_ev > 0.05 {
            work_function_ev.clamp(0.1, 10.0)
        } else {
            2.36
        };
        Self {
            name: name.to_string(),
            work_function_j: ev_to_joules(clean_ev),
        }
    }

    /// Obtiene la función de trabajo en eV
    pub fn work_function_ev(&self) -> f64 {
        self.work_function_j / crate::constants::EV_TO_JOULES
    }

    /// Calcula la Frecuencia Umbral (f0 = Φ / h)
    pub fn threshold_frequency_hz(&self) -> f64 {
        self.work_function_j / PLANCK_H
    }

    /// Calcula la Longitud de Onda Umbral en metros (λ0 = c / f0)
    pub fn threshold_wavelength_m(&self) -> f64 {
        crate::constants::SPEED_OF_LIGHT_C / self.threshold_frequency_hz()
    }
}
