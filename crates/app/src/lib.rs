//! Interfaz egui conectada al motor físico y al renderer Macroquad.

pub mod format;
pub mod math;
mod persistence;
pub mod physics_adapter;
pub mod state;
mod theme;
mod ui;
mod visualization;

pub use persistence::UiPersistence;
pub use physics_adapter::{
    electron_speed_scale, photoelectron_strength, refresh_state, REFERENCE_IRRADIANCE_W_M2,
};
pub use state::{
    AppState, CathodePreset, CurvePoint, DockPanel, ExperimentControls, ExperimentPoint,
    ExperimentState, HistoryEntry, InspectedObject, LabTab, LayoutPreset, MaterialChoice,
    PhysicsReadout, SavedFit, Scenario, MAX_EXPERIMENT_POINTS,
};

/// Región del canvas que `engine` debe dibujar después de construir la UI.
#[derive(Debug, Clone, Copy)]
pub struct UiFrame {
    pub scene_rect: Option<egui_macroquad::egui::Rect>,
    /// Conversión desde puntos egui a unidades lógicas de Macroquad.
    pub coordinate_scale: f32,
}

impl Default for UiFrame {
    fn default() -> Self {
        Self {
            scene_rect: None,
            coordinate_scale: 1.0,
        }
    }
}

/// Construye los widgets egui y devuelve el espacio reservado para la celda.
pub fn draw_ui(ctx: &egui_macroquad::egui::Context, state: &mut AppState) -> UiFrame {
    theme::apply(ctx, state.preferences.theme, state.preferences.font_scale);
    ui::draw(ctx, state)
}

#[cfg(test)]
mod tests;
