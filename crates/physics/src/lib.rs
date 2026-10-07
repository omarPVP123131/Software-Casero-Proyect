//! Esqueleto del núcleo físico del proyecto.
//!
//! La implementación de las ecuaciones corresponde al equipo de física.


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

/// Función principal de la API para consultar el estado físico actual
pub fn get_physics_readout(material: &Material, wavelength_nm: f64) -> PhysicsReadout {
    let result = calculate_effect(material, wavelength_nm);
    
    // v = sqrt(2 * K_max / m_e)
    let electron_max_speed = if result.emits_electron {
        ((2.0 * result.k_max_j) / constants::ELECTRON_MASS_M).sqrt()
    } else {
        0.0
    };

    PhysicsReadout {
        emits: result.emits_electron,
        photon_energy_ev: constants::joules_to_ev(result.photon_energy_j),
        photon_energy_j: result.photon_energy_j,
        work_function_ev: material.work_function_ev(),
        threshold_frequency_hz: material.threshold_frequency_hz(),
        threshold_wavelength_nm: constants::meters_to_nm(material.threshold_wavelength_m()),
        k_max_ev: constants::joules_to_ev(result.k_max_j),
        k_max_j: result.k_max_j,
        stopping_potential_v: result.stopping_potential_v,
        electron_max_speed_m_s: electron_max_speed,
    }
}