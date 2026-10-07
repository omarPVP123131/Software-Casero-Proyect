//! Renderer visual basado en Macroquad, desacoplado de la UI egui y de la física.

#![forbid(unsafe_code)]

pub mod layers;
pub mod renderer;
pub mod scene;

pub use renderer::Renderer;
pub use scene::{RenderLayers, RenderQuality, RenderStats, SceneFrame, SceneObject, Viewport};
