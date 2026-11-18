//! Implementación de los comandos (ver `lib.rs`).

use photolab_model::{
    catalog::material_catalog,
    experiments::{
        auto_sweep, experiment_csv as csv_for_state, fit_report, SweepPoint, SweepRequest,
    },
    readout::evaluate_full,
    session::{ExperimentControls, ExperimentPoint, ExperimentState, Session, UiPreferences},
    FitReport,
};
use serde_json::Value;

/// Evalúa la sesión completa: lecturas, comparación y curva Kmax(λ).
/// La UI llama a esto UNA vez por cambio de control (debounce de un frame),
/// nunca 60 veces por segundo.
#[tauri::command]
pub fn physics_evaluate(controls_json: String, prefs_json: String) -> Result<Value, String> {
    let controls: ExperimentControls =
        serde_json::from_str(&controls_json).map_err(|e| format!("controls: {e}"))?;
    let prefs: UiPreferences =
        serde_json::from_str(&prefs_json).map_err(|e| format!("preferences: {e}"))?;
    let response = evaluate_full(&controls, &prefs, 61, true);
    serde_json::to_value(response).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn physics_material_catalog() -> Result<Value, String> {
    serde_json::to_value(material_catalog()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn experiment_fit(points_json: String) -> Result<Value, String> {
    let points: Vec<ExperimentPoint> =
        serde_json::from_str(&points_json).map_err(|e| format!("points: {e}"))?;
    let report: Option<FitReport> = fit_report(&points);
    serde_json::to_value(report).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn experiment_sweep(request_json: String) -> Result<Value, String> {
    let request: SweepRequest =
        serde_json::from_str(&request_json).map_err(|e| format!("sweep: {e}"))?;
    let points: Vec<SweepPoint> = auto_sweep(request);
    serde_json::to_value(points).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn experiment_csv(experiment_json: String) -> Result<String, String> {
    let experiment: ExperimentState =
        serde_json::from_str(&experiment_json).map_err(|e| format!("experiment: {e}"))?;
    Ok(csv_for_state(&experiment))
}

/// Valida y normaliza una sesión entrante (útil antes de autoguardar).
#[tauri::command]
pub fn session_normalize(session_json: String) -> Result<Value, String> {
    let session: Session =
        serde_json::from_str(&session_json).map_err(|e| format!("session: {e}"))?;
    let mut normalized = session;
    normalized.schema_version = photolab_model::SCHEMA_VERSION;
    normalized.controls = normalized.controls.sanitized();
    // La tabla V₀–f solo admite un material a la vez (la pendiente es h/e).
    if let Some(first) = normalized.experiment.points.first() {
        let first_id = first.material_id.clone();
        normalized
            .experiment
            .points
            .retain(|p| p.material_id == first_id);
    }
    normalized.experiment.saved_fits.truncate(6);
    serde_json::to_value(normalized).map_err(|e| e.to_string())
}

// ---------- Comandos de disco (los únicos con efectos secundarios) ----------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    pub path: String,
    pub bytes: usize,
}

/// Guarda la sesión `.photolab.json` y devuelve la ruta absoluta.
#[tauri::command]
pub fn save_session(session_json: String, path: Option<String>) -> Result<SaveResult, String> {
    let session: Session =
        serde_json::from_str(&session_json).map_err(|e| format!("session: {e}"))?;
    let pretty = serde_json::to_string_pretty(&session).map_err(|e| e.to_string())?;
    let target = path.unwrap_or_else(|| "sesion.photolab.json".to_owned());
    std::fs::write(&target, pretty.as_bytes())
        .map_err(|e| format!("No se pudo guardar {target}: {e}"))?;
    let absolute = std::path::Path::new(&target)
        .canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(target);
    Ok(SaveResult {
        path: absolute,
        bytes: pretty.len(),
    })
}

/// Carga `.photolab.json`; tolera campos ausentes (migración con serde defaults).
#[tauri::command]
pub fn load_session(path: String) -> Result<Value, String> {
    let contents =
        std::fs::read_to_string(&path).map_err(|e| format!("No se pudo leer {path}: {e}"))?;
    let session: Session =
        serde_json::from_str(&contents).map_err(|e| format!("JSON inválido: {e}"))?;
    serde_json::to_value(session).map_err(|e| e.to_string())
}

/// Escribe un archivo de exportación (CSV/PNG/SVG/JSON) y devuelve la ruta.
#[tauri::command]
pub fn export_file(filename: String, contents: String) -> Result<SaveResult, String> {
    // Seguridad mínima: sin rutas absolutas ni escapes del directorio actual.
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err("Nombre de archivo no permitido".to_owned());
    }
    std::fs::write(&filename, contents.as_bytes())
        .map_err(|e| format!("No se pudo escribir {filename}: {e}"))?;
    let absolute = std::path::Path::new(&filename)
        .canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(filename);
    Ok(SaveResult {
        path: absolute,
        bytes: contents.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_command_returns_full_contract() {
        let controls = serde_json::to_string(&ExperimentControls::default()).unwrap();
        let prefs = serde_json::to_string(&UiPreferences::default()).unwrap();
        let value = physics_evaluate(controls, prefs).unwrap();
        assert_eq!(value["readout"]["emits"], true);
        assert_eq!(value["curve"].as_array().unwrap().len(), 61);
        assert!(value["readout"]["kMaxEv"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn catalog_command_lists_six_materials() {
        let value = physics_material_catalog().unwrap();
        let arr = value.as_array().unwrap();
        assert_eq!(arr.len(), 6);
        assert_eq!(arr[1]["id"], "sodium");
    }

    #[test]
    fn normalize_drops_mixed_materials_and_bad_values() {
        let mut session = Session::default();
        // `f64::NAN` se serializa como null en JSON: usamos un valor extremo real.
        session.controls.wavelength_nm = 1.0e9;
        session.experiment.points.push(ExperimentPoint {
            material_id: "sodium".into(),
            ..Default::default()
        });
        session.experiment.points.push(ExperimentPoint {
            material_id: "copper".into(),
            ..Default::default()
        });
        let value = session_normalize(serde_json::to_string(&session).unwrap()).unwrap();
        assert_eq!(value["experiment"]["points"].as_array().unwrap().len(), 1);
        // 1e9 nm se satura al máximo del slider (900 nm), nunca NaN.
        assert_eq!(value["controls"]["wavelengthNm"], 900.0);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir
            .path()
            .join("t.photolab.json")
            .to_string_lossy()
            .into_owned();
        let session = Session {
            notes: "medir h".into(),
            ..Session::default()
        };
        let saved =
            save_session(serde_json::to_string(&session).unwrap(), Some(path.clone())).unwrap();
        assert!(saved.bytes > 0);
        let loaded = load_session(path).unwrap();
        assert_eq!(loaded["notes"], "medir h");
    }

    #[test]
    fn export_rejects_path_traversal() {
        assert!(export_file("../evil.csv".into(), "x".into()).is_err());
        assert!(export_file("a/b.csv".into(), "x".into()).is_err());
    }
}
