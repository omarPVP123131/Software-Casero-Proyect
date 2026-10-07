//! Núcleo físico del efecto fotoeléctrico (modelo ideal en SI).
//!
//! Garantías: ninguna función devuelve NaN/inf; las entradas inválidas se
//! sanean a valores seguros y el umbral usa tolerancia anti-parpadeo.

// crates/physics/src/lib.rs

pub mod constants;
pub mod fit;
pub mod material;
pub mod photon;

pub use fit::{fit_planck_constant, DataPoint, LinearFitResult};
pub use material::Material;
pub use photon::{calculate_effect, PhotoelectricEffectResult};

/// Estructura de la API que expone todos los datos leíbles para el Frontend/UI
#[derive(Debug, Clone)]
pub struct PhysicsReadout {
    // Estado de emisión
    pub emits: bool,

    // Energía del fotón
    pub photon_energy_ev: f64,
    pub photon_energy_j: f64,

    // Trabajo de extracción (Material)
    pub work_function_ev: f64,
    pub threshold_frequency_hz: f64,
    pub threshold_wavelength_nm: f64,

    // Fotoelectrón emitido
    pub k_max_ev: f64,
    pub k_max_j: f64,
    pub stopping_potential_v: f64,
    pub electron_max_speed_m_s: f64, // Velocidad máxima v = sqrt(2 * K_max / m_e)
}

/// Función principal de la API para consultar el estado físico actual.
/// Garantía: jamás devuelve NaN/inf; las entradas inválidas se sanean.
pub fn get_physics_readout(material: &Material, wavelength_nm: f64) -> PhysicsReadout {
    let result = calculate_effect(material, wavelength_nm);

    // v = sqrt(2 * K_max / m_e) — solo si hay emisión y K válido
    let electron_max_speed =
        if result.emits_electron && result.k_max_j.is_finite() && result.k_max_j > 0.0 {
            ((2.0 * result.k_max_j) / constants::ELECTRON_MASS_M).sqrt()
        } else {
            0.0
        };

    PhysicsReadout {
        emits: result.emits_electron,
        photon_energy_ev: finite_or_zero(constants::joules_to_ev(result.photon_energy_j)),
        photon_energy_j: finite_or_zero(result.photon_energy_j),
        work_function_ev: finite_or_zero(material.work_function_ev()),
        threshold_frequency_hz: finite_or_zero(material.threshold_frequency_hz()),
        threshold_wavelength_nm: finite_or_zero(constants::meters_to_nm(
            material.threshold_wavelength_m(),
        )),
        k_max_ev: finite_or_zero(constants::joules_to_ev(result.k_max_j)),
        k_max_j: finite_or_zero(result.k_max_j),
        stopping_potential_v: finite_or_zero(result.stopping_potential_v),
        electron_max_speed_m_s: finite_or_zero(electron_max_speed),
    }
}

fn finite_or_zero(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}
