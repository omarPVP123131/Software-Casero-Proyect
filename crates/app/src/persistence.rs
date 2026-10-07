//! Persistencia ligera de diseño y preferencias visuales.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::state::{AppState, LabTab, UiPreferences};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct PersistedUi {
    version: u32,
    preferences: UiPreferences,
    active_tab: LabTab,
}

impl Default for PersistedUi {
    fn default() -> Self {
        Self {
            version: 1,
            preferences: UiPreferences::default(),
            active_tab: LabTab::Cell,
        }
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
        state.scene_camera.zoom = state.preferences.zoom;
        state.scene_camera.target_zoom = state.preferences.zoom;
        state.scene_camera.pan_x = 0.0;
        state.scene_camera.pan_y = 0.0;
        state.scene_camera.target_pan_x = 0.0;
        state.scene_camera.target_pan_y = 0.0;
        state.inspector = None;
        persistence.last_serialized = serde_json::to_string(&PersistedUi {
            version: 1,
            preferences: state.preferences.clone(),
            active_tab: state.active_tab,
        })
        .ok();
        persistence
    }

    /// Guarda solo si cambió el diseño y aplica un debounce pequeño al disco.
    pub fn save_if_changed(&mut self, state: &AppState, now_seconds: f64) {
        if now_seconds - self.last_write_seconds < 0.5 {
            return;
        }
        let snapshot = PersistedUi {
            version: 1,
            preferences: state.preferences.clone(),
            active_tab: state.active_tab,
        };
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
    use super::{PersistedUi, UiPreferences};
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
}
