//! Persistencia ligera de diseño y preferencias visuales.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::state::{
    AppState, ExperimentControls, ExperimentPoint, LabTab, MaterialChoice, SavedFit, UiPreferences,
};

/// Subconjunto serializable del experimento: puntos, modo, semilla y ajustes.
/// El estado efímero (mensajes, última ruta, selección, zoom) no se guarda.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct PersistedExperiment {
    points: Vec<ExperimentPoint>,
    experimental_mode: bool,
    noise_percent: f32,
    next_seed: u64,
    saved_fits: Vec<SavedFit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct PersistedControls {
    material: MaterialChoice,
    comparison_material: MaterialChoice,
    wavelength_nm: f32,
    intensity_percent: f32,
    applied_voltage_v: f32,
    cathode_area_cm2: f32,
    quantum_efficiency_percent: f32,
}

impl Default for PersistedControls {
    fn default() -> Self {
        let controls = ExperimentControls::default();
        Self {
            material: controls.material,
            comparison_material: controls.comparison_material,
            wavelength_nm: controls.wavelength_nm,
            intensity_percent: controls.intensity_percent,
            applied_voltage_v: controls.applied_voltage_v,
            cathode_area_cm2: controls.cathode_area_cm2,
            quantum_efficiency_percent: controls.quantum_efficiency_percent,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct PersistedUi {
    version: u32,
    preferences: UiPreferences,
    active_tab: LabTab,
    controls: PersistedControls,
    experiment: PersistedExperiment,
    notes: String,
}

impl Default for PersistedUi {
    fn default() -> Self {
        Self {
            version: 1,
            preferences: UiPreferences::default(),
            active_tab: LabTab::Cell,
            controls: PersistedControls::default(),
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
        controls: PersistedControls {
            material: state.controls.material,
            comparison_material: state.controls.comparison_material,
            wavelength_nm: state.controls.wavelength_nm,
            intensity_percent: state.controls.intensity_percent,
            applied_voltage_v: state.controls.applied_voltage_v,
            cathode_area_cm2: state.controls.cathode_area_cm2,
            quantum_efficiency_percent: state.controls.quantum_efficiency_percent,
        },
        experiment: PersistedExperiment {
            points: state.experiment.points.clone(),
            experimental_mode: state.experiment.experimental_mode,
            noise_percent: state.experiment.noise_percent,
            next_seed: state.experiment.next_seed,
            saved_fits: state.experiment.saved_fits.clone(),
        },
        notes: state.notes.clone(),
    }
}

/// Aplica un snapshot al estado (ruta real que usa `restore`, testeable).
fn apply_snapshot(state: &mut AppState, snapshot: &PersistedUi) {
    state.preferences = snapshot.preferences.clone();
    state.active_tab = snapshot.active_tab;
    state.controls.material = snapshot.controls.material;
    state.controls.comparison_material = snapshot.controls.comparison_material;
    state.controls.wavelength_nm = snapshot.controls.wavelength_nm;
    state.controls.intensity_percent = snapshot.controls.intensity_percent;
    state.controls.applied_voltage_v = snapshot.controls.applied_voltage_v;
    state.controls.cathode_area_cm2 = snapshot.controls.cathode_area_cm2;
    state.controls.quantum_efficiency_percent = snapshot.controls.quantum_efficiency_percent;
    state.experiment.points = snapshot.experiment.points.clone();
    state.experiment.experimental_mode = snapshot.experiment.experimental_mode;
    state.experiment.noise_percent = snapshot.experiment.noise_percent;
    state.experiment.next_seed = snapshot.experiment.next_seed;
    state.experiment.saved_fits = snapshot.experiment.saved_fits.clone();
    state.notes = snapshot.notes.clone();
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
        apply_snapshot(state, &snapshot);
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
    use super::{
        apply_snapshot, PersistedControls, PersistedExperiment, PersistedUi, UiPreferences,
    };
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
                saved_fits: Vec::new(),
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

    #[test]
    fn controls_round_trip_and_restore() {
        use crate::state::{AppState, MaterialChoice};
        let snapshot = PersistedUi {
            controls: PersistedControls {
                material: MaterialChoice::Potassium,
                comparison_material: MaterialChoice::Platinum,
                wavelength_nm: 300.0,
                intensity_percent: 70.0,
                applied_voltage_v: -2.0,
                cathode_area_cm2: 2.5,
                quantum_efficiency_percent: 25.0,
            },
            ..PersistedUi::default()
        };
        let encoded = serde_json::to_string(&snapshot).unwrap();
        let decoded: PersistedUi = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.controls.material, MaterialChoice::Potassium);
        assert_eq!(decoded.controls.wavelength_nm, 300.0);
        assert_eq!(decoded.controls.quantum_efficiency_percent, 25.0);

        // Archivos sin "controls" migran a los defaults del experimento.
        let legacy: PersistedUi =
            serde_json::from_str(r#"{"version":1,"active_tab":"cell"}"#).unwrap();
        assert_eq!(
            legacy.controls.wavelength_nm,
            crate::state::ExperimentControls::default().wavelength_nm
        );

        // La ruta real: snapshot → estado, como hace restore().
        let mut app = AppState::default();
        apply_snapshot(&mut app, &decoded);
        assert_eq!(app.controls.material, MaterialChoice::Potassium);
        assert_eq!(app.controls.wavelength_nm, 300.0);
        app.refresh_physics();
        assert_eq!(app.readout.emission_possible, Some(true));
    }

    #[test]
    fn cathode_presets_hold_documented_efficiencies() {
        use crate::state::CathodePreset;
        assert_eq!(CathodePreset::Bialkali.quantum_efficiency_percent(), 25.0);
        assert_eq!(
            CathodePreset::Multialkali.quantum_efficiency_percent(),
            20.0
        );
        assert_eq!(CathodePreset::Demo.quantum_efficiency_percent(), 1.0);
    }
}
