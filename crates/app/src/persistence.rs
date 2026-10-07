//! Persistencia ligera de diseño y preferencias visuales.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::state::{AppState, ExperimentPoint, LabTab, UiPreferences};

/// Subconjunto serializable del experimento: puntos, modo y semilla.
/// El estado efímero (mensajes, última ruta) no se guarda.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct PersistedExperiment {
    points: Vec<ExperimentPoint>,
    experimental_mode: bool,
    noise_percent: f32,
    next_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct PersistedUi {
    version: u32,
    preferences: UiPreferences,
    active_tab: LabTab,
    experiment: PersistedExperiment,
    notes: String,
}

impl Default for PersistedUi {
    fn default() -> Self {
        Self {
            version: 1,
            preferences: UiPreferences::default(),
            active_tab: LabTab::Cell,
            experiment: PersistedExperiment::default(),
            notes: String::new(),
        }
    }
}

fn snapshot_of(state: &AppState) -> PersistedUi {
    PersistedUi {
        version: 1,
        preferences: state.preferences.clone(),
        active_tab: state.active_tab,
        experiment: PersistedExperiment {
            points: state.experiment.points.clone(),
            experimental_mode: state.experiment.experimental_mode,
            noise_percent: state.experiment.noise_percent,
            next_seed: state.experiment.next_seed,
        },
        notes: state.notes.clone(),
    }
}

#[derive(Debug, Default)]
pub struct UiPersistence {
    path: Option<PathBuf>,
    last_serialized: Option<String>,
    last_write_seconds: f64,
}

impl UiPersistence {
    /// Carga la última disposición; si el archivo no existe, conserva defaults.
    pub fn restore(state: &mut AppState) -> Self {
        let path = config_path();
        let mut persistence = Self {
            path,
            last_serialized: None,
            last_write_seconds: 0.0,
        };
        let Some(file) = persistence.path.as_ref() else {
            return persistence;
        };
        let Ok(contents) = fs::read_to_string(file) else {
            return persistence;
        };
        let Ok(snapshot) = serde_json::from_str::<PersistedUi>(&contents) else {
            return persistence;
        };
        if snapshot.version != 1 {
            return persistence;
        }
        state.preferences = snapshot.preferences;
        state.active_tab = snapshot.active_tab;
        state.experiment.points = snapshot.experiment.points;
        state.experiment.experimental_mode = snapshot.experiment.experimental_mode;
        state.experiment.noise_percent = snapshot.experiment.noise_percent;
        state.experiment.next_seed = snapshot.experiment.next_seed;
        state.notes = snapshot.notes;
        state.scene_camera.zoom = state.preferences.zoom;
        state.scene_camera.target_zoom = state.preferences.zoom;
        state.scene_camera.pan_x = 0.0;
        state.scene_camera.pan_y = 0.0;
        state.scene_camera.target_pan_x = 0.0;
        state.scene_camera.target_pan_y = 0.0;
        state.inspector = None;
        persistence.last_serialized = serde_json::to_string(&snapshot_of(state)).ok();
        persistence
    }

    /// Guarda solo si cambió el diseño y aplica un debounce pequeño al disco.
    pub fn save_if_changed(&mut self, state: &AppState, now_seconds: f64) {
        if now_seconds - self.last_write_seconds < 0.5 {
            return;
        }
        let snapshot = snapshot_of(state);
        let Ok(serialized) = serde_json::to_string_pretty(&snapshot) else {
            return;
        };
        if self.last_serialized.as_deref() == Some(serialized.as_str()) {
            return;
        }
        let Some(path) = self.path.as_ref() else {
            return;
        };
        let Some(parent) = path.parent() else {
            return;
        };
        if fs::create_dir_all(parent).is_err() || fs::write(path, serialized.as_bytes()).is_err() {
            return;
        }
        self.last_serialized = Some(serialized);
        self.last_write_seconds = now_seconds;
    }
}

#[cfg(target_os = "windows")]
fn config_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|root| PathBuf::from(root).join("PhotoLab").join("ui.json"))
}

#[cfg(target_os = "macos")]
fn config_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("PhotoLab")
            .join("ui.json")
    })
}

#[cfg(all(unix, not(target_os = "macos")))]
fn config_path() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|root| root.join("photolab").join("ui.json"))
}

#[cfg(target_arch = "wasm32")]
fn config_path() -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use super::{PersistedExperiment, PersistedUi, UiPreferences};
    use crate::state::{LabTab, LayoutPreset};

    #[test]
    fn versioned_layout_snapshot_round_trips_panel_preferences() {
        let snapshot = PersistedUi {
            version: 1,
            preferences: UiPreferences {
                layout_preset: LayoutPreset::Analysis,
                controls_panel_open: false,
                readouts_panel_width: 340.0,
                ..UiPreferences::default()
            },
            active_tab: LabTab::Chart,
            ..PersistedUi::default()
        };
        let encoded = serde_json::to_string(&snapshot).unwrap();
        let decoded: PersistedUi = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.active_tab, LabTab::Chart);
        assert_eq!(decoded.preferences.layout_preset, LayoutPreset::Analysis);
        assert!(!decoded.preferences.controls_panel_open);
        assert_eq!(decoded.preferences.readouts_panel_width, 340.0);
    }

    #[test]
    fn missing_preferences_migrate_to_current_defaults() {
        let decoded: PersistedUi =
            serde_json::from_str(r#"{"version":1,"active_tab":"cell"}"#).unwrap();
        assert_eq!(decoded.preferences, UiPreferences::default());
    }

    #[test]
    fn experiment_points_and_notes_round_trip() {
        use crate::state::ExperimentPoint;
        let snapshot = PersistedUi {
            experiment: PersistedExperiment {
                points: vec![ExperimentPoint {
                    wavelength_nm: 400.0,
                    frequency_hz: 7.49e14,
                    stopping_measured_v: 0.74,
                    stopping_ideal_v: 0.74,
                    material_name: "Sodio".to_owned(),
                    noise_percent: 0.0,
                }],
                experimental_mode: true,
                noise_percent: 2.0,
                next_seed: 42,
            },
            notes: "observar el umbral".to_owned(),
            ..PersistedUi::default()
        };
        let encoded = serde_json::to_string(&snapshot).unwrap();
        let decoded: PersistedUi = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.experiment.points.len(), 1);
        assert_eq!(decoded.experiment.points[0].material_name, "Sodio");
        assert!(decoded.experiment.experimental_mode);
        assert_eq!(decoded.notes, "observar el umbral");
    }

    #[test]
    fn old_snapshot_without_experiment_loads_empty_table() {
        // Archivos v1 sin los campos nuevos migran a tabla vacía + notas vacías.
        let decoded: PersistedUi =
            serde_json::from_str(r#"{"version":1,"active_tab":"experiment","preferences":{}}"#)
                .unwrap();
        assert_eq!(decoded.active_tab, LabTab::Experiment);
        assert!(decoded.experiment.points.is_empty());
        assert!(decoded.notes.is_empty());
    }
}
