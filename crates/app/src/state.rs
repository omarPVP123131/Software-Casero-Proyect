//! Estado de la aplicación y contrato con la capa física.
//!
//! Este módulo solo guarda controles y lecturas para presentar. No contiene
//! ecuaciones ni calcula valores físicos.

use fotoelectrico_engine::RenderStats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialChoice {
    Potassium,
    Sodium,
    Calcium,
    Zinc,
    Copper,
    Platinum,
}

impl MaterialChoice {
    pub const ALL: [Self; 6] = [
        Self::Potassium,
        Self::Sodium,
        Self::Calcium,
        Self::Zinc,
        Self::Copper,
        Self::Platinum,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Potassium => "Potasio",
            Self::Sodium => "Sodio",
            Self::Calcium => "Calcio",
            Self::Zinc => "Zinc",
            Self::Copper => "Cobre",
            Self::Platinum => "Platino",
        }
    }

    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Potassium => "K",
            Self::Sodium => "Na",
            Self::Calcium => "Ca",
            Self::Zinc => "Zn",
            Self::Copper => "Cu",
            Self::Platinum => "Pt",
        }
    }

    /// Función de trabajo en eV (valores estándar de literatura documentados
    /// en `physics_adapter`; el equipo de física puede ajustarlos allí).
    pub const fn work_function_ev(self) -> f64 {
        match self {
            Self::Potassium => 2.29,
            Self::Sodium => 2.36,
            Self::Calcium => 2.87,
            Self::Zinc => 4.30,
            Self::Copper => 4.70,
            Self::Platinum => 5.65,
        }
    }
}

/// Escenarios de demostración de un clic: ajustan varios controles a la vez
/// para exponer sin pelear con sliders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    /// Sodio junto a su umbral (λ0 ≈ 525 nm): emisión apenas posible.
    Threshold,
    /// Sodio 400 nm con frenado total: emisión sí, colección no.
    Blocked,
    /// Potasio frente a platino a 400 nm: uno emite, el otro no.
    Contrast,
}

impl Scenario {
    pub const ALL: [Self; 3] = [Self::Threshold, Self::Blocked, Self::Contrast];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Threshold => "Cruzar el umbral",
            Self::Blocked => "Frenado total",
            Self::Contrast => "K frente a Pt",
        }
    }

    pub const fn detail(self) -> &'static str {
        match self {
            Self::Threshold => "Sodio a 525 nm (junto a λ0): mueve ±20 nm y cruza el umbral.",
            Self::Blocked => "Sodio a 400 nm con V = −2 V: hay emisión pero cero corriente.",
            Self::Contrast => "Potasio frente a platino a 400 nm: uno emite, el otro no.",
        }
    }

    pub fn controls(self) -> ExperimentControls {
        match self {
            Self::Threshold => ExperimentControls {
                material: MaterialChoice::Sodium,
                comparison_material: MaterialChoice::Copper,
                wavelength_nm: 525.0,
                intensity_percent: 60.0,
                applied_voltage_v: 0.0,
                cathode_area_cm2: 1.0,
                quantum_efficiency_percent: 1.0,
            },
            Self::Blocked => ExperimentControls {
                material: MaterialChoice::Sodium,
                comparison_material: MaterialChoice::Copper,
                wavelength_nm: 400.0,
                intensity_percent: 60.0,
                applied_voltage_v: -2.0,
                cathode_area_cm2: 1.0,
                quantum_efficiency_percent: 1.0,
            },
            Self::Contrast => ExperimentControls {
                material: MaterialChoice::Potassium,
                comparison_material: MaterialChoice::Platinum,
                wavelength_nm: 400.0,
                intensity_percent: 60.0,
                applied_voltage_v: 0.0,
                cathode_area_cm2: 1.0,
                quantum_efficiency_percent: 1.0,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabTab {
    #[default]
    Cell,
    Chart,
    Experiment,
    Compare,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeChoice {
    #[default]
    Midnight,
    Slate,
    HighContrast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceMode {
    #[default]
    Balanced,
    Eco,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutKey {
    Space,
    P,
    R,
    N,
    D,
}

impl ShortcutKey {
    pub const ALL: [Self; 5] = [Self::Space, Self::P, Self::R, Self::N, Self::D];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Space => "Espacio",
            Self::P => "P",
            Self::R => "R",
            Self::N => "N",
            Self::D => "D",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutPreferences {
    pub toggle_play: ShortcutKey,
    pub reset_animation: ShortcutKey,
    pub step_frame: ShortcutKey,
    pub diagnostics: ShortcutKey,
}

impl Default for ShortcutPreferences {
    fn default() -> Self {
        Self {
            toggle_play: ShortcutKey::Space,
            reset_animation: ShortcutKey::R,
            step_frame: ShortcutKey::N,
            diagnostics: ShortcutKey::D,
        }
    }
}

/// Entradas seleccionadas por el usuario; son valores de interfaz, no un modelo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExperimentControls {
    pub material: MaterialChoice,
    pub comparison_material: MaterialChoice,
    pub wavelength_nm: f32,
    pub intensity_percent: f32,
    pub applied_voltage_v: f32,
    /// Área emisora del cátodo en cm² (modelo de corriente, calibrable).
    pub cathode_area_cm2: f32,
    /// Eficiencia cuántica en % (modelo de corriente, calibrable).
    pub quantum_efficiency_percent: f32,
}

impl Default for ExperimentControls {
    fn default() -> Self {
        Self {
            material: MaterialChoice::Sodium,
            comparison_material: MaterialChoice::Copper,
            wavelength_nm: 400.0,
            intensity_percent: 55.0,
            applied_voltage_v: 0.0,
            cathode_area_cm2: 1.0,
            quantum_efficiency_percent: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerVisibility {
    pub photons: bool,
    pub electrons: bool,
    pub arrows: bool,
    pub labels: bool,
    pub trails: bool,
    pub grid: bool,
    pub ruler: bool,
    pub electrode_texture: bool,
    pub demo_electrons: bool,
}

impl Default for LayerVisibility {
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
            demo_electrons: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartViewState {
    pub zoom: f32,
    pub pan: f32,
    pub show_data_table: bool,
}

impl Default for ChartViewState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: 0.0,
            show_data_table: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutPreset {
    #[default]
    Workbench,
    Analysis,
    Presentation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPreferences {
    pub theme: ThemeChoice,
    pub performance: PerformanceMode,
    pub simple_mode: bool,
    pub presentation_mode: bool,
    pub layout_preset: LayoutPreset,
    pub controls_on_left: bool,
    pub controls_panel_open: bool,
    pub readouts_panel_open: bool,
    pub controls_panel_width: f32,
    pub readouts_panel_width: f32,
    pub show_hitboxes: bool,
    pub font_scale: f32,
    pub zoom: f32,
    pub layers: LayerVisibility,
    pub shortcuts: ShortcutPreferences,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::Midnight,
            performance: PerformanceMode::Balanced,
            simple_mode: false,
            presentation_mode: false,
            layout_preset: LayoutPreset::Workbench,
            controls_on_left: true,
            controls_panel_open: true,
            readouts_panel_open: true,
            controls_panel_width: 286.0,
            readouts_panel_width: 282.0,
            show_hitboxes: true,
            font_scale: 1.0,
            zoom: 1.0,
            layers: LayerVisibility::default(),
            shortcuts: ShortcutPreferences::default(),
        }
    }
}

/// Punto que la capa física puede entregar a la gráfica.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CurvePoint {
    pub wavelength_nm: f64,
    pub max_kinetic_energy_ev: f64,
}

/// Datos que la UI muestra, calculados por `fotoelectrico-physics`.
/// `None` solo aparece antes del primer `refresh_physics`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PhysicsReadout {
    pub emission_possible: Option<bool>,
    pub collected_possible: Option<bool>,
    pub frequency_hz: Option<f64>,
    pub photon_energy_ev: Option<f64>,
    pub work_function_ev: Option<f64>,
    pub threshold_frequency_hz: Option<f64>,
    pub threshold_wavelength_nm: Option<f64>,
    pub max_kinetic_energy_ev: Option<f64>,
    pub stopping_potential_v: Option<f64>,
    pub photon_flux_density_per_m2_s: Option<f64>,
    pub electron_max_speed_m_s: Option<f64>,
    /// Fotocorriente estimada en amperios (modelo demo: `I = e·Φ·A·QE·g(V)`).
    pub photocurrent_a: Option<f64>,
    /// Fracción de colección 0..1 (rampa de frenado continua).
    pub collection_factor: Option<f64>,
    pub kinetic_energy_curve: Vec<CurvePoint>,
}

/// Un punto medido del experimento V0 contra f.
///
/// Guarda el V0 ideal y el medido (con ruido si el modo experimental está
/// activo). El ruido se genera una sola vez al capturar el punto, así el
/// valor es estable entre frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExperimentPoint {
    pub wavelength_nm: f32,
    pub frequency_hz: f64,
    pub stopping_measured_v: f64,
    pub stopping_ideal_v: f64,
    pub material_name: String,
    pub noise_percent: f32,
}

/// Estado del experimento V0 contra f y su configuración.
#[derive(Debug, Clone, PartialEq)]
pub struct ExperimentState {
    pub points: Vec<ExperimentPoint>,
    /// false = ideal (V0 exacto), true = experimental (ruido ±`noise_percent` %).
    pub experimental_mode: bool,
    pub noise_percent: f32,
    /// Semilla del generador determinista (estable entre frames).
    pub next_seed: u64,
    /// Último mensaje de captura/exportación para mostrar en la UI.
    pub status: String,
    /// Última ruta de exportación (para confirmar al usuario).
    pub last_export_path: String,
}

impl Default for ExperimentState {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            experimental_mode: false,
            noise_percent: 2.0,
            next_seed: 0x9E37_79B9_7F4A_7C15,
            status: String::new(),
            last_export_path: String::new(),
        }
    }
}

/// Máximo de puntos del experimento (suficiente para un buen ajuste).
pub const MAX_EXPERIMENT_POINTS: usize = 24;

impl ExperimentState {
    /// Intenta registrar el punto actual. Falla con mensaje si no hay emisión
    /// o si ya existe un punto con la misma λ (evita duplicados verticales).
    pub fn try_add(
        &mut self,
        material: MaterialChoice,
        wavelength_nm: f32,
        frequency_hz: f64,
        stopping_ideal_v: f64,
        emits: bool,
        noise_applied_v: f64,
    ) -> Result<(), String> {
        if !emits {
            return Err("Sin emisión a esta λ (λ > λ0): baja la longitud de onda.".to_owned());
        }
        if self.points.len() >= MAX_EXPERIMENT_POINTS {
            return Err(format!(
                "Tabla llena ({MAX_EXPERIMENT_POINTS} puntos). Quita alguno."
            ));
        }
        // El ajuste V0 = m·f + b supone un solo Φ: no mezclar materiales.
        if let Some(first) = self.points.first() {
            if first.material_name != material.name() {
                return Err(format!(
                    "La tabla es de {}: vacíala para medir otro material.",
                    first.material_name
                ));
            }
        }
        let duplicate = self
            .points
            .iter()
            .any(|p| (p.wavelength_nm - wavelength_nm).abs() < 0.5);
        if duplicate {
            return Err("Ya hay un punto con esa λ: cámbiala al menos 1 nm.".to_owned());
        }
        self.points.push(ExperimentPoint {
            wavelength_nm,
            frequency_hz,
            stopping_measured_v: noise_applied_v,
            stopping_ideal_v,
            material_name: material.name().to_owned(),
            noise_percent: if self.experimental_mode {
                self.noise_percent
            } else {
                0.0
            },
        });
        self.status = format!(
            "Punto {} agregado: λ = {:.0} nm, V0 = {:.3} V{}.",
            self.points.len(),
            wavelength_nm,
            noise_applied_v,
            if self.experimental_mode {
                format!(" (ruido ±{:.1}%)", self.noise_percent)
            } else {
                " (ideal)".to_owned()
            }
        );
        Ok(())
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.points.len() {
            self.points.remove(index);
            self.status = format!("Punto quitado. Quedan {}.", self.points.len());
        }
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.status = "Tabla del experimento vaciada.".to_owned();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectedObject {
    Canvas,
    PhotonBeam,
    Cathode,
    Anode,
    ElectronLayer,
    FieldArrows,
}

impl InspectedObject {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Canvas => "Canvas de la celda",
            Self::PhotonBeam => "Haz de fotones",
            Self::Cathode => "Cátodo",
            Self::Anode => "Ánodo",
            Self::ElectronLayer => "Capa de electrones",
            Self::FieldArrows => "Flechas de polaridad",
        }
    }

    pub const fn screen_hint(self) -> &'static str {
        match self {
            Self::Canvas => "área completa de la celda",
            Self::PhotonBeam => "haz visible a la izquierda de los electrodos",
            Self::Cathode => "placa izquierda del canvas",
            Self::Anode => "placa derecha del canvas",
            Self::ElectronLayer => "espacio entre electrodos",
            Self::FieldArrows => "centro del espacio entre electrodos",
        }
    }

    /// Punto de anclaje normalizado para centrar el objeto en la vista.
    pub const fn anchor_fraction(self) -> (f32, f32) {
        match self {
            Self::Canvas => (0.5, 0.5),
            Self::PhotonBeam => (0.18, 0.53),
            Self::Cathode => (0.33, 0.53),
            Self::Anode => (0.71, 0.53),
            Self::ElectronLayer | Self::FieldArrows => (0.52, 0.53),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneCamera {
    pub zoom: f32,
    pub target_zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub target_pan_x: f32,
    pub target_pan_y: f32,
}

impl Default for SceneCamera {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            target_zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            target_pan_x: 0.0,
            target_pan_y: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockPanel {
    Controls,
    Readouts,
}

impl From<InspectedObject> for fotoelectrico_engine::SceneObject {
    fn from(object: InspectedObject) -> Self {
        match object {
            InspectedObject::Canvas => Self::Canvas,
            InspectedObject::PhotonBeam => Self::PhotonBeam,
            InspectedObject::Cathode => Self::Cathode,
            InspectedObject::Anode => Self::Anode,
            InspectedObject::ElectronLayer => Self::ElectronLayer,
            InspectedObject::FieldArrows => Self::FieldArrows,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    pub elapsed_seconds: f64,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub controls: ExperimentControls,
    pub readout: PhysicsReadout,
    pub comparison_readout: PhysicsReadout,
    pub chart: ChartViewState,
    pub experiment: ExperimentState,
    pub preferences: UiPreferences,
    pub active_tab: LabTab,
    pub animation_running: bool,
    pub animation_speed: f32,
    pub animation_time_seconds: f64,
    pub session_seconds: f64,
    pub show_preferences: bool,
    pub show_shortcuts: bool,
    pub show_command_palette: bool,
    pub command_search: String,
    pub show_diagnostics: bool,
    pub inspector: Option<InspectedObject>,
    inspector_anchor_fraction: Option<(f32, f32)>,
    pub scene_camera: SceneCamera,
    pub active_drawer: Option<DockPanel>,
    pub notes: String,
    pub render_stats: RenderStats,
    pub history: Vec<HistoryEntry>,
    pub undo_stack: Vec<ExperimentControls>,
    pub redo_stack: Vec<ExperimentControls>,
    last_undo_group: Option<(String, f64)>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut state = Self {
            controls: ExperimentControls::default(),
            readout: PhysicsReadout::default(),
            comparison_readout: PhysicsReadout::default(),
            chart: ChartViewState::default(),
            experiment: ExperimentState::default(),
            preferences: UiPreferences::default(),
            active_tab: LabTab::Cell,
            animation_running: true,
            animation_speed: 1.0,
            animation_time_seconds: 0.0,
            session_seconds: 0.0,
            show_preferences: false,
            show_shortcuts: false,
            show_command_palette: false,
            command_search: String::new(),
            show_diagnostics: false,
            inspector: None,
            inspector_anchor_fraction: None,
            scene_camera: SceneCamera::default(),
            active_drawer: None,
            notes: String::new(),
            render_stats: RenderStats::default(),
            history: vec![HistoryEntry {
                elapsed_seconds: 0.0,
                text: "Sesión iniciada · motor físico conectado (modelo ideal)".to_owned(),
            }],
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_undo_group: None,
        };
        crate::physics_adapter::refresh_state(&mut state);
        state
    }
}

impl AppState {
    pub fn tick(&mut self, delta_seconds: f64) {
        let delta = delta_seconds.clamp(0.0, 0.1);
        self.session_seconds += delta;
        if self.animation_running {
            self.animation_time_seconds += delta * self.animation_speed.clamp(0.1, 3.0) as f64;
        }
        let ease = 1.0 - (-8.0 * delta as f32).exp();
        self.scene_camera.zoom += (self.scene_camera.target_zoom - self.scene_camera.zoom) * ease;
        self.scene_camera.pan_x +=
            (self.scene_camera.target_pan_x - self.scene_camera.pan_x) * ease;
        self.scene_camera.pan_y +=
            (self.scene_camera.target_pan_y - self.scene_camera.pan_y) * ease;
        if (self.scene_camera.target_zoom - self.scene_camera.zoom).abs() < 0.001 {
            self.scene_camera.zoom = self.scene_camera.target_zoom;
        }
        if (self.scene_camera.target_pan_x - self.scene_camera.pan_x).abs() < 0.001 {
            self.scene_camera.pan_x = self.scene_camera.target_pan_x;
        }
        if (self.scene_camera.target_pan_y - self.scene_camera.pan_y).abs() < 0.001 {
            self.scene_camera.pan_y = self.scene_camera.target_pan_y;
        }
    }

    pub fn select_object(&mut self, object: InspectedObject) {
        self.select_object_at(object, object.anchor_fraction());
    }

    /// Selecciona un objeto usando su posición real en la escena como foco de cámara.
    pub fn select_object_at(&mut self, object: InspectedObject, anchor_fraction: (f32, f32)) {
        let changed = self.inspector != Some(object);
        self.inspector = Some(object);
        self.inspector_anchor_fraction = Some((
            anchor_fraction.0.clamp(0.0, 1.0),
            anchor_fraction.1.clamp(0.0, 1.0),
        ));
        if !self.preferences.readouts_panel_open {
            self.active_drawer = Some(DockPanel::Readouts);
        }
        self.sync_camera_target();
        if changed {
            self.push_history(format!("Inspector seleccionado: {}", object.label()));
        }
    }

    /// El inspector solo se cierra si el usuario oculta su capa.
    /// Los cambios físicos (sin emisión, bloqueo por frenado) jamás cierran
    /// el inspector: el panel explica el porqué en vez de "desconectar".
    pub fn reconcile_inspector_visibility(&mut self) {
        let is_visible = match self.inspector {
            Some(InspectedObject::Canvas | InspectedObject::Cathode | InspectedObject::Anode) => {
                true
            }
            Some(InspectedObject::PhotonBeam) => self.preferences.layers.photons,
            Some(InspectedObject::ElectronLayer) => self.preferences.layers.electrons,
            Some(InspectedObject::FieldArrows) => self.preferences.layers.arrows,
            None => true,
        };
        if !is_visible {
            self.close_inspector();
        }
    }

    pub fn sync_camera_target(&mut self) {
        let zoom = match self.inspector {
            Some(InspectedObject::Canvas) => 1.0,
            Some(_) => self.preferences.zoom.clamp(0.7, 1.8).max(1.25),
            None => self.preferences.zoom.clamp(0.7, 1.8),
        };
        self.scene_camera.target_zoom = zoom;
        if let Some(object) = self
            .inspector
            .filter(|object| *object != InspectedObject::Canvas)
        {
            let (anchor_x, anchor_y) = self
                .inspector_anchor_fraction
                .unwrap_or_else(|| object.anchor_fraction());
            self.scene_camera.target_pan_x = -(anchor_x - 0.5) * zoom;
            self.scene_camera.target_pan_y = -(anchor_y - 0.5) * zoom;
        } else {
            self.scene_camera.target_pan_x = 0.0;
            self.scene_camera.target_pan_y = 0.0;
        }
    }

    pub fn close_inspector(&mut self) {
        if self.inspector.take().is_some() {
            self.inspector_anchor_fraction = None;
            self.sync_camera_target();
            self.push_history("Inspector cerrado · vista general restaurada");
        }
    }

    pub fn set_layout_preset(&mut self, preset: LayoutPreset) {
        self.preferences.layout_preset = preset;
        match preset {
            LayoutPreset::Workbench => {
                self.preferences.controls_panel_open = true;
                self.preferences.readouts_panel_open = true;
                self.preferences.presentation_mode = false;
                self.active_tab = LabTab::Cell;
            }
            LayoutPreset::Analysis => {
                self.preferences.controls_panel_open = false;
                self.preferences.readouts_panel_open = true;
                self.preferences.presentation_mode = false;
                self.active_tab = LabTab::Chart;
            }
            LayoutPreset::Presentation => {
                self.preferences.controls_panel_open = false;
                self.preferences.readouts_panel_open = false;
                self.preferences.presentation_mode = true;
                self.active_tab = LabTab::Cell;
            }
        }
        self.active_drawer = None;
        self.push_history(format!("Diseño aplicado: {preset:?}"));
    }

    pub fn reset_animation(&mut self) {
        self.animation_time_seconds = 0.0;
        self.push_history("Animación reiniciada");
    }

    pub fn step_frame(&mut self) {
        self.animation_time_seconds += 1.0 / 30.0;
    }

    pub fn record_control_change(&mut self, before: ExperimentControls) {
        let after = self.controls;
        if before == after {
            return;
        }
        let text = control_diff(before, after);
        let group_key = text
            .split_once(" ->")
            .map(|(key, _)| key.to_owned())
            .unwrap_or_else(|| text.clone());
        let new_undo_group = self
            .last_undo_group
            .as_ref()
            .map(|(label, time)| label != &group_key || self.session_seconds - time > 0.45)
            .unwrap_or(true);
        if new_undo_group {
            self.undo_stack.push(before);
            if self.undo_stack.len() > 60 {
                self.undo_stack.remove(0);
            }
        }
        self.redo_stack.clear();
        self.last_undo_group = Some((group_key.clone(), self.session_seconds));
        if let Some(last) = self.history.last_mut() {
            let last_group_key = last.text.split_once(" ->").map(|(key, _)| key.to_owned());
            if last_group_key.as_deref() == Some(group_key.as_str())
                && self.session_seconds - last.elapsed_seconds < 0.45
            {
                last.elapsed_seconds = self.session_seconds;
                last.text = text;
                return;
            }
        }
        self.push_history(text);
    }

    pub fn undo(&mut self) {
        if let Some(previous) = self.undo_stack.pop() {
            self.redo_stack.push(self.controls);
            self.controls = previous;
            self.last_undo_group = None;
            self.push_history("Deshacer cambio de controles");
            self.refresh_physics();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.controls);
            self.controls = next;
            self.last_undo_group = None;
            self.push_history("Rehacer cambio de controles");
            self.refresh_physics();
        }
    }

    pub fn reset_controls(&mut self) {
        let before = self.controls;
        self.controls = ExperimentControls::default();
        self.record_control_change(before);
    }

    /// Aplica un escenario de demo: sustituye los controles, lo registra en
    /// el historial y recalcula la física. El llamador decide si cuenta para undo.
    pub fn apply_scenario(&mut self, scenario: Scenario) {
        self.controls = scenario.controls();
        self.last_undo_group = None;
        self.redo_stack.clear();
        self.push_history(format!("Escenario aplicado: {}", scenario.label()));
        self.refresh_physics();
    }

    /// Recalcula las lecturas con el motor físico. Llamar tras cambiar
    /// controles y una vez por frame en el loop principal.
    pub fn refresh_physics(&mut self) {
        crate::physics_adapter::refresh_state(self);
        self.reconcile_inspector_visibility();
    }

    pub fn push_history(&mut self, text: impl Into<String>) {
        self.history.push(HistoryEntry {
            elapsed_seconds: self.session_seconds,
            text: text.into(),
        });
        if self.history.len() > 120 {
            self.history.remove(0);
        }
    }
}

/// ¿Deben verse electrones en la escena? Emisión + colección por voltaje
/// + intensidad. Si aún no hay lectura, usa la demo.
pub fn electrons_visible_in_scene(state: &AppState) -> bool {
    if let Some(collected) = state.readout.collected_possible {
        if !collected {
            return false;
        }
        return state.controls.intensity_percent > 0.1;
    }
    if let Some(emission) = state.readout.emission_possible {
        return emission && state.controls.intensity_percent > 0.1;
    }
    state.preferences.layers.demo_electrons
}

fn control_diff(before: ExperimentControls, after: ExperimentControls) -> String {
    if before.material != after.material {
        format!("Material del cátodo -> {}", after.material.name())
    } else if before.comparison_material != after.comparison_material {
        format!(
            "Material de comparación -> {}",
            after.comparison_material.name()
        )
    } else if (before.wavelength_nm - after.wavelength_nm).abs() > 0.01 {
        format!("Longitud de onda -> {:.0} nm", after.wavelength_nm)
    } else if (before.intensity_percent - after.intensity_percent).abs() > 0.01 {
        format!("Intensidad -> {:.0}%", after.intensity_percent)
    } else if (before.applied_voltage_v - after.applied_voltage_v).abs() > 0.01 {
        format!("Voltaje aplicado -> {:+.1} V", after.applied_voltage_v)
    } else if (before.cathode_area_cm2 - after.cathode_area_cm2).abs() > 0.001 {
        format!("Área del cátodo -> {:.2} cm²", after.cathode_area_cm2)
    } else if (before.quantum_efficiency_percent - after.quantum_efficiency_percent).abs() > 0.01 {
        format!(
            "Eficiencia cuántica -> {:.1}%",
            after.quantum_efficiency_percent
        )
    } else {
        "Controles actualizados".to_owned()
    }
}
