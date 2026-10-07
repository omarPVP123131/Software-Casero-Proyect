# Arquitectura

## Workspace

```text
crates/physics/   fotoelectrico-physics   Ecuaciones + pruebas. Sin UI ni dibujo.
crates/app/       fotoelectrico-app       Controles, lecturas, puente al motor. Binario.
crates/engine/    fotoelectrico-engine    Renderer Macroquad por capas. Sin ecuaciones.
```

Dependencias permitidas: `app → physics`, `app → engine`. Prohibido: `engine → physics`, `engine → app`, `physics → {app, engine}`, y cualquier ecuación fuera de `physics` + `physics_adapter`.

## Flujo de datos

```text
ExperimentControls { material, comparison_material, wavelength_nm,
                     intensity_percent, applied_voltage_v }
        │
        ▼  physics_adapter::refresh_state() — cada frame y tras undo/redo
PhysicsReadout { emission_possible, collected_possible, frequency_hz,
                 photon_energy_ev, work_function_ev, threshold_*,
                 max_kinetic_energy_ev, stopping_potential_v,
                 photon_flux_density_per_m2_s, electron_max_speed_m_s,
                 kinetic_energy_curve }  ×2 (principal + comparación)
        │
        ▼  main.rs empaqueta SceneFrame
SceneFrame { wavelength_nm, intensity_percent, visual_voltage_v,
             emission_possible, collection_possible, k_max_ev,
             electron_max_speed_m_s, viewport, capas, calidad, ... }
        │
        ▼  Renderer::render() por capas: cell → field → photons → electrons → overlays
RenderStats { fps, frame_time_ms, photons_drawn, electrons_drawn }
```

## Archivos clave

- `crates/physics/src/{constants,material,photon,fit,lib}.rs` — modelo y `get_physics_readout`.
- `crates/app/src/physics_adapter.rs` — único lugar que traduce controles a lecturas.
- `crates/app/src/state.rs` — `ExperimentControls`, `PhysicsReadout` (opciones para el primer frame), preferencias, historial, undo/redo.
- `crates/app/src/main.rs` — loop: `tick` (tiempo) → `refresh_physics` → `draw_ui` → `render`.
- `crates/app/src/ui.rs`, `visualization.rs` — solo presentan; no calculan.
- `crates/engine/src/scene.rs` — contrato `SceneFrame`; `renderer.rs` + `layers/*` — dibujo.

## Invariantes

1. Ningún número físico se calcula en `ui.rs`, `visualization.rs` ni `engine/`.
2. Cálculos internos en SI; los bordes convierten a eV/nm/V con unidades en el nombre.
3. `tick` solo avanza tiempo y cámara; la física se refresca explícitamente.
4. `SceneFrame` lleva valores finales, no parámetros para derivar física en el renderer.
