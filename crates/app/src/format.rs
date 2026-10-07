//! Formato corto de magnitudes para tarjetas de lectura.
//!
//! Objetivo: valores que nunca se solapan en paneles de 220–390 px.
//! Las frecuencias, flujos y velocidades grandes usan notación científica
//! compacta con superíndices Unicode (`5.45×10¹⁴ Hz`), del mismo estilo
//! que el renderizado de las fórmulas de [`crate::math`].

/// Convierte dígitos y signo a superíndices Unicode.
pub fn superscript(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            '-' => '⁻',
            '+' => '⁺',
            _ => c,
        })
        .collect()
}

/// Notación científica compacta: `5.45×10¹⁴`. Cero y no-finitos → `"0"`.
pub fn scientific(value: f64, decimals: usize) -> String {
    if !value.is_finite() {
        return "—".to_owned();
    }
    if value == 0.0 {
        return "0".to_owned();
    }
    let exponent = value.abs().log10().floor() as i32;
    // Mantisa en [1, 10): evita "10.2×10³", prefiere "1.02×10⁴".
    if exponent == 0 {
        return format!("{value:.decimals$}");
    }
    let mantissa = value / 10_f64.powi(exponent);
    format!(
        "{mantissa:.decimals$}×10{}",
        superscript(&exponent.to_string())
    )
}

/// Frecuencia en Hz, siempre científica compacta (`5.45×10¹⁴ Hz`).
pub fn hz(value: Option<f64>) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() || v < 0.0 {
            return "—".to_owned();
        }
        format!("{} Hz", scientific(v, 2))
    })
}

/// Electronvoltios con 2 decimales (`3.10 eV`). Corto y exacto para el rango 0–7 eV.
pub fn ev(value: Option<f64>) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() || v < 0.0 {
            return "—".to_owned();
        }
        format!("{v:.2} eV")
    })
}

/// Voltios con 2 decimales (`1.24 V`).
pub fn volts(value: Option<f64>) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() || v < 0.0 {
            return "—".to_owned();
        }
        format!("{v:.2} V")
    })
}

/// Nanómetros con 1 decimal (`541.2 nm`).
pub fn nm(value: Option<f64>) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() || v < 0.0 {
            return "—".to_owned();
        }
        format!("{v:.1} nm")
    })
}

/// Flujo de fotones en m⁻²·s⁻¹, siempre científico (`1.23×10²⁰ m⁻²·s⁻¹`).
pub fn flux(value: Option<f64>) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() || v < 0.0 {
            return "—".to_owned();
        }
        if v == 0.0 {
            return "0 m⁻²·s⁻¹".to_owned();
        }
        format!("{} m⁻²·s⁻¹", scientific(v, 2))
    })
}

/// Velocidad en m/s: científica a partir de 1e5 (`8.72×10⁵ m/s`), si no 0 decimales.
pub fn speed(value: Option<f64>) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() || v < 0.0 {
            return "—".to_owned();
        }
        if v == 0.0 {
            return "0 m/s".to_owned();
        }
        if v >= 1e5 {
            format!("{} m/s", scientific(v, 2))
        } else {
            format!("{v:.0} m/s")
        }
    })
}

/// Corriente con unidad automática: A, mA, µA o nA (`17.72 µA`).
pub fn current(value: Option<f64>) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() || v < 0.0 {
            return "—".to_owned();
        }
        if v == 0.0 {
            return "0 µA".to_owned();
        }
        if v >= 1.0 {
            format!("{v:.3} A")
        } else if v >= 1e-3 {
            format!("{:.3} mA", v * 1e3)
        } else if v >= 1e-6 {
            format!("{:.2} µA", v * 1e6)
        } else {
            format!("{:.1} nA", v * 1e9)
        }
    })
}

/// Constante de Planck en J·s, siempre científica (`6.63×10⁻³⁴ J·s`).
pub fn planck(value: Option<f64>) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() || v < 0.0 {
            return "—".to_owned();
        }
        format!("{} J·s", scientific(v, 2))
    })
}

/// Pendiente V₀–f en V·s, siempre científica (`4.14×10⁻¹⁵ V·s`).
pub fn slope(value: Option<f64>) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() {
            return "—".to_owned();
        }
        format!("{} V·s", scientific(v, 2))
    })
}

/// Porcentaje con 1 decimal (`2.4 %`).
pub fn percent(value: Option<f64>) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() || v < 0.0 {
            return "—".to_owned();
        }
        format!("{v:.1} %")
    })
}

/// Valor con signo y unidad (`−2.36 V`). Para magnitudes que pueden ser
/// negativas, como la ordenada del ajuste V0 = m·f + b.
pub fn signed(value: Option<f64>, decimals: usize, unit: &str) -> Option<String> {
    value.map(|v| {
        if !v.is_finite() {
            return "—".to_owned();
        }
        if v == 0.0 {
            return format!("{:.decimals$}{unit}", 0.0);
        }
        let sign = if v < 0.0 { "−" } else { "" };
        format!("{sign}{:.decimals$}{unit}", v.abs())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hz_uses_compact_scientific_notation() {
        assert_eq!(hz(Some(5.45e14)).as_deref(), Some("5.45×10¹⁴ Hz"));
        // Antes: "550000000000000.000 Hz" (24 caracteres, se solapaba).
        assert!(hz(Some(5.45e14)).unwrap().len() < 24);
    }

    #[test]
    fn flux_and_speed_are_compact() {
        assert_eq!(flux(Some(1.23e20)).as_deref(), Some("1.23×10²⁰ m⁻²·s⁻¹"));
        assert_eq!(speed(Some(8.72e5)).as_deref(), Some("8.72×10⁵ m/s"));
        assert_eq!(speed(Some(0.0)).as_deref(), Some("0 m/s"));
    }

    #[test]
    fn invalid_values_show_dash_never_nan() {
        assert_eq!(hz(Some(f64::NAN)).as_deref(), Some("—"));
        assert_eq!(ev(Some(f64::INFINITY)).as_deref(), Some("—"));
        assert_eq!(flux(Some(-1.0)).as_deref(), Some("—"));
    }

    #[test]
    fn current_picks_units_automatically() {
        assert_eq!(current(Some(0.0)).as_deref(), Some("0 µA"));
        assert_eq!(current(Some(17.72e-6)).as_deref(), Some("17.72 µA"));
        assert_eq!(current(Some(2.5e-3)).as_deref(), Some("2.500 mA"));
        assert_eq!(current(Some(f64::NAN)).as_deref(), Some("—"));
    }

    #[test]
    fn planck_and_slope_use_scientific_notation() {
        assert_eq!(
            planck(Some(6.62607015e-34)).as_deref(),
            Some("6.63×10⁻³⁴ J·s")
        );
        assert_eq!(slope(Some(4.14e-15)).as_deref(), Some("4.14×10⁻¹⁵ V·s"));
    }

    #[test]
    fn signed_keeps_negative_values_visible() {
        // La ordenada b ≈ −Φ/e es negativa: jamás debe mostrar "—".
        assert_eq!(signed(Some(-2.36), 2, " V").as_deref(), Some("−2.36 V"));
        assert_eq!(signed(Some(0.5), 2, " V").as_deref(), Some("0.50 V"));
        assert_eq!(signed(Some(f64::NAN), 2, " V").as_deref(), Some("—"));
    }
}
