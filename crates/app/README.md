# `fotoelectrico-app` — interfaz y puente al motor

Propósito: presentar controles y lecturas, y ser el único punto que llama al motor físico. No contiene ecuaciones: delega en `physics_adapter`.

## Cómo funciona

1. El usuario mueve un control (`state::ExperimentControls`).
2. `physics_adapter::refresh_state` recalcula `PhysicsReadout` principal + comparación y la curva `Kmax(λ)` (61 puntos, 180–900 nm).
3. `main.rs` empaqueta un `SceneFrame` y `engine` lo dibuja.

Supuestos del puente (documentados en código y en `docs/physics-model.md`): 100 % = 10 mW/cm², `flujo = P/E_fotón`, colección bloqueada si `V < −V₀`.

## Ejecutar

```bash
cargo run -p fotoelectrico-app
```

Estructura: `state.rs` (controles, lecturas, preferencias), `physics_adapter.rs` (puente), `ui.rs` (paneles y pestañas), `visualization.rs` (gráfica y espectro), `persistence.rs` (diseño guardado), `main.rs` (loop + `SceneFrame`).

Guía de uso en [`docs/user-guide.md`](../../docs/user-guide.md); contrato entre crates en [`docs/architecture.md`](../../docs/architecture.md).
