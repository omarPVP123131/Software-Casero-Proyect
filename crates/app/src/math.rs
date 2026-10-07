//! Fórmulas del efecto fotoeléctrico en LaTeX + renderizado Unicode.
//!
//! egui no renderiza LaTeX de forma nativa, así que cada fórmula guarda su
//! fuente LaTeX (para copiar al informe) y su renderizado Unicode con la
//! misma semántica, que es lo que se pinta en la tarjeta:
//!
//! ```text
//! LaTeX:    $K_{\max} = hf - \Phi$
//! Render:   Kₘₐₓ = h·f − Φ
//! ```
//!
//! El tooltip de cada tarjeta muestra ambas formas.

/// Una fórmula: fuente LaTeX y su renderizado compacto para egui.
#[derive(Debug, Clone, Copy)]
pub struct Formula {
    /// Fuente LaTeX lista para copiar, p. ej. `$K_{\max} = hf - \Phi$`.
    pub latex: &'static str,
    /// Renderizado Unicode que se pinta, p. ej. `Kₘₐₓ = h·f − Φ`.
    pub rendered: &'static str,
}

impl Formula {
    pub const fn new(latex: &'static str, rendered: &'static str) -> Self {
        Self { latex, rendered }
    }
}

/// LaTeX: `$h\,f > \Phi$`
pub const EMISSION_COND: Formula = Formula::new(r"$h\,f > \Phi$", "h·f > Φ");
/// LaTeX: `$f = c / \lambda$`
pub const FREQUENCY: Formula = Formula::new(r"$f = c / \lambda$", "f = c / λ");
/// LaTeX: `$E = h\,f$`
pub const PHOTON_ENERGY: Formula = Formula::new(r"$E = h\,f$", "E = h·f");
/// LaTeX: `$\Phi$ (material)`
pub const WORK_FUNCTION: Formula = Formula::new(r"$\Phi$ (material)", "Φ del material");
/// LaTeX: `$f_0 = \Phi / h$`
pub const THRESHOLD_FREQ: Formula = Formula::new(r"$f_0 = \Phi / h$", "f₀ = Φ / h");
/// LaTeX: `$\lambda_0 = c / f_0$`
pub const THRESHOLD_WL: Formula = Formula::new(r"$\lambda_0 = c / f_0$", "λ₀ = c / f₀");
/// LaTeX: `$K_{\max} = h\,f - \Phi$`
pub const KMAX: Formula = Formula::new(r"$K_{\max} = h\,f - \Phi$", "Kₘₐₓ = h·f − Φ");
/// LaTeX: `$V_0 = K_{\max} / e$`
pub const STOPPING: Formula = Formula::new(r"$V_0 = K_{\max} / e$", "V₀ = Kₘₐₓ / e");
/// LaTeX: `$\Phi_{\text{fot}} = P / E_{\text{fotón}}$`
pub const FLUX: Formula = Formula::new(
    r"$\Phi_{\text{fot}} = P / E_{\text{fotón}}$",
    "Φ_fot = P / E_fotón",
);
/// LaTeX: `$v_{\max} = \sqrt{2\,K_{\max} / m_e}$`
pub const SPEED: Formula = Formula::new(
    r"$v_{\max} = \sqrt{2\,K_{\max} / m_e}$",
    "vₘₐₓ = √(2·Kₘₐₓ / mₑ)",
);
/// LaTeX: `$V < -V_0 \Rightarrow$ bloqueo`
pub const COLLECTION: Formula =
    Formula::new(r"$V < -V_0 \Rightarrow$ bloqueo", "V < −V₀ ⇒ bloqueo");
/// LaTeX: `$V_0 = m\,f + b$`
pub const FIT_LINE: Formula = Formula::new(r"$V_0 = m\,f + b$", "V₀ = m·f + b");
/// LaTeX: `$h = e \cdot m$`
pub const PLANCK_FIT: Formula = Formula::new(r"$h = e \cdot m$", "h = e·m");
/// LaTeX: `$R^2 = 1 - SS_{\mathrm{res}} / SS_{\mathrm{tot}}$`
pub const RSQUARED: Formula = Formula::new(
    r"$R^2 = 1 - SS_{\mathrm{res}} / SS_{\mathrm{tot}}$",
    "R² = 1 − SS_res/SS_tot",
);
/// LaTeX: `$I = e\,\Phi\,A\,\mathrm{QE}\,g(V)$`
pub const PHOTOCURRENT: Formula =
    Formula::new(r"$I = e\,\Phi\,A\,\mathrm{QE}\,g(V)$", "I = e·Φ·A·QE·g(V)");
