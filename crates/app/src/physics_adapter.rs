//! Puente entre los controles de la UI y `fotoelectrico-physics`.
//!
//! Toda magnitud física se calcula aquí llamando al motor. La UI y el
//! renderer solo presentan estos valores.

use fotoelectrico_physics::{constants, Material};

use crate::state::{AppState, CurvePoint, ExperimentControls, MaterialChoice, PhysicsReadout};

/// Irradiancia de referencia: 100 % de intensidad equivale a 10 mW/cm²
/// (100 W/m²). Modelo lineal documentado para la demo.
pub const REFERENCE_IRRADIANCE_W_M2: f64 = 100.0;

/// Área emisora del cátodo (demo ilustrativa: 1 cm²).
pub const CATHODE_AREA_M2: f64 = 1e-4;

/// Eficiencia cuántica ilustrativa: 1 de cada 100 fotones libera un electrón.
pub const QUANTUM_EFFICIENCY: f64 = 0.01;

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
/// Modelo mínimo binario para la animación: el frenado bloquea la colección
/// cuando `V_aplicado < -V0`. Kmax y V0 no cambian; solo la corriente
/// colectada. Garantía: entradas inválidas → sin colección pero sin fallar.
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

/// Fracción de colección continua 0..1 (rampa de frenado).
///
/// A diferencia de `is_collected` (binario, para la animación), esto modela
/// la curva I–V: con `V ≥ 0` se colecta todo; entre `−V₀` y 0 la colección
/// crece linealmente; por debajo de `−V₀` es cero. Kmax no cambia.
pub fn collection_factor(
    emits: bool,
    stopping_potential_v: f64,
    applied_voltage_v: f32,
    intensity_percent: f32,
) -> f64 {
    let intensity = sanitize(intensity_percent, 0.0, 0.0, 100.0);
    let applied = sanitize(applied_voltage_v, 0.0, -30.0, 30.0) as f64;
    let v0 = if stopping_potential_v.is_finite() && stopping_potential_v > 0.0 {
        stopping_potential_v
    } else {
        0.0
    };
    if !emits || intensity <= 0.1 || v0 <= 0.0 {
        // Sin V₀ (sin emisión) no hay rampa que modelar; con V ≥ 0 y emisión
        // la colección es total aunque v0 sea 0 por redondeo.
        if emits && intensity > 0.1 && applied >= 0.0 {
            return 1.0;
        }
        return 0.0;
    }
    if applied >= 0.0 {
        1.0
    } else if applied <= -v0 {
        0.0
    } else {
        ((applied + v0) / v0).clamp(0.0, 1.0)
    }
}

/// Fotocorriente estimada en amperios: `I = e·Φ·A·QE·g(V)`.
///
/// `Φ` = flujo de fotones, `A` = área del cátodo, `QE` = eficiencia
/// cuántica, `g(V)` = rampa de colección. Constantes A y QE ilustrativas
/// (ver `CATHODE_AREA_M2`, `QUANTUM_EFFICIENCY`).
pub fn photocurrent_a(
    photon_flux_per_m2_s: f64,
    emits: bool,
    stopping_potential_v: f64,
    applied_voltage_v: f32,
    intensity_percent: f32,
) -> f64 {
    let flux = if photon_flux_per_m2_s.is_finite() && photon_flux_per_m2_s > 0.0 {
        photon_flux_per_m2_s
    } else {
        return 0.0;
    };
    let g = collection_factor(
        emits,
        stopping_potential_v,
        applied_voltage_v,
        intensity_percent,
    );
    let current = constants::ELEMENTARY_CHARGE_E * flux * CATHODE_AREA_M2 * QUANTUM_EFFICIENCY * g;
    if current.is_finite() && current >= 0.0 {
        current
    } else {
        0.0
    }
}

/// Generador determinista splitmix64: el ruido experimental es estable entre
/// frames porque se genera una sola vez al capturar cada punto.
pub fn splitmix_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Aplica ruido uniforme ±`noise_percent` % al V₀ ideal. Determinista por semilla.
pub fn noisy_stopping_v(ideal_v: f64, noise_percent: f32, seed: &mut u64) -> f64 {
    let noise = noise_percent.clamp(0.0, 25.0) as f64 / 100.0;
    if !(ideal_v.is_finite() && ideal_v > 0.0) || noise <= 0.0 {
        return if ideal_v.is_finite() && ideal_v >= 0.0 {
            ideal_v
        } else {
            0.0
        };
    }
    let u = splitmix_next(seed) as f64 / u64::MAX as f64; // [0, 1)
    (ideal_v * (1.0 + (u * 2.0 - 1.0) * noise)).max(0.0)
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
    let factor = collection_factor(
        readout.emits,
        readout.stopping_potential_v,
        applied,
        intensity,
    );
    let current = photocurrent_a(
        flux,
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
        photocurrent_a: Some(current),
        collection_factor: Some(factor),
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

/// CSV del experimento V₀ contra f. Encabezados con unidades.
/// Incluye V₀ ideal y medido (con ruido) para auditar el ajuste.
pub fn experiment_csv(points: &[crate::state::ExperimentPoint]) -> String {
    let mut out = String::from(
        "n,material,wavelength_nm,frequency_hz,stopping_ideal_v,stopping_measured_v,noise_percent\n",
    );
    for (i, p) in points.iter().enumerate() {
        out.push_str(&format!(
            "{},{},{:.2},{:.6e},{:.6},{:.6},{:.1}\n",
            i + 1,
            p.material_name,
            p.wavelength_nm,
            p.frequency_hz,
            p.stopping_ideal_v,
            p.stopping_measured_v,
            p.noise_percent,
        ));
    }
    out
}

/// CSV de la sesión completa: parámetros, lecturas, curva Kmax(λ),
/// puntos del experimento y notas del operador.
pub fn session_csv(state: &AppState) -> String {
    let mut out = String::new();
    out.push_str("# PhotoLab — sesión del efecto fotoeléctrico\n");
    out.push_str(&format!(
        "# material,{}\n# comparison_material,{}\n# wavelength_nm,{:.1}\n# intensity_percent,{:.1}\n# applied_voltage_v,{:+.2}\n",
        state.controls.material.name(),
        state.controls.comparison_material.name(),
        state.controls.wavelength_nm,
        state.controls.intensity_percent,
        state.controls.applied_voltage_v,
    ));
    let r = &state.readout;
    out.push_str(&format!(
        "# emission,{}\n# collected,{}\n# frequency_hz,{:.6e}\n# photon_energy_ev,{:.4}\n# work_function_ev,{:.4}\n# kmax_ev,{:.4}\n# stopping_v,{:.4}\n# photocurrent_a,{:.6e}\n",
        r.emission_possible.unwrap_or(false),
        r.collected_possible.unwrap_or(false),
        r.frequency_hz.unwrap_or(0.0),
        r.photon_energy_ev.unwrap_or(0.0),
        r.work_function_ev.unwrap_or(0.0),
        r.max_kinetic_energy_ev.unwrap_or(0.0),
        r.stopping_potential_v.unwrap_or(0.0),
        r.photocurrent_a.unwrap_or(0.0),
    ));
    out.push_str("\n[curva_kmax_vs_lambda]\nwavelength_nm,kmax_ev\n");
    for point in &r.kinetic_energy_curve {
        out.push_str(&format!(
            "{:.2},{:.6}\n",
            point.wavelength_nm, point.max_kinetic_energy_ev
        ));
    }
    out.push_str("\n[experimento_v0_vs_f]\n");
    out.push_str(&experiment_csv(&state.experiment.points));
    out.push_str("\n[notas]\n");
    for line in state.notes.lines() {
        out.push_str(&format!("# {line}\n"));
    }
    out
}

/// Guarda texto en el directorio de trabajo. Devuelve la ruta absoluta
/// para mostrarla en la UI. Sin diálogos: simple y sin dependencias.
pub fn save_text_file(filename: &str, contents: &str) -> Result<String, String> {
    std::fs::write(filename, contents).map_err(|e| format!("No se pudo guardar: {e}"))?;
    let path = std::env::current_dir()
        .map(|dir| dir.join(filename).to_string_lossy().into_owned())
        .unwrap_or_else(|_| filename.to_owned());
    Ok(path)
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
                        readout.photocurrent_a,
                        readout.collection_factor,
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

    #[test]
    fn noise_is_deterministic_and_bounded() {
        let mut a = 12345u64;
        let mut b = 12345u64;
        let v1 = noisy_stopping_v(1.0, 5.0, &mut a);
        let v2 = noisy_stopping_v(1.0, 5.0, &mut b);
        assert_eq!(v1, v2, "misma semilla → mismo ruido");
        assert!((v1 - 1.0).abs() <= 0.05 + 1e-12, "ruido dentro de ±5%");
        let mut c = 999u64;
        assert_eq!(noisy_stopping_v(1.0, 0.0, &mut c), 1.0, "sin ruido → ideal");
        assert_eq!(noisy_stopping_v(0.0, 5.0, &mut c), 0.0);
        assert!(noisy_stopping_v(1.0, 5.0, &mut c) >= 0.0);
    }

    #[test]
    fn collection_ramp_is_continuous_and_current_scales() {
        // Sodio 400 nm: V₀ ≈ 0.74 V. Rampa: 0 en −V₀, 1 en 0, 1 más allá.
        let free = build_readout(MaterialChoice::Sodium, 400.0, 55.0, 0.0);
        let v0 = free.stopping_potential_v.unwrap();
        assert!((free.collection_factor.unwrap() - 1.0).abs() < 1e-9);
        let blocked = build_readout(MaterialChoice::Sodium, 400.0, 55.0, -5.0);
        assert_eq!(blocked.collection_factor, Some(0.0));
        assert_eq!(blocked.photocurrent_a, Some(0.0));
        let mid = collection_factor(true, v0, (-v0 / 2.0) as f32, 55.0);
        assert!((mid - 0.5).abs() < 0.01, "rampa lineal a mitad del frenado");
        // Más intensidad → más corriente; sin emisión → cero.
        let dim = build_readout(MaterialChoice::Sodium, 400.0, 10.0, 0.0);
        let bright = build_readout(MaterialChoice::Sodium, 400.0, 90.0, 0.0);
        assert!(bright.photocurrent_a.unwrap() > dim.photocurrent_a.unwrap());
        assert!(bright.photocurrent_a.unwrap() > 0.0);
        let dark = build_readout(MaterialChoice::Sodium, 700.0, 90.0, 0.0);
        assert_eq!(dark.photocurrent_a, Some(0.0));
    }

    #[test]
    fn csv_builders_have_headers_and_units() {
        let csv = experiment_csv(&[]);
        assert!(csv.starts_with("n,material,"));
        assert!(csv.contains("frequency_hz"));
        assert!(csv.contains("stopping_measured_v"));
    }
}
