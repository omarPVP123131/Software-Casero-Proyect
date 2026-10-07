//! Interfaz desacoplada para el laboratorio fotoeléctrico.
//!
//! La capa física puede alimentar [`PhysicsReadout`] sin que esta crate
//! implemente ni modifique las ecuaciones del simulador.

pub mod state;
mod theme;
mod ui;
mod visualization;

pub use state::{CurvePoint, LabControls, MaterialChoice, PhysicsReadout};

/// Dibuja la interfaz para el frame actual.
pub fn draw(
    ctx: &egui_macroquad::egui::Context,
    controls: &mut LabControls,
    readout: &PhysicsReadout,
    time_seconds: f64,
) {
    theme::apply(ctx);
    ui::draw(ctx, controls, readout, time_seconds);
}

#[cfg(test)]
mod tests;
