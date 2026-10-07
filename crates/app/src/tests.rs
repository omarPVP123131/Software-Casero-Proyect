use crate::state::{
    AppState, CurvePoint, ExperimentControls, InspectedObject, LayoutPreset, MaterialChoice,
    PhysicsReadout,
};

#[test]
fn app_starts_with_physics_connected() {
    let app = AppState::default();
    assert_eq!(app.controls.material, MaterialChoice::Sodium);
    assert!(app.controls.wavelength_nm > 0.0);
    assert!(app.animation_running);
    // Motor conectado: lecturas y curva presentes desde el arranque.
    assert!(app.readout.emission_possible.is_some());
    assert!(app.readout.photon_energy_ev.is_some());
    assert!(app.readout.max_kinetic_energy_ev.is_some());
    assert!(!app.readout.kinetic_energy_curve.is_empty());
    assert!(!app.comparison_readout.kinetic_energy_curve.is_empty());
}

#[test]
fn interface_defaults_keep_debug_hitboxes_and_canvas_access_available() {
    let app = AppState::default();
    assert!(app.preferences.show_hitboxes);
    assert!(app.preferences.controls_panel_open);
    assert!(app.preferences.readouts_panel_open);
    assert_eq!(app.preferences.layout_preset, LayoutPreset::Workbench);
}

#[test]
fn selecting_an_object_sets_inspector_and_smooth_camera_target() {
    let mut app = AppState::default();
    app.select_object(InspectedObject::Cathode);

    assert_eq!(app.inspector, Some(InspectedObject::Cathode));
    assert_eq!(
        app.preferences.zoom, 1.0,
        "focus zoom is transient, not a saved preference"
    );
    assert!((app.scene_camera.target_zoom - 1.25).abs() < f32::EPSILON);
    assert!(app.scene_camera.target_pan_x > 0.0);
    assert!(app.scene_camera.pan_x.abs() < 0.001);

    app.tick(0.05);
    assert!(app.scene_camera.zoom > 1.0);
    assert!(app.scene_camera.zoom < app.scene_camera.target_zoom);
    assert!(app.scene_camera.pan_x > 0.0);
}

#[test]
fn hiding_a_selected_visual_layer_closes_its_inspector_and_refocuses_canvas() {
    let mut app = AppState::default();
    app.select_object(InspectedObject::PhotonBeam);
    app.preferences.layers.photons = false;
    app.reconcile_inspector_visibility();

    assert_eq!(app.inspector, None);
    assert!((app.scene_camera.target_zoom - app.preferences.zoom).abs() < f32::EPSILON);
    assert_eq!(app.scene_camera.target_pan_x, 0.0);
    assert_eq!(app.scene_camera.target_pan_y, 0.0);
}

#[test]
fn camera_focus_uses_the_real_position_of_the_selected_scene_object() {
    let mut app = AppState::default();
    app.select_object_at(InspectedObject::ElectronLayer, (0.62, 0.44));

    assert!((app.scene_camera.target_zoom - 1.25).abs() < f32::EPSILON);
    assert!((app.scene_camera.target_pan_x + 0.15).abs() < 0.0001);
    assert!((app.scene_camera.target_pan_y - 0.075).abs() < 0.0001);
}

#[test]
fn closing_inspector_restores_general_camera_and_selection() {
    let mut app = AppState::default();
    app.select_object(InspectedObject::Anode);
    app.close_inspector();

    assert_eq!(app.inspector, None);
    assert_eq!(app.preferences.zoom, 1.0);
    assert_eq!(app.scene_camera.target_zoom, 1.0);
    assert_eq!(app.scene_camera.target_pan_x, 0.0);
    assert_eq!(app.scene_camera.target_pan_y, 0.0);
}

#[test]
fn inspector_focus_does_not_overwrite_a_manual_zoom_preference() {
    let mut app = AppState::default();
    app.preferences.zoom = 1.4;
    app.select_object(InspectedObject::Anode);
    assert_eq!(app.preferences.zoom, 1.4);
    assert!((app.scene_camera.target_zoom - 1.4).abs() < f32::EPSILON);

    app.preferences.zoom = 0.9;
    app.close_inspector();
    assert!((app.scene_camera.target_zoom - 0.9).abs() < f32::EPSILON);
}

#[test]
fn layout_presets_configure_and_remember_panel_arrangement() {
    let mut app = AppState::default();
    app.set_layout_preset(LayoutPreset::Analysis);
    assert_eq!(app.active_tab, crate::state::LabTab::Chart);
    assert!(!app.preferences.controls_panel_open);
    assert!(app.preferences.readouts_panel_open);

    app.set_layout_preset(LayoutPreset::Presentation);
    assert!(app.preferences.presentation_mode);
    assert!(!app.preferences.controls_panel_open);
    assert!(!app.preferences.readouts_panel_open);
    assert_eq!(app.preferences.layout_preset, LayoutPreset::Presentation);
}

#[test]
fn physics_readout_struct_defaults_to_none_before_refresh() {
    let readout = PhysicsReadout::default();
    assert_eq!(readout.emission_possible, None);
    assert_eq!(readout.collected_possible, None);
    assert_eq!(readout.frequency_hz, None);
    assert_eq!(readout.photon_energy_ev, None);
    assert!(readout.kinetic_energy_curve.is_empty());
}

#[test]
fn app_default_refreshes_physics_immediately() {
    let app = AppState::default();
    // Default ya llamó a refresh_state.
    assert_eq!(app.readout.emission_possible, Some(true));
    assert_eq!(app.readout.collected_possible, Some(true));
    assert!(app.readout.electron_max_speed_m_s.unwrap() > 0.0);
}

#[test]
fn curve_point_is_a_view_value_only() {
    let point = CurvePoint {
        wavelength_nm: 400.0,
        max_kinetic_energy_ev: 1.2,
    };
    assert_eq!(point.wavelength_nm, 400.0);
    assert_eq!(point.max_kinetic_energy_ev, 1.2);
}

#[test]
fn control_undo_and_redo_restore_user_inputs_and_refresh_physics() {
    let mut app = AppState::default();
    let before = app.controls;
    app.controls.wavelength_nm = 510.0;
    app.record_control_change(before);
    app.refresh_physics();
    assert_eq!(app.controls.wavelength_nm, 510.0);
    app.undo();
    assert_eq!(app.controls, ExperimentControls::default());
    // Tras undo, la física se recalcula y la curva sigue presente.
    assert!(!app.readout.kinetic_energy_curve.is_empty());
    assert_eq!(app.readout.emission_possible, Some(true));
    app.redo();
    assert_eq!(app.controls.wavelength_nm, 510.0);
    assert!(!app.readout.kinetic_energy_curve.is_empty());
}

#[test]
fn continuous_slider_edits_are_grouped_for_undo() {
    let mut app = AppState::default();
    let before = app.controls;
    app.controls.wavelength_nm = 430.0;
    app.record_control_change(before);
    let before_second_drag_update = app.controls;
    app.controls.wavelength_nm = 470.0;
    app.record_control_change(before_second_drag_update);

    app.undo();
    assert_eq!(
        app.controls.wavelength_nm,
        ExperimentControls::default().wavelength_nm
    );
    app.redo();
    assert_eq!(app.controls.wavelength_nm, 470.0);
}

#[test]
fn animation_clock_does_not_touch_physics_readouts() {
    let mut app = AppState::default();
    let initial_readout = app.readout.clone();
    app.tick(0.05);
    assert!(app.animation_time_seconds > 0.0);
    assert_eq!(app.readout, initial_readout);
}

#[test]
fn experiment_table_rejects_no_emission_duplicates_and_mixed_materials() {
    use crate::state::{ExperimentState, MAX_EXPERIMENT_POINTS};
    let mut exp = ExperimentState::default();
    assert!(exp.points.is_empty());

    // Sin emisión no se puede medir.
    assert!(exp
        .try_add(MaterialChoice::Sodium, 700.0, 4.28e14, 0.0, false, 0.0)
        .is_err());

    // Punto válido.
    assert!(exp
        .try_add(MaterialChoice::Sodium, 400.0, 7.49e14, 0.74, true, 0.74)
        .is_ok());
    assert_eq!(exp.points.len(), 1);

    // Duplicado de λ (±0.5 nm) se rechaza.
    assert!(exp
        .try_add(MaterialChoice::Sodium, 400.3, 7.49e14, 0.74, true, 0.74)
        .is_err());

    // Otro material exige vaciar primero (el ajuste supone un solo Φ).
    assert!(exp
        .try_add(MaterialChoice::Copper, 300.0, 9.99e14, 1.5, true, 1.5)
        .is_err());

    exp.remove(0);
    assert!(exp.points.is_empty());
    assert!(exp
        .try_add(MaterialChoice::Copper, 300.0, 9.99e14, 1.5, true, 1.5)
        .is_ok());

    exp.clear();
    assert!(exp.points.is_empty());
    assert_eq!(MAX_EXPERIMENT_POINTS, 24);
}

#[test]
fn experiment_fit_recovers_planck_from_ideal_points() {
    use fotoelectrico_physics::{fit_planck_constant, DataPoint};
    // Recta teórica del sodio: V₀ = (h/e)·f − Φ/e.
    let h = 6.62607015e-34;
    let e = 1.602176634e-19;
    let points: Vec<DataPoint> = [400.0, 350.0, 300.0, 250.0]
        .iter()
        .map(|nm| {
            let f = 299_792_458.0 / (nm * 1e-9);
            DataPoint {
                frequency_hz: f,
                stopping_potential_v: h * f / e - 2.36,
            }
        })
        .collect();
    let fit = fit_planck_constant(&points).unwrap();
    assert!((fit.r_squared - 1.0).abs() < 1e-9);
    assert!(fit.error_percentage < 0.01);
}

#[test]
fn readout_includes_photocurrent_and_collection_factor() {
    let app = AppState::default();
    let current = app.readout.photocurrent_a.expect("fotocorriente calculada");
    assert!(current.is_finite() && current > 0.0);
    let factor = app.readout.collection_factor.expect("factor calculado");
    assert!((factor - 1.0).abs() < 1e-9, "a V=0 la colección es total");
}

#[test]
fn full_experiment_flow_from_app_state_recovers_planck() {
    use crate::physics_adapter::noisy_stopping_v;
    use fotoelectrico_physics::fit_planck_constant;
    use fotoelectrico_physics::DataPoint;
    // Flujo extremo a extremo: controles → refresh → captura → ajuste.
    let mut app = AppState::default();
    for lambda in [300.0, 350.0, 400.0, 450.0] {
        app.controls.wavelength_nm = lambda;
        app.controls.applied_voltage_v = 0.0;
        app.refresh_physics();
        assert_eq!(app.readout.emission_possible, Some(true));
        let freq = app.readout.frequency_hz.unwrap();
        let ideal = app.readout.stopping_potential_v.unwrap();
        let measured = noisy_stopping_v(ideal, 0.0, &mut app.experiment.next_seed);
        app.experiment
            .try_add(app.controls.material, lambda, freq, ideal, true, measured)
            .expect("punto válido");
    }
    assert_eq!(app.experiment.points.len(), 4);
    let data: Vec<DataPoint> = app
        .experiment
        .points
        .iter()
        .map(|p| DataPoint {
            frequency_hz: p.frequency_hz,
            stopping_potential_v: p.stopping_measured_v,
        })
        .collect();
    let fit = fit_planck_constant(&data).expect("ajuste con 4 puntos");
    assert!(fit.error_percentage < 0.01, "modo ideal recupera h");
    assert!((fit.r_squared - 1.0).abs() < 1e-9);
}

#[test]
fn scenario_presets_produce_expected_physics() {
    use crate::state::Scenario;
    let mut app = AppState::default();
    // Contraste K/Pt a 400 nm: el potasio emite, el platino no.
    app.apply_scenario(Scenario::Contrast);
    assert_eq!(app.readout.emission_possible, Some(true));
    assert_eq!(app.comparison_readout.emission_possible, Some(false));
    // Frenado total: emisión sí, colección no.
    app.apply_scenario(Scenario::Blocked);
    assert_eq!(app.readout.emission_possible, Some(true));
    assert_eq!(app.readout.collected_possible, Some(false));
    assert_eq!(app.readout.photocurrent_a, Some(0.0));
}
