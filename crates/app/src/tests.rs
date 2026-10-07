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
