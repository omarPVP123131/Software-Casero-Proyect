//! Experimento V₀–f: barrido automático determinista, ajuste lineal con
//! incertidumbre y exportación CSV con unidades.

use fotoelectrico_physics::{constants, fit_planck_constant, DataPoint};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::session::{ExperimentPoint, ExperimentState, MaterialChoice, SavedFit};

/// Generador determinista splitmix64: el "ruido" experimental es estable y
/// reproducible (la semilla viaja en la sesión).
pub fn splitmix_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Aplica ruido uniforme ±percent % al V₀ ideal. Determinista por semilla.
pub fn noisy_stopping_v(ideal_v: f64, noise_percent: f64, seed: &mut u64) -> f64 {
    let noise = if noise_percent.is_finite() {
        noise_percent.clamp(0.0, 25.0)
    } else {
        0.0
    } / 100.0;
    if !(ideal_v.is_finite() && ideal_v > 0.0) || noise <= 0.0 {
        return if ideal_v.is_finite() && ideal_v >= 0.0 {
            ideal_v
        } else {
            0.0
        };
    }
    let u = splitmix_next(seed) as f64 / u64::MAX as f64;
    (ideal_v * (1.0 + (u * 2.0 - 1.0) * noise)).max(0.0)
}

/// `seed` viaja como string: JS no puede representar u64 exactos y un f64
/// perdía los bits bajos (la API original usa splitmix64 de 64 bits).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export, export_to = "SweepRequest.ts")]
pub struct SweepRequest {
    pub material: MaterialChoice,
    pub custom_phi_ev: f64,
    pub count: usize,
    pub noise_percent: f64,
    #[serde(deserialize_with = "de_u64_from_string")]
    #[ts(type = "string")]
    pub seed: u64,
}

pub(crate) fn de_u64_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;
    impl serde::de::Visitor<'_> for Visitor {
        type Value = u64;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("u64 exacto como string o número entero")
        }
        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<u64, E> {
            value.parse::<u64>().map_err(E::custom)
        }
        fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<u64, E> {
            Ok(value)
        }
        fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<u64, E> {
            u64::try_from(value).map_err(E::custom)
        }
        fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<u64, E> {
            if value.is_finite() && value >= 0.0 && value <= u64::MAX as f64 {
                Ok(value as u64)
            } else {
                Err(E::custom("semilla fuera de rango u64"))
            }
        }
    }
    deserializer.deserialize_any(Visitor)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export, export_to = "SweepPoint.ts")]
pub struct SweepPoint {
    pub wavelength_nm: f64,
    pub frequency_hz: f64,
    pub stopping_ideal_v: f64,
    pub stopping_measured_v: f64,
    pub error_v: f64,
    pub material_id: String,
    pub material_name: String,
    pub noise_percent: f64,
}

/// Barrido automático: `count` puntos uniformes entre 200 nm y 10 nm por
/// debajo del umbral (siempre con emisión). Devuelve vacío si el umbral no
/// deja al menos dos puntos útiles.
pub fn auto_sweep(request: SweepRequest) -> Vec<SweepPoint> {
    let material = request.material.to_physics(request.custom_phi_ev);
    let lambda0_nm = constants::meters_to_nm(material.threshold_wavelength_m());
    let hi = (lambda0_nm - 10.0).clamp(200.0, 890.0);
    let lo = 200.0_f64;
    if hi - lo <= 1.0 || request.count < 2 {
        return Vec::new();
    }
    let n = request.count.min(12);
    let mut seed = request.seed;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let lambda = lo + (hi - lo) * i as f64 / (n - 1) as f64;
        let result = fotoelectrico_physics::photon::calculate_effect(&material, lambda);
        if !result.emits_electron {
            continue;
        }
        let freq = constants::SPEED_OF_LIGHT_C / constants::nm_to_meters(lambda);
        let ideal_v = result.k_max_j / constants::ELEMENTARY_CHARGE_E;
        let measured = noisy_stopping_v(ideal_v, request.noise_percent, &mut seed);
        out.push(SweepPoint {
            wavelength_nm: lambda,
            frequency_hz: freq,
            stopping_ideal_v: ideal_v,
            stopping_measured_v: measured,
            error_v: (measured - ideal_v).abs(),
            material_id: request.material.id().to_owned(),
            material_name: request.material.name_es().to_owned(),
            noise_percent: if measured != ideal_v {
                request.noise_percent
            } else {
                0.0
            },
        });
    }
    out
}

/// Resultado del ajuste V₀ = m·f + b, con h y su incertidumbre (1σ).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export, export_to = "FitReport.ts")]
pub struct FitReport {
    // Nota: los campos `*_v_s` / `*_j_s` se renombran a mano para que el
    // sufijo de unidad quede "Vs"/"Js" y no "VS"/"JS" (camelCase ingenuo).
    pub n_points: usize,
    #[serde(rename = "slopeVs")]
    pub slope_v_s: f64,
    #[serde(rename = "slopeStdVs")]
    pub slope_std_v_s: f64,
    pub intercept_v: f64,
    pub intercept_std_v: f64,
    #[serde(rename = "hExperimentalJs")]
    pub h_experimental_j_s: f64,
    #[serde(rename = "hStdJs")]
    pub h_std_j_s: f64,
    #[serde(rename = "hTheoreticalJs")]
    pub h_theoretical_j_s: f64,
    pub error_percentage: f64,
    pub r_squared: f64,
    /// Residuos (V) en el mismo orden que los puntos.
    pub residuals_v: Vec<f64>,
}

/// Ajuste por mínimos cuadrados con errores estándar (modelo lineal simple).
/// `fit_planck_constant` (del motor) aporta m, b, h y R²; aquí se añaden
/// σ_m, σ_b, residuos y la comparación con −Φ/e.
pub fn fit_report(points: &[ExperimentPoint]) -> Option<FitReport> {
    if points.len() < 2 {
        return None;
    }
    let data: Vec<DataPoint> = points
        .iter()
        .map(|p| DataPoint {
            frequency_hz: p.frequency_hz,
            stopping_potential_v: p.stopping_measured_v,
        })
        .collect();
    let base = fit_planck_constant(&data)?;
    let n = points.len() as f64;
    let (mut sx, mut sxx) = (0.0, 0.0);
    for p in points {
        sx += p.frequency_hz;
        sxx += p.frequency_hz * p.frequency_hz;
    }
    let denominator = n * sxx - sx * sx;
    let residuals_v: Vec<f64> = points
        .iter()
        .map(|p| p.stopping_measured_v - (base.slope_m * p.frequency_hz + base.intercept_b))
        .collect();
    let ss_res: f64 = residuals_v.iter().map(|r| r * r).sum();
    let dof = (points.len() as f64 - 2.0).max(1.0);
    let s2 = ss_res / dof;
    let (slope_std, intercept_std) = if denominator.abs() < 1e-30 || s2 == 0.0 {
        (0.0, 0.0)
    } else {
        (
            (n * s2 / denominator).sqrt(),
            (s2 * sxx / denominator).sqrt(),
        )
    };
    let h_std = slope_std * constants::ELEMENTARY_CHARGE_E;
    Some(FitReport {
        n_points: points.len(),
        slope_v_s: base.slope_m,
        slope_std_v_s: finite(slope_std),
        intercept_v: base.intercept_b,
        intercept_std_v: finite(intercept_std),
        h_experimental_j_s: base.h_experimental,
        h_std_j_s: finite(h_std),
        h_theoretical_j_s: constants::PLANCK_H,
        error_percentage: finite(base.error_percentage),
        r_squared: base.r_squared,
        residuals_v,
    })
}

fn finite(value: f64) -> f64 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// Convierte un `SweepPoint` en punto de tabla.
pub fn point_from_sweep(point: &SweepPoint) -> ExperimentPoint {
    ExperimentPoint {
        wavelength_nm: point.wavelength_nm,
        frequency_hz: point.frequency_hz,
        stopping_measured_v: point.stopping_measured_v,
        stopping_ideal_v: point.stopping_ideal_v,
        error_v: point.error_v,
        material_id: point.material_id.clone(),
        material_name: point.material_name.clone(),
        noise_percent: point.noise_percent,
    }
}

/// Ajuste → `SavedFit` persistible.
pub fn saved_fit_from(report: &FitReport, material_id: &str, material_name: &str) -> SavedFit {
    SavedFit {
        material_id: material_id.to_owned(),
        material_name: material_name.to_owned(),
        n_points: report.n_points,
        slope: report.slope_v_s,
        intercept: report.intercept_v,
        h_experimental: report.h_experimental_j_s,
        error_percentage: report.error_percentage,
        r_squared: report.r_squared,
    }
}

/// CSV del experimento con encabezados y unidades.
pub fn experiment_csv(state: &ExperimentState) -> String {
    let mut out = String::from(
        "n,material,wavelength_nm,frequency_hz,stopping_ideal_v,stopping_measured_v,error_v,noise_percent\n",
    );
    for (i, p) in state.points.iter().enumerate() {
        out.push_str(&format!(
            "{},{},{:.2},{:.6e},{:.6},{:.6},{:.6},{:.1}\n",
            i + 1,
            p.material_name,
            p.wavelength_nm,
            p.frequency_hz,
            p.stopping_ideal_v,
            p.stopping_measured_v,
            p.error_v,
            p.noise_percent,
        ));
    }
    if let Some(report) = fit_report(&state.points) {
        out.push_str(&format!(
            "# fit: slope={:.6e} V/Hz +-{:.2e}; h={:.6e} J*s +-{:.2e}; error={:.2} %; R2={:.6}\n",
            report.slope_v_s,
            report.slope_std_v_s,
            report.h_experimental_j_s,
            report.h_std_j_s,
            report.error_percentage,
            report.r_squared,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noisy_value_is_deterministic() {
        let mut a = 12345u64;
        let mut b = 12345u64;
        assert_eq!(
            noisy_stopping_v(1.0, 5.0, &mut a),
            noisy_stopping_v(1.0, 5.0, &mut b)
        );
        let mut c = 7u64;
        assert_eq!(noisy_stopping_v(1.0, 0.0, &mut c), 1.0);
    }

    #[test]
    fn sweep_covers_emitting_range() {
        let points = auto_sweep(SweepRequest {
            material: MaterialChoice::Sodium,
            custom_phi_ev: 3.0,
            count: 6,
            noise_percent: 0.0,
            seed: 42,
        });
        assert_eq!(points.len(), 6);
        assert!((points[0].wavelength_nm - 200.0).abs() < 0.01);
        assert!(points
            .windows(2)
            .all(|w| w[0].wavelength_nm < w[1].wavelength_nm));
        for p in &points {
            assert!(p.stopping_ideal_v > 0.0);
            assert_eq!(p.stopping_measured_v, p.stopping_ideal_v);
        }
    }

    fn ideal_points(material: MaterialChoice, phi: f64, noise: f64) -> Vec<ExperimentPoint> {
        auto_sweep(SweepRequest {
            material,
            custom_phi_ev: phi,
            count: 6,
            noise_percent: noise,
            seed: 9,
        })
        .iter()
        .map(point_from_sweep)
        .collect()
    }

    #[test]
    fn fit_recovers_planck_with_ideal_data() {
        let points = ideal_points(MaterialChoice::Sodium, 3.0, 0.0);
        let report = fit_report(&points).unwrap();
        assert!(report.error_percentage < 1.0);
        assert!(report.r_squared > 0.999);
        assert!((report.slope_std_v_s).abs() < 1e-20);
    }

    #[test]
    fn fit_slope_is_material_independent() {
        let sodium = fit_report(&ideal_points(MaterialChoice::Sodium, 3.0, 0.0)).unwrap();
        let zinc = fit_report(&ideal_points(MaterialChoice::Zinc, 3.0, 0.0)).unwrap();
        assert!((sodium.slope_v_s - zinc.slope_v_s).abs() / sodium.slope_v_s < 1e-6);
        // Solo cambia la ordenada: la pendiente es h/e.
        assert!(sodium.intercept_v > zinc.intercept_v);
    }

    #[test]
    fn noisy_fit_has_error_estimates() {
        let points = ideal_points(MaterialChoice::Sodium, 3.0, 3.0);
        let report = fit_report(&points).unwrap();
        assert!(report.h_std_j_s >= 0.0);
        assert_eq!(report.residuals_v.len(), points.len());
    }

    #[test]
    fn csv_has_units_header_and_fit_line() {
        let state = ExperimentState {
            points: ideal_points(MaterialChoice::Sodium, 3.0, 0.0),
            ..Default::default()
        };
        let csv = experiment_csv(&state);
        assert!(csv.starts_with("n,material,wavelength_nm,frequency_hz"));
        assert!(csv.contains("# fit: slope="));
    }
}
