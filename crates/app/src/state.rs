//! Estado y contrato de presentación de la interfaz.
//!
//! Este módulo no calcula física. Los controles son entradas de UI y
//! `PhysicsReadout` es el espacio para recibir resultados de la crate física.

/// Opciones de material para la lista gráfica. No contiene parámetros físicos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialChoice {
    Potassium,
    Sodium,
    Calcium,
    Zinc,
    Copper,
    Platinum,
}

impl MaterialChoice {
    pub const ALL: [Self; 6] = [
        Self::Potassium,
        Self::Sodium,
        Self::Calcium,
        Self::Zinc,
        Self::Copper,
        Self::Platinum,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Potassium => "Potasio",
            Self::Sodium => "Sodio",
            Self::Calcium => "Calcio",
            Self::Zinc => "Zinc",
            Self::Copper => "Cobre",
            Self::Platinum => "Platino",
        }
    }

    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Potassium => "K",
            Self::Sodium => "Na",
            Self::Calcium => "Ca",
            Self::Zinc => "Zn",
            Self::Copper => "Cu",
            Self::Platinum => "Pt",
        }
    }
}

/// Controles que la capa gráfica expone para que la capa física los consuma.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabControls {
    pub material: MaterialChoice,
    pub wavelength_nm: f32,
    pub intensity_percent: f32,
    pub applied_voltage_v: f32,
    pub animation_running: bool,
    /// Se usa solo cuando todavía no llega un resultado físico de emisión.
    pub demo_electrons: bool,
}

impl Default for LabControls {
    fn default() -> Self {
        Self {
            material: MaterialChoice::Sodium,
            wavelength_nm: 400.0,
            intensity_percent: 55.0,
            applied_voltage_v: 0.0,
            animation_running: true,
            demo_electrons: true,
        }
    }
}

/// Punto de una curva que la capa física puede entregar a la gráfica.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CurvePoint {
    pub wavelength_nm: f64,
    pub max_kinetic_energy_ev: f64,
}

/// Datos calculados que la UI presenta, pero no obtiene por sí misma.
///
/// Al integrar la física, el adaptador de aplicación llena estos campos con
/// los resultados del modelo. `None` significa que aún no se conectaron.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PhysicsReadout {
    pub emission_possible: Option<bool>,
    pub frequency_hz: Option<f64>,
    pub photon_energy_ev: Option<f64>,
    pub work_function_ev: Option<f64>,
    pub threshold_frequency_hz: Option<f64>,
    pub threshold_wavelength_nm: Option<f64>,
    pub max_kinetic_energy_ev: Option<f64>,
    pub stopping_potential_v: Option<f64>,
    pub photon_flux_density_per_m2_s: Option<f64>,
    pub kinetic_energy_curve: Vec<CurvePoint>,
}
