//! Puente entre los controles de la UI y `fotoelectrico-physics`.
//!
//! Toda magnitud física se calcula aquí llamando al motor. La UI y el
//! renderer solo presentan estos valores.

use fotoelectrico_physics::{constants, Material};

use crate::state::{AppState, CurvePoint, ExperimentControls, MaterialChoice, PhysicsReadout};

/// Irradiancia de referencia: 100 % de intensidad equivale a 10 mW/cm²
/// (100 W/m²). Modelo lineal documentado para la demo.
pub const REFERENCE_IRRADIANCE_W_M2: f64 = 100.0;

/// Rango de la curva Kmax(λ) mostrado en la pestaña Gráfica.
pub const CURVE_MIN_NM: f64 = 180.0;
pub const CURVE_MAX_NM: f64 = 900.0;
pub const CURVE_POINTS: usize = 61;

pub fn material_for(choice: MaterialChoice) -> Material {
    Material::new(choice.name(), choice.work_function_ev())
}

/// Sanea un f32 de la UI: NaN/inf → fallback; luego clamp al rango válido.
fn sanitize(value: f32, fallback: f32, min: f32, max: f32) -> f32 {
    if !value.is_finite() {
        return fallback;
    }
    value.clamp(min, max)
}

/// Flujo de fotones incidentes a partir de la intensidad visual.
///
/// `flujo = irradiancia / E_fotón`. Kmax y V0 no dependen de este valor;
/// solo escala la cantidad de electrones / brillo del haz.
/// Garantía: jamás NaN/inf; con E inválida devuelve 0.
pub fn photon_flux_density_per_m2_s(intensity_percent: f32, photon_energy_j: f64) -> f64 {
    let intensity = sanitize(intensity_percent, 0.0, 0.0, 100.0);
    if !photon_energy_j.is_finite() || photon_energy_j <= 0.0 || intensity <= 0.1 {
        return 0.0;
    }
    let irradiance = intensity as f64 / 100.0 * REFERENCE_IRRADIANCE_W_M2;
    let flux = irradiance / photon_energy_j;
    if flux.is_finite() && flux >= 0.0 {
        flux
    } else {
        0.0
    }
}

/// ¿Llegan los fotoelectrones al ánodo con el voltaje aplicado?
///
/// Modelo mínimo: el frenado bloquea la colección cuando
/// `V_aplicado < -V0`. Kmax y V0 no cambian; solo la corriente colectada.
/// Garantía: entradas inválidas → sin colección (no electrones) pero sin fallar.
pub fn is_collected(
    emits: bool,
    stopping_potential_v: f64,
    applied_voltage_v: f32,
    intensity_percent: f32,
) -> bool {
    let intensity = sanitize(intensity_percent, 0.0, 0.0, 100.0);
    let applied = sanitize(applied_voltage_v, 0.0, -30.0, 30.0);
    let v0 = if stopping_potential_v.is_finite() && stopping_potential_v >= 0.0 {
        stopping_potential_v
    } else {
        0.0
    };
    if !emits || intensity <= 0.1 {
        return false;
    }
    (applied as f64) >= -v0 - 1e-9
}

fn frequency_hz_for(wavelength_nm: f64) -> f64 {
    let clean = if wavelength_nm.is_finite() {
        wavelength_nm.max(1.0)
    } else {
        550.0
    };
    let lambda_m = constants::nm_to_meters(clean);
    let freq = constants::SPEED_OF_LIGHT_C / lambda_m;
    if freq.is_finite() && freq > 0.0 {
        freq
    } else {
        0.0
    }
}

pub fn build_readout(
    material: MaterialChoice,
    wavelength_nm: f32,
    intensity_percent: f32,
    applied_voltage_v: f32,
) -> PhysicsReadout {
    // Saneo total: el motor jamás se "desconecta" por parámetros extremos.
    let lambda = sanitize(wavelength_nm, 400.0, 180.0, 900.0) as f64;
    let intensity = sanitize(intensity_percent, 55.0, 0.0, 100.0);
    let applied = sanitize(applied_voltage_v, 0.0, -5.0, 5.0);
    let physics_material = material_for(material);
    let readout = fotoelectrico_physics::get_physics_readout(&physics_material, lambda);
    let frequency_hz = frequency_hz_for(lambda);
    let flux = photon_flux_density_per_m2_s(intensity, readout.photon_energy_j);
    let collected = is_collected(
        readout.emits,
        readout.stopping_potential_v,
        applied,
        intensity,
    );

    PhysicsReadout {
        emission_possible: Some(readout.emits),
        collected_possible: Some(collected),
        frequency_hz: Some(frequency_hz),
        photon_energy_ev: Some(readout.photon_energy_ev),
        work_function_ev: Some(readout.work_function_ev),
        threshold_frequency_hz: Some(readout.threshold_frequency_hz),
        threshold_wavelength_nm: Some(readout.threshold_wavelength_nm),
        max_kinetic_energy_ev: Some(readout.k_max_ev),
        stopping_potential_v: Some(readout.stopping_potential_v),
        photon_flux_density_per_m2_s: Some(flux),
        electron_max_speed_m_s: Some(readout.electron_max_speed_m_s),
        kinetic_energy_curve: build_curve(&physics_material),
    }
}

fn build_curve(material: &Material) -> Vec<CurvePoint> {
    let mut curve = Vec::with_capacity(CURVE_POINTS);
    for i in 0..CURVE_POINTS {
        let fraction = i as f64 / (CURVE_POINTS - 1) as f64;
        let lambda_nm = CURVE_MIN_NM + fraction * (CURVE_MAX_NM - CURVE_MIN_NM);
        let result = fotoelectrico_physics::photon::calculate_effect(material, lambda_nm);
        curve.push(CurvePoint {
            wavelength_nm: lambda_nm,
            max_kinetic_energy_ev: constants::joules_to_ev(result.k_max_j),
        });
    }
    curve
}

/// Recalcula ambas lecturas (principal + comparación) desde los controles.
pub fn refresh_state(state: &mut AppState) {
    refresh_controls(state, state.controls);
}

fn refresh_controls(state: &mut AppState, controls: ExperimentControls) {
    state.readout = build_readout(
        controls.material,
        controls.wavelength_nm,
        controls.intensity_percent,
        controls.applied_voltage_v,
    );
    state.comparison_readout = build_readout(
        controls.comparison_material,
        controls.wavelength_nm,
        controls.intensity_percent,
        controls.applied_voltage_v,
    );
}

/// Escala visual de velocidad a partir de Kmax: v ∝ sqrt(K).
/// Kref = 1 eV → escala 1.0. Solo afecta animación, no física.
/// Garantía: NaN/inf/negativo → 1.0.
pub fn electron_speed_scale(k_max_ev: Option<f64>) -> f32 {
    match k_max_ev {
        Some(k) if k.is_finite() && k > 1e-9 => (k / 1.0).sqrt().clamp(0.5, 2.2) as f32,
        _ => 1.0,
    }
}

/// Fuerza relativa de fotoelectrones 0..1 (intensidad cuando hay colección).
/// Garantía: jamás NaN; fuera de rango → 0.
pub fn photoelectron_strength(
    intensity_percent: f32,
    collected_possible: Option<bool>,
    emission_possible: Option<bool>,
) -> f32 {
    let intensity = sanitize(intensity_percent, 0.0, 0.0, 100.0);
    let collected = collected_possible.or(emission_possible).unwrap_or(true);
    if !collected || intensity <= 0.1 {
        return 0.0;
    }
    (intensity / 100.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readout_uses_real_physics_values() {
        let readout = build_readout(MaterialChoice::Sodium, 400.0, 55.0, 0.0);
        assert_eq!(readout.emission_possible, Some(true));
        assert!(readout.photon_energy_ev.unwrap() > 0.0);
        assert!(readout.max_kinetic_energy_ev.unwrap() > 0.0);
        assert_eq!(readout.kinetic_energy_curve.len(), CURVE_POINTS);
        assert!(readout.electron_max_speed_m_s.unwrap() > 0.0);
        assert!(readout.photon_flux_density_per_m2_s.unwrap() > 0.0);
    }

    #[test]
    fn red_light_does_not_emit_for_sodium_but_flux_exists() {
        let readout = build_readout(MaterialChoice::Sodium, 700.0, 80.0, 0.0);
        assert_eq!(readout.emission_possible, Some(false));
        assert_eq!(readout.max_kinetic_energy_ev, Some(0.0));
        assert_eq!(readout.collected_possible, Some(false));
        assert!(readout.photon_flux_density_per_m2_s.unwrap() > 0.0);
    }

    #[test]
    fn retarding_voltage_blocks_collection_without_changing_kmax() {
        let free = build_readout(MaterialChoice::Sodium, 400.0, 55.0, 0.0);
        let v0 = free.stopping_potential_v.unwrap();
        let blocked = build_readout(MaterialChoice::Sodium, 400.0, 55.0, (-v0 - 0.5) as f32);
        assert_eq!(blocked.emission_possible, Some(true));
        assert_eq!(blocked.collected_possible, Some(false));
        assert_eq!(
            blocked.max_kinetic_energy_ev, free.max_kinetic_energy_ev,
            "Kmax no depende del voltaje aplicado"
        );
    }

    #[test]
    fn intensity_does_not_change_kmax_or_stopping() {
        let low = build_readout(MaterialChoice::Sodium, 400.0, 10.0, 0.0);
        let high = build_readout(MaterialChoice::Sodium, 400.0, 90.0, 0.0);
        assert_eq!(low.max_kinetic_energy_ev, high.max_kinetic_energy_ev);
        assert_eq!(low.stopping_potential_v, high.stopping_potential_v);
        assert!(
            high.photon_flux_density_per_m2_s.unwrap() > low.photon_flux_density_per_m2_s.unwrap()
        );
    }

    #[test]
    fn speed_scale_grows_with_kmax() {
        assert!(electron_speed_scale(Some(2.0)) > electron_speed_scale(Some(0.5)));
        assert_eq!(electron_speed_scale(Some(0.0)), 1.0);
        assert_eq!(electron_speed_scale(None), 1.0);
    }

    #[test]
    fn adapter_never_disconnects_on_extreme_inputs() {
        // El motor siempre devuelve lecturas completas (Some + finitas),
        // incluso con parámetros absurdos: jamás "desconecta".
        for weird_lambda in [0.0, -50.0, f32::NAN, f32::INFINITY, 180.0, 900.0] {
            for weird_intensity in [0.0, -10.0, f32::NAN, f32::INFINITY, 100.0] {
                for weird_voltage in [-20.0, f32::NAN, f32::INFINITY, 5.0] {
                    let readout = build_readout(
                        MaterialChoice::Sodium,
                        weird_lambda,
                        weird_intensity,
                        weird_voltage,
                    );
                    assert!(readout.emission_possible.is_some());
                    assert!(readout.collected_possible.is_some());
                    for value in [
                        readout.frequency_hz,
                        readout.photon_energy_ev,
                        readout.work_function_ev,
                        readout.threshold_frequency_hz,
                        readout.threshold_wavelength_nm,
                        readout.max_kinetic_energy_ev,
                        readout.stopping_potential_v,
                        readout.photon_flux_density_per_m2_s,
                        readout.electron_max_speed_m_s,
                    ] {
                        let v = value.expect("toda lectura debe ser Some");
                        assert!(v.is_finite(), "lectura finita con λ={weird_lambda}");
                        assert!(v >= 0.0, "lectura no negativa");
                    }
                    assert_eq!(readout.kinetic_energy_curve.len(), CURVE_POINTS);
                }
            }
        }
    }

    #[test]
    fn long_wavelength_keeps_photon_flux_but_no_emission() {
        // Alta λ: el haz sigue (flujo > 0) aunque no haya emisión. Los fotones
        // no desaparecen por física, solo los electrones.
        let readout = build_readout(MaterialChoice::Sodium, 900.0, 80.0, 0.0);
        assert_eq!(readout.emission_possible, Some(false));
        assert!(readout.photon_flux_density_per_m2_s.unwrap() > 0.0);
        assert!(readout.photon_energy_ev.unwrap() > 0.0);
    }

    #[test]
    fn inverted_voltage_blocks_collection_keeps_photon_numbers() {
        // Voltaje invertido/frenado: bloquea colección sin tocar fotones ni Kmax.
        let free = build_readout(MaterialChoice::Potassium, 300.0, 70.0, 0.0);
        let blocked = build_readout(MaterialChoice::Potassium, 300.0, 70.0, -5.0);
        assert_eq!(blocked.emission_possible, Some(true));
        assert_eq!(blocked.collected_possible, Some(false));
        assert_eq!(
            blocked.photon_energy_ev, free.photon_energy_ev,
            "el voltaje no toca al fotón"
        );
        assert_eq!(
            blocked.max_kinetic_energy_ev, free.max_kinetic_energy_ev,
            "el voltaje no toca Kmax"
        );
        assert_eq!(
            blocked.photon_flux_density_per_m2_s, free.photon_flux_density_per_m2_s,
            "el voltaje no toca el flujo"
        );
    }
}
