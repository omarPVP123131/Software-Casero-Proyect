use crate::state::{CurvePoint, LabControls, MaterialChoice, PhysicsReadout};

#[test]
fn controls_start_with_a_visible_default_experiment() {
    let controls = LabControls::default();
    assert_eq!(controls.material, MaterialChoice::Sodium);
    assert!(controls.wavelength_nm > 0.0);
    assert!(controls.animation_running);
}

#[test]
fn physics_readout_defaults_to_unconnected_values() {
    let readout = PhysicsReadout::default();
    assert_eq!(readout.emission_possible, None);
    assert_eq!(readout.frequency_hz, None);
    assert_eq!(readout.photon_energy_ev, None);
    assert!(readout.kinetic_energy_curve.is_empty());
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
