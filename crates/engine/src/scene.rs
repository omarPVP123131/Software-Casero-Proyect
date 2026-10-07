//! View-modelos de renderizado. No contienen ecuaciones físicas.

/// Rectángulo del canvas en píxeles de ventana.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn is_visible(self) -> bool {
        self.width > 16.0 && self.height > 16.0
    }
}

/// Nivel visual de detalle. `Performance` reduce efectos y partículas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderQuality {
    #[default]
    Balanced,
    Performance,
}

/// Capas visuales que el usuario puede activar por separado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderLayers {
    pub photons: bool,
    pub electrons: bool,
    pub arrows: bool,
    pub labels: bool,
    pub trails: bool,
    pub grid: bool,
    pub ruler: bool,
    pub electrode_texture: bool,
}

impl Default for RenderLayers {
    fn default() -> Self {
        Self {
            photons: true,
            electrons: true,
            arrows: true,
            labels: true,
            trails: true,
            grid: true,
            ruler: false,
            electrode_texture: true,
        }
    }
}

/// Objetos visuales seleccionables; no representan entidades físicas integradas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneObject {
    Canvas,
    PhotonBeam,
    Cathode,
    Anode,
    ElectronLayer,
    FieldArrows,
}

/// Datos visuales que `app` entrega al renderer, calculados por
/// `fotoelectrico-physics` (emisión, colección por voltaje, Kmax).
///
/// `emission_possible` es emisión potencial (hf > Φ).
/// `collection_possible` es si llegan al ánodo (no bloqueados por frenado).
/// Si son `None` (arranque), el motor puede mostrar demo ilustrativa.
#[derive(Debug, Clone, Copy)]
pub struct SceneFrame<'a> {
    pub viewport: Viewport,
    pub wavelength_nm: f32,
    pub intensity_percent: f32,
    pub visual_voltage_v: f32,
    pub material_label: &'a str,
    pub time_seconds: f64,
    pub animation_running: bool,
    pub animation_speed: f32,
    pub demo_electrons: bool,
    pub emission_possible: Option<bool>,
    pub collection_possible: Option<bool>,
    pub k_max_ev: Option<f64>,
    pub electron_max_speed_m_s: Option<f64>,
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub selected_object: Option<SceneObject>,
    pub show_hitboxes: bool,
    pub font_scale: f32,
    pub layers: RenderLayers,
    pub quality: RenderQuality,
    pub high_contrast: bool,
}

/// Estadísticas del frame para el panel de diagnóstico gráfico.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RenderStats {
    pub fps: u32,
    pub frame_time_ms: f32,
    pub photons_drawn: usize,
    pub electrons_drawn: usize,
}
