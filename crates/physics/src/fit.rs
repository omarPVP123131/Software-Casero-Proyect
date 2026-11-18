// crates/physics/src/fit.rs

use crate::constants::ELEMENTARY_CHARGE_E;

/// Representa un punto experimental: (Frecuencia en Hz, Potencial de Frenado en Voltios)
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub frequency_hz: f64,
    pub stopping_potential_v: f64,
}

/// Resultado del ajuste por regresión lineal
#[derive(Debug, Clone)]
pub struct LinearFitResult {
    pub slope_m: f64,          // Pendiente (V·s)
    pub intercept_b: f64,      // Intersección (V)
    pub h_experimental: f64,   // Constante de Planck estimada (J·s)
    pub error_percentage: f64, // Error porcentual comparado con el valor teórico
    pub r_squared: f64,        // Bondad del ajuste 0..1 (1 = puntos colineales)
}

/// Realiza una regresión lineal sobre una lista de puntos (f, V0)
pub fn fit_planck_constant(points: &[DataPoint]) -> Option<LinearFitResult> {
    let n = points.len() as f64;
    if n < 2.0 {
        return None; // Se necesitan al menos 2 puntos para una recta
    }

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_xx = 0.0;

    for p in points {
        let x = p.frequency_hz;
        let y = p.stopping_potential_v;
        sum_x += x;
        sum_y += y;
        sum_xy += x * y;
        sum_xx += x * x;
    }

    // Pendiente m = (N*Σ(xy) - Σx*Σy) / (N*Σ(x²) - (Σx)²)
    let denominator = n * sum_xx - sum_x * sum_x;
    if denominator.abs() < 1e-20 {
        return None; // Evitar división por cero
    }

    let slope_m = (n * sum_xy - sum_x * sum_y) / denominator;
    let intercept_b = (sum_y - slope_m * sum_x) / n;

    // h_exp = m * e
    let h_experimental = slope_m * ELEMENTARY_CHARGE_E;

    // Error porcentual = |h_exp - h_teorico| / h_teorico * 100
    let h_theoretical = crate::constants::PLANCK_H;
    let error_percentage = ((h_experimental - h_theoretical).abs() / h_theoretical) * 100.0;

    // R² = 1 - SSres/SStot. Con puntos colineales (datos ideales) → 1.
    let mean_y = sum_y / n;
    let mut ss_tot = 0.0;
    let mut ss_res = 0.0;
    for p in points {
        let y = p.stopping_potential_v;
        let y_fit = slope_m * p.frequency_hz + intercept_b;
        ss_tot += (y - mean_y) * (y - mean_y);
        ss_res += (y - y_fit) * (y - y_fit);
    }
    let r_squared = if ss_tot < 1e-30 {
        // Todos los V₀ iguales: ajuste perfecto solo si no hay residuo.
        if ss_res < 1e-30 {
            1.0
        } else {
            0.0
        }
    } else {
        (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
    };

    Some(LinearFitResult {
        slope_m,
        intercept_b,
        h_experimental,
        error_percentage,
        r_squared,
    })
}

// Pruebas unitarias para la regresión
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_fit_planck() {
        // Puntos teóricos idénticos derivados de V0 = (h/e)*f - (Φ/e)
        let sample_points = vec![
            DataPoint {
                frequency_hz: 6.0e14,
                stopping_potential_v: 0.191,
            },
            DataPoint {
                frequency_hz: 7.0e14,
                stopping_potential_v: 0.605,
            },
            DataPoint {
                frequency_hz: 8.0e14,
                stopping_potential_v: 1.019,
            },
        ];

        let result = fit_planck_constant(&sample_points).unwrap();

        // El error debe ser menor al 1% con datos ideales
        assert!(result.error_percentage < 1.0);
        // Datos casi colineales → R² ≈ 1.
        assert!(result.r_squared > 0.999);
    }

    #[test]
    fn test_r_squared_is_one_for_perfect_line() {
        // V₀ = m·f + b exacta con m = h/e real.
        let m = crate::constants::PLANCK_H / crate::constants::ELEMENTARY_CHARGE_E;
        let points: Vec<DataPoint> = (0..5)
            .map(|i| {
                let f = 6.0e14 + i as f64 * 5.0e13;
                DataPoint {
                    frequency_hz: f,
                    stopping_potential_v: m * f - 2.0,
                }
            })
            .collect();
        let result = fit_planck_constant(&points).unwrap();
        assert!((result.r_squared - 1.0).abs() < 1e-9);
        assert!(result.error_percentage < 0.01);
    }
}
