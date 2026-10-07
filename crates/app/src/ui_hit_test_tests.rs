use fotoelectrico_engine::layers::{
    cell::layout_geometry,
    electrons::{electron_count, electron_speed_scale, visual_electrons},
    field::arrow_segments,
    photons::{beam_lane_y, beam_x_bounds},
};
use fotoelectrico_engine::{RenderQuality, Viewport};

use crate::state::{electrons_visible_in_scene, AppState, InspectedObject};
use crate::ui::hit_test_object;

fn setup() -> (AppState, fotoelectrico_engine::layers::cell::CellGeometry) {
    let mut state = AppState::default();
    // Sodio 400 nm por defecto: hay emisión y colección para probar electrones.
    state.controls.wavelength_nm = 400.0;
    state.controls.intensity_percent = 55.0;
    state.controls.applied_voltage_v = 0.0;
    state.refresh_physics();
    // Aproxima el canvas central tras los paneles en una ventana inicial de 1440×900.
    let viewport = Viewport::new(290.0, 170.0, 850.0, 620.0);
    let geometry = layout_geometry(viewport, 1.0, 0.0, 0.0);
    (state, geometry)
}

fn state_electrons(
    state: &AppState,
    geometry: fotoelectrico_engine::layers::cell::CellGeometry,
) -> Vec<fotoelectrico_engine::layers::electrons::ElectronVisual> {
    let speed = electron_speed_scale(state.readout.max_kinetic_energy_ev);
    let visible = electrons_visible_in_scene(state);
    let count = electron_count(
        state.controls.intensity_percent,
        visible,
        RenderQuality::Balanced,
    );
    visual_electrons(
        geometry,
        state.animation_time_seconds,
        state.animation_running,
        state.animation_speed,
        RenderQuality::Balanced,
        speed,
        count,
    )
}

#[test]
fn startup_hit_tests_cover_electrodes_beam_electrons_and_canvas() {
    let (state, geometry) = setup();
    assert_eq!(
        hit_test_object(&state, geometry, (geometry.cathode_x, geometry.mid_y)),
        Some(InspectedObject::Cathode)
    );
    assert_eq!(
        hit_test_object(&state, geometry, (geometry.anode_x, geometry.mid_y)),
        Some(InspectedObject::Anode)
    );

    let (beam_start, beam_end) = beam_x_bounds(geometry);
    let beam_point = ((beam_start + beam_end) * 0.5, beam_lane_y(geometry, 2));
    assert_eq!(
        hit_test_object(&state, geometry, beam_point),
        Some(InspectedObject::PhotonBeam)
    );

    let electrons = state_electrons(&state, geometry);
    assert!(!electrons.is_empty(), "con emisión debe haber electrones");
    let electron = electrons[0];
    assert_eq!(
        hit_test_object(&state, geometry, (electron.x, electron.y)),
        Some(InspectedObject::ElectronLayer)
    );

    assert_eq!(
        hit_test_object(&state, geometry, (310.0, 190.0)),
        None,
        "a click elsewhere in the allocated canvas is selectable as the canvas fallback"
    );
}

#[test]
fn overlapping_electrode_hitbox_has_explicit_priority_over_particles() {
    let (state, geometry) = setup();
    assert_eq!(
        hit_test_object(&state, geometry, (geometry.cathode_x, geometry.mid_y)),
        Some(InspectedObject::Cathode)
    );
}

#[test]
fn hidden_or_empty_layers_are_not_interactive_objects() {
    let (mut state, geometry) = setup();
    let (beam_start, beam_end) = beam_x_bounds(geometry);
    let beam_point = ((beam_start + beam_end) * 0.5, beam_lane_y(geometry, 2));
    state.preferences.layers.photons = false;
    assert_eq!(hit_test_object(&state, geometry, beam_point), None);

    state.preferences.layers.electrons = false;
    state.preferences.layers.demo_electrons = false;
    let particle = visual_electrons(
        geometry,
        state.animation_time_seconds,
        state.animation_running,
        state.animation_speed,
        RenderQuality::Balanced,
        1.0,
        4,
    )[0];
    assert_eq!(
        hit_test_object(&state, geometry, (particle.x, particle.y)),
        None
    );

    // Sin emisión (700 nm en sodio) no hay electrones aunque la capa siga activa.
    state.preferences.layers.electrons = true;
    state.controls.wavelength_nm = 700.0;
    state.refresh_physics();
    assert_eq!(state.readout.emission_possible, Some(false));
    let empty = state_electrons(&state, geometry);
    assert!(empty.is_empty());

    state.preferences.layers.arrows = false;
    state.controls.applied_voltage_v = 0.0;
    assert_eq!(
        hit_test_object(
            &state,
            geometry,
            (
                (geometry.cathode_x + geometry.anode_x) * 0.5,
                geometry.mid_y
            ),
        ),
        None
    );
}

#[test]
fn blocked_collection_hides_electrons_from_hit_test() {
    let (mut state, geometry) = setup();
    // Voltaje muy negativo bloquea la colección aunque haya emisión.
    let v0 = state.readout.stopping_potential_v.unwrap();
    state.controls.applied_voltage_v = (-v0 - 1.0) as f32;
    // Ocultamos flechas para aislar el centro de la celda.
    state.preferences.layers.arrows = false;
    state.refresh_physics();
    assert_eq!(state.readout.emission_possible, Some(true));
    assert_eq!(state.readout.collected_possible, Some(false));
    assert!(state_electrons(&state, geometry).is_empty());
    assert_eq!(
        hit_test_object(
            &state,
            geometry,
            (
                (geometry.cathode_x + geometry.anode_x) * 0.5,
                geometry.mid_y
            ),
        ),
        None
    );
}

#[test]
fn field_arrows_are_tested_using_the_same_segments_as_the_renderer() {
    let (mut state, geometry) = setup();
    state.preferences.layers.electrons = false;
    state.controls.applied_voltage_v = 2.0;
    let segment = arrow_segments(geometry, state.controls.applied_voltage_v)[0];
    let point = (
        (segment.start_x + segment.end_x) * 0.5,
        (segment.start_y + segment.end_y) * 0.5,
    );
    assert_eq!(
        hit_test_object(&state, geometry, point),
        Some(InspectedObject::FieldArrows)
    );
}
