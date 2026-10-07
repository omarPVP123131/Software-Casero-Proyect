// crates/physics/src/constants.rs

/// Constante de Planck (J·s)
pub const PLANCK_H: f64 = 6.626_070_15e-34;

/// Carga elemental del electrón (C)
pub const ELEMENTARY_CHARGE_E: f64 = 1.602_176_634e-19;

/// Velocidad de la luz en el vacío (m/s)
pub const SPEED_OF_LIGHT_C: f64 = 299_792_458.0;

/// Masa del electrón (kg)
pub const ELECTRON_MASS_M: f64 = 9.109_383_7e-31;

/// Factor de conversión: 1 eV en Joules
pub const EV_TO_JOULES: f64 = ELEMENTARY_CHARGE_E;

// Funciones auxiliares de conversión (útiles para la interfaz gráfica)

/// Convierte electronvoltios (eV) a Joules (J)
pub fn ev_to_joules(ev: f64) -> f64 {
    ev * EV_TO_JOULES
}

/// Convierte Joules (J) a electronvoltios (eV)
pub fn joules_to_ev(joules: f64) -> f64 {
    joules / EV_TO_JOULES
}

/// Convierte nanómetros (nm) a metros (m)
pub fn nm_to_meters(nm: f64) -> f64 {
    nm * 1e-9
}

/// Convierte metros (m) a nanómetros (nm)
pub fn meters_to_nm(meters: f64) -> f64 {
    meters * 1e9
}