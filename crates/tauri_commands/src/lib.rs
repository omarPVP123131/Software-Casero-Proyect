//! Comandos `invoke` de PhotoLab. Sustituyen al `physics_adapter` de egui.
//!
//! Todos los comandos son *puros* salvo los tres de disco (`save_session`,
//! `load_session`, `export_file`): reciben un JSON serializable, llaman a
//! `photolab-model` (que llama a `fotoelectrico-physics`) y devuelven JSON.
//! Nada de egui/Macroquad: el render vive en el frontend (canvas 2D).
//!
//! Contrato con el frontend (mismo en Tauri y en WASM):
//! - `physics_evaluate(controlsJson, prefsJson)` → `EvaluateResponse`
//! - `physics_material_catalog()` → `MaterialInfo[]`
//! - `experiment_fit(pointsJson)` → `FitReport | null`
//! - `experiment_sweep(requestJson)` → `SweepPoint[]`
//! - `experiment_csv(experimentJson)` → `string`
//! - `save_session(sessionJson) -> path`, `load_session(path?) -> Session`

//!
//! Los comandos viven en el submódulo `commands` y se reexportan aquí:
//! poner `#[tauri::command]` directamente en la raíz del crate duplica las
//! macros `__cmd__*` (#[macro_export] + ámbito del módulo) y rompe el build.

mod commands;

pub use commands::*;
