//! `photolab-model` — el contrato único entre la física, Tauri y WASM.
//!
//! Aquí viven:
//! - Los tipos que cruzan la frontera (`invoke` en Tauri / `wasm-bindgen` en web).
//! - El adaptador que traduce controles de UI a llamadas a `fotoelectrico-physics`.
//! - La persistencia de sesión (JSON serializable).
//!
//! Regla del proyecto: ni la UI de Svelte ni el backend de Tauri recalculan
//! física. Todo número visible sale de este crate, que a su vez solo llama a
//! `fotoelectrico-physics`.

pub mod catalog;
pub mod experiments;
pub mod readout;
pub mod session;

pub use catalog::{material_catalog, MaterialInfo};
pub use experiments::{
    auto_sweep, experiment_csv, fit_report, FitReport, SweepPoint, SweepRequest,
};
pub use fotoelectrico_physics::calculate_effect;
pub use readout::{
    collection_factor, evaluate, evaluate_full, is_collected, photocurrent_a,
    photon_flux_density_per_m2_s, CurvePoint, EvaluateResponse, Readout,
};
pub use session::{
    CanvasLayers, ControlChange, ExperimentControls, ExperimentPoint, ExperimentState, Lang,
    MaterialChoice, MotionPreference, SavedFit, Session, SpeedScaleMode, ThemeChoice,
    UiPreferences, SCHEMA_VERSION,
};

/// Constantes de presentación documentadas (no son física fundamental).
pub mod constants {
    /// 100 % de intensidad = 10 mW/cm² = 100 W/m² (modelo lineal de la demo).
    pub const REFERENCE_IRRADIANCE_W_M2: f64 = 100.0;
    /// Rango de la curva Kmax(λ) y del slider de λ.
    pub const CURVE_MIN_NM: f64 = 180.0;
    pub const CURVE_MAX_NM: f64 = 900.0;
    pub const CURVE_POINTS: usize = 61;
    /// Referencia de la escala visual de velocidad: K = 1 eV → 1.0.
    pub const SPEED_SCALE_K_REF_EV: f64 = 1.0;
    pub const SPEED_SCALE_MIN: f64 = 0.5;
    pub const SPEED_SCALE_MAX: f64 = 2.2;
    /// Límite de puntos de la tabla V₀–f.
    pub const MAX_EXPERIMENT_POINTS: usize = 24;
}

/// Exporta todos los bindings TypeScript a `frontend/src/lib/generated/`.
///
/// ts-rs 12 escribe cada tipo según su `#[ts(export_to = "...")]` dentro de
/// `TS_RS_EXPORT_DIR`. Con `PHOTOLAB_TS_OUT=<carpeta>` este test escribe los
/// `.ts` del frontend (ver `scripts/gen-types.sh`).
pub fn export_ts_bindings(out_dir: &str) {
    use ts_rs::TS;
    std::fs::create_dir_all(out_dir).expect("crear carpeta de bindings");
    std::env::set_var("TS_RS_EXPORT_DIR", out_dir);
    let cfg = ts_rs::Config::from_env();
    Readout::export_all(&cfg).expect("exportar Readout");
    EvaluateResponse::export_all(&cfg).expect("exportar EvaluateResponse");
    Session::export_all(&cfg).expect("exportar Session");
    MaterialChoice::export_all(&cfg).expect("exportar MaterialChoice");
    // Types sin dependencia recursiva de los anteriores (export_all de
    // Session ya cubre los anidados; estos van explícitos por claridad).
    use crate::experiments::FitReport;
    use crate::experiments::SweepRequest;
    FitReport::export_all(&cfg).expect("exportar FitReport");
    SweepRequest::export_all(&cfg).expect("exportar SweepRequest");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_bindings() {
        // `cargo test -p photolab-model export_bindings -- --ignored` desde la raíz
        // escribe los .ts; en CI normal no se toca el árbol fuente.
        let out = std::env::var("PHOTOLAB_TS_OUT");
        if let Ok(dir) = out {
            export_ts_bindings(&dir);
        }
    }

    #[test]
    fn contract_is_serializable_roundtrip() {
        let controls = session::ExperimentControls::default();
        let response = evaluate(&controls, 61);
        let session = Session::from_parts(
            controls,
            UiPreferences::default(),
            ExperimentState::default(),
            "nota".into(),
        );
        let json = serde_json::to_string(&session).expect("sesión serializable");
        let back: Session = serde_json::from_str(&json).expect("sesión deserializable");
        assert_eq!(back.controls.wavelength_nm, session.controls.wavelength_nm);
        assert!(response.readout.emits);
    }
}
