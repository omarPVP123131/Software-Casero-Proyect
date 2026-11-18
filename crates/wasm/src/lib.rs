//! Bindings WASM del motor PhotoLab.
//!
//! Ventaja de compilar la física a WASM: el frontend calcula sin `invoke`
//! (cero IPC para sliders continuos) y **la misma física** corre en web y
//! en Tauri. Los comandos Tauri de `photolab-tauri-commands` llaman a los
//! mismos tipos de `photolab-model`.

use wasm_bindgen::prelude::*;

use photolab_model::{
    catalog::material_catalog,
    experiments::{auto_sweep, experiment_csv as csv_for_state, fit_report, SweepRequest},
    readout::evaluate_full,
    session::{ExperimentControls, ExperimentPoint, ExperimentState, UiPreferences},
};

fn err(message: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&message.to_string())
}

/// Lecturas + curva Kmax(λ). `controls` y `prefs` son objetos JS planos
/// con el mismo JSON que usan los comandos Tauri.
#[wasm_bindgen]
pub fn physics_evaluate(controls: JsValue, prefs: JsValue) -> Result<JsValue, JsValue> {
    let controls: ExperimentControls = serde_wasm_bindgen::from_value(controls).map_err(err)?;
    let prefs: UiPreferences = serde_wasm_bindgen::from_value(prefs).map_err(err)?;
    let response = evaluate_full(&controls, &prefs, 61, true);
    serde_wasm_bindgen::to_value(&response).map_err(err)
}

#[wasm_bindgen]
pub fn physics_material_catalog() -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&material_catalog()).map_err(err)
}

#[wasm_bindgen]
pub fn experiment_fit(points: JsValue) -> Result<JsValue, JsValue> {
    let points: Vec<ExperimentPoint> = serde_wasm_bindgen::from_value(points).map_err(err)?;
    serde_wasm_bindgen::to_value(&fit_report(&points)).map_err(err)
}

#[wasm_bindgen]
pub fn experiment_sweep(request: JsValue) -> Result<JsValue, JsValue> {
    let request: SweepRequest = serde_wasm_bindgen::from_value(request).map_err(err)?;
    serde_wasm_bindgen::to_value(&auto_sweep(request)).map_err(err)
}

#[wasm_bindgen]
pub fn experiment_csv(experiment: JsValue) -> Result<String, JsValue> {
    let experiment: ExperimentState = serde_wasm_bindgen::from_value(experiment).map_err(err)?;
    Ok(csv_for_state(&experiment))
}

/// Versión del motor (para la barra de estado: "motor OK").
#[wasm_bindgen]
pub fn engine_version() -> String {
    format!("photolab-model {}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn engine_version_is_visible() {
        assert!(super::engine_version().starts_with("photolab-model"));
    }
}
