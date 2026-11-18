//! Estado de sesión: controles, preferencias, experimento y mercancías
//! serializables. Este es el "contrato SceneFrame" del proyecto Tauri:
//! un único objeto `Session` que se guarda/carga como `.photolab.json`.

use serde::{Deserialize, Serialize, Serializer};
use ts_rs::TS;

use fotoelectrico_physics::Material;

use crate::experiments::de_u64_from_string;

fn ser_u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

/// Versión del esquema de `.photolab.json` (migraciones se hacen en load).
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(TS)]
#[ts(export, export_to = "MaterialChoice.ts")]
pub enum MaterialChoice {
    Potassium,
    #[default]
    Sodium,
    Calcium,
    Zinc,
    Copper,
    Platinum,
    /// Φ definida por el usuario (eV). El valor vive en los controles.
    Custom,
}

impl MaterialChoice {
    pub const ALL: [Self; 7] = [
        Self::Potassium,
        Self::Sodium,
        Self::Calcium,
        Self::Zinc,
        Self::Copper,
        Self::Platinum,
        Self::Custom,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Potassium => "potassium",
            Self::Sodium => "sodium",
            Self::Calcium => "calcium",
            Self::Zinc => "zinc",
            Self::Copper => "copper",
            Self::Platinum => "platinum",
            Self::Custom => "custom",
        }
    }

    pub const fn name_es(self) -> &'static str {
        match self {
            Self::Potassium => "Potasio",
            Self::Sodium => "Sodio",
            Self::Calcium => "Calcio",
            Self::Zinc => "Zinc",
            Self::Copper => "Cobre",
            Self::Platinum => "Platino",
            Self::Custom => "Personalizado",
        }
    }

    pub const fn name_en(self) -> &'static str {
        match self {
            Self::Potassium => "Potassium",
            Self::Sodium => "Sodium",
            Self::Calcium => "Calcium",
            Self::Zinc => "Zinc",
            Self::Copper => "Copper",
            Self::Platinum => "Platinum",
            Self::Custom => "Custom",
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
            Self::Custom => "Φ",
        }
    }

    /// Función de trabajo de literatura (eV). `custom_phi_ev` solo se usa
    /// cuando la variante es `Custom`.
    pub fn work_function_ev(self, custom_phi_ev: f64) -> f64 {
        match self {
            Self::Potassium => 2.29,
            Self::Sodium => 2.36,
            Self::Calcium => 2.87,
            Self::Zinc => 4.30,
            Self::Copper => 4.70,
            Self::Platinum => 5.65,
            Self::Custom => {
                if custom_phi_ev.is_finite() && (1.0..=6.5).contains(&custom_phi_ev) {
                    custom_phi_ev
                } else {
                    3.0
                }
            }
        }
    }

    pub const fn source(self) -> &'static str {
        match self {
            Self::Potassium => {
                "Φ ≈ 2.29 eV (valor estándar de literatura; depende de la superficie)."
            }
            Self::Sodium => "Φ ≈ 2.36 eV (valor estándar de literatura; depende de la superficie).",
            Self::Calcium => {
                "Φ ≈ 2.87 eV (valor estándar de literatura; depende de la superficie)."
            }
            Self::Zinc => "Φ ≈ 4.30 eV (valor estándar de literatura; depende de la superficie).",
            Self::Copper => "Φ ≈ 4.70 eV (valor estándar de literatura; depende de la superficie).",
            Self::Platinum => {
                "Φ ≈ 5.65 eV (valor estándar de literatura; depende de la superficie)."
            }
            Self::Custom => "Φ definida por el usuario: útil para explorar, no para citar.",
        }
    }

    /// Construye el `Material` del motor físico.
    pub fn to_physics(self, custom_phi_ev: f64) -> Material {
        Material::new(self.name_es(), self.work_function_ev(custom_phi_ev))
    }
}

/// Entradas seleccionadas por el usuario. Valores de interfaz, no del modelo.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[derive(TS)]
#[ts(export, export_to = "ExperimentControls.ts")]
pub struct ExperimentControls {
    pub material: MaterialChoice,
    pub comparison_material: MaterialChoice,
    /// Solo si `material == Custom`.
    pub custom_phi_ev: f64,
    /// Solo si `comparison_material == Custom`.
    pub comparison_custom_phi_ev: f64,
    pub wavelength_nm: f64,
    pub intensity_percent: f64,
    pub applied_voltage_v: f64,
    /// Área emisora del cátodo en cm² (modelo de corriente, calibrable).
    pub cathode_area_cm2: f64,
    /// Eficiencia cuántica en % (modelo de corriente, calibrable).
    pub quantum_efficiency_percent: f64,
}

impl Default for ExperimentControls {
    fn default() -> Self {
        Self {
            material: MaterialChoice::Sodium,
            comparison_material: MaterialChoice::Copper,
            custom_phi_ev: 3.0,
            comparison_custom_phi_ev: 3.0,
            wavelength_nm: 400.0,
            intensity_percent: 55.0,
            applied_voltage_v: 0.0,
            cathode_area_cm2: 1.0,
            quantum_efficiency_percent: 1.0,
        }
    }
}

impl ExperimentControls {
    /// Saneo total: NaN/inf y fuera de rango se saturan a valores seguros.
    pub fn sanitized(mut self) -> Self {
        fn clean(value: f64, fallback: f64, min: f64, max: f64) -> f64 {
            if !value.is_finite() {
                return fallback;
            }
            value.clamp(min, max)
        }
        self.wavelength_nm = clean(self.wavelength_nm, 400.0, 180.0, 900.0);
        self.intensity_percent = clean(self.intensity_percent, 55.0, 0.0, 100.0);
        self.applied_voltage_v = clean(self.applied_voltage_v, 0.0, -5.0, 5.0);
        self.cathode_area_cm2 = clean(self.cathode_area_cm2, 1.0, 0.05, 25.0);
        self.quantum_efficiency_percent = clean(self.quantum_efficiency_percent, 1.0, 0.0, 50.0);
        self.custom_phi_ev = clean(self.custom_phi_ev, 3.0, 1.0, 6.5);
        self.comparison_custom_phi_ev = clean(self.comparison_custom_phi_ev, 3.0, 1.0, 6.5);
        self
    }
}

/// Un punto medido del experimento V₀–f.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[derive(TS)]
#[ts(export, export_to = "ExperimentPoint.ts")]
pub struct ExperimentPoint {
    pub wavelength_nm: f64,
    pub frequency_hz: f64,
    pub stopping_measured_v: f64,
    pub stopping_ideal_v: f64,
    /// Media desviación estándar asignada (barras de error), en V.
    pub error_v: f64,
    pub material_id: String,
    pub material_name: String,
    pub noise_percent: f64,
}

impl Default for ExperimentPoint {
    fn default() -> Self {
        Self {
            wavelength_nm: 400.0,
            frequency_hz: 0.0,
            stopping_measured_v: 0.0,
            stopping_ideal_v: 0.0,
            error_v: 0.0,
            material_id: String::new(),
            material_name: String::new(),
            noise_percent: 0.0,
        }
    }
}

/// Ajuste guardado para comparar pendientes entre materiales.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[derive(TS)]
#[ts(export, export_to = "SavedFit.ts")]
pub struct SavedFit {
    pub material_id: String,
    pub material_name: String,
    pub n_points: usize,
    pub slope: f64,
    pub intercept: f64,
    pub h_experimental: f64,
    pub error_percentage: f64,
    pub r_squared: f64,
}

impl Default for SavedFit {
    fn default() -> Self {
        Self {
            material_id: String::new(),
            material_name: String::new(),
            n_points: 0,
            slope: 0.0,
            intercept: 0.0,
            h_experimental: 0.0,
            error_percentage: 0.0,
            r_squared: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[derive(TS)]
#[ts(export, export_to = "ExperimentState.ts")]
pub struct ExperimentState {
    pub points: Vec<ExperimentPoint>,
    /// false = ideal (V₀ exacto), true = experimental (ruido ±noisePercent).
    pub experimental_mode: bool,
    pub noise_percent: f64,
    /// Semilla splitmix64. En TS se representa como string para no perder
    /// los bits bajos (JS no tiene enteros de 64 bits exactos). Se acepta
    /// también un número al leer sesiones antiguas.
    #[serde(
        serialize_with = "ser_u64_as_string",
        deserialize_with = "de_u64_from_string"
    )]
    #[ts(type = "string")]
    pub next_seed: u64,
    pub saved_fits: Vec<SavedFit>,
}

impl Default for ExperimentState {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            experimental_mode: false,
            noise_percent: 2.0,
            next_seed: 0x9E37_79B9_7F4A_7C15,
            saved_fits: Vec::new(),
        }
    }
}

/// Temas de la UI: oscuro (por defecto), claro y alto contraste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(TS)]
#[ts(export, export_to = "ThemeChoice.ts")]
pub enum ThemeChoice {
    #[default]
    Dark,
    Light,
    /// `"contrast"` en JSON/TS (no `high_contrast`), por brevedad en el front.
    #[serde(rename = "contrast")]
    #[ts(rename = "contrast")]
    HighContrast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(TS)]
#[ts(export, export_to = "SpeedScaleMode.ts")]
pub enum SpeedScaleMode {
    /// √(K/1 eV) acotada a [0.5, 2.2] — comprime diferencias grandes.
    #[default]
    Compressed,
    /// v real (m/s) mapeada logarítmicamente, con leyenda explicativa.
    RealLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(TS)]
#[ts(export, export_to = "Lang.ts")]
pub enum Lang {
    #[default]
    Es,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(TS)]
#[ts(export, export_to = "ReducedMotion.ts")]
pub enum MotionPreference {
    #[default]
    Full,
    /// Sin estelas ni bloom (accesibilidad / prefers-reduced-motion).
    Reduced,
}

/// Capas del canvas conmutables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[derive(TS)]
#[ts(export, export_to = "CanvasLayers.ts")]
pub struct CanvasLayers {
    pub field: bool,
    pub energy_labels: bool,
    pub trajectories: bool,
    pub grid: bool,
    pub current_meter: bool,
    pub energy_ladder: bool,
}

impl Default for CanvasLayers {
    fn default() -> Self {
        Self {
            field: true,
            energy_labels: true,
            trajectories: true,
            grid: true,
            current_meter: true,
            energy_ladder: true,
        }
    }
}

/// Preferencias de UI persistidas con la sesión.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[derive(TS)]
#[ts(export, export_to = "UiPreferences.ts")]
pub struct UiPreferences {
    pub theme: ThemeChoice,
    pub lang: Lang,
    pub font_scale: f64,
    pub speed_scale: SpeedScaleMode,
    pub motion: MotionPreference,
    pub layers: CanvasLayers,
    pub controls_panel_open: bool,
    pub readouts_panel_open: bool,
    pub controls_panel_width: f64,
    pub readouts_panel_width: f64,
    pub presentation_mode: bool,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::Dark,
            lang: Lang::Es,
            font_scale: 1.0,
            speed_scale: SpeedScaleMode::Compressed,
            motion: MotionPreference::Full,
            layers: CanvasLayers::default(),
            controls_panel_open: true,
            readouts_panel_open: true,
            controls_panel_width: 296.0,
            readouts_panel_width: 300.0,
            presentation_mode: false,
        }
    }
}

/// Sesión completa `.photolab.json`: lo único que se guarda/carga.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[derive(TS)]
#[ts(export, export_to = "Session.ts")]
pub struct Session {
    pub schema_version: u32,
    pub app: String,
    pub controls: ExperimentControls,
    pub preferences: UiPreferences,
    pub experiment: ExperimentState,
    pub notes: String,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            app: "photolab".to_owned(),
            controls: ExperimentControls::default(),
            preferences: UiPreferences::default(),
            experiment: ExperimentState::default(),
            notes: String::new(),
        }
    }
}

impl Session {
    pub fn from_parts(
        controls: ExperimentControls,
        preferences: UiPreferences,
        experiment: ExperimentState,
        notes: String,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            app: "photolab".to_owned(),
            controls: controls.sanitized(),
            preferences,
            experiment,
            notes,
        }
    }
}

/// Cambio de controles registrado para undo/redo (viaja en `EvaluateResponse`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export, export_to = "ControlChange.ts")]
pub struct ControlChange {
    pub can_undo: bool,
    pub can_redo: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_sanitize_never_nan() {
        let weird = ExperimentControls {
            wavelength_nm: f64::NAN,
            intensity_percent: f64::INFINITY,
            applied_voltage_v: -50.0,
            cathode_area_cm2: -1.0,
            quantum_efficiency_percent: 900.0,
            ..Default::default()
        }
        .sanitized();
        assert_eq!(weird.wavelength_nm, 400.0);
        assert_eq!(weird.intensity_percent, 55.0);
        assert_eq!(weird.applied_voltage_v, -5.0);
        assert_eq!(weird.cathode_area_cm2, 0.05);
        assert_eq!(weird.quantum_efficiency_percent, 50.0);
    }

    #[test]
    fn custom_material_is_clamped() {
        let custom = MaterialChoice::Custom;
        assert_eq!(custom.work_function_ev(2.5), 2.5);
        assert_eq!(custom.work_function_ev(99.0), 3.0);
        assert_eq!(custom.work_function_ev(f64::NAN), 3.0);
    }

    #[test]
    fn session_roundtrips_and_tolerates_missing_fields() {
        let json = r#"{"schemaVersion":1,"app":"photolab","notes":"hola"}"#;
        let session: Session = serde_json::from_str(json).unwrap();
        assert_eq!(session.notes, "hola");
        assert_eq!(session.controls.material, MaterialChoice::Sodium);
        let encoded = serde_json::to_string(&session).unwrap();
        let back: Session = serde_json::from_str(&encoded).unwrap();
        assert_eq!(back, session);
    }

    #[test]
    fn next_seed_is_json_string_and_legacy_numbers_still_load() {
        let mut session = Session::default();
        session.experiment.next_seed = 11_400_714_819_323_198_485;
        let encoded = serde_json::to_string(&session).unwrap();
        assert!(encoded.contains(r#""nextSeed":"11400714819323198485""#));
        let back: Session = serde_json::from_str(&encoded).unwrap();
        assert_eq!(back.experiment.next_seed, session.experiment.next_seed);

        let legacy = encoded.replace(
            r#""nextSeed":"11400714819323198485""#,
            r#""nextSeed":11400714819323198485"#,
        );
        let parsed: Session = serde_json::from_str(&legacy).unwrap();
        assert_eq!(parsed.experiment.next_seed, session.experiment.next_seed);
    }
}
