# Desarrollo

## Entorno

- Rust estable (probado con `rustc 1.99.0`, `cargo 1.99.0`).
- Primera compilación requiere red para descargar dependencias (`egui-macroquad`, `macroquad`, `serde`).

## Comandos

```bash
cargo run -p fotoelectrico-app
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo build --workspace
```

`cargo test -p fotoelectrico-physics` corre solo el modelo; `cargo run` usa `crates/app` (miembro por defecto del workspace).

## Convenciones

- Unidades en SI dentro del motor; nombres con sufijo de unidad (`_hz`, `*_m`, `*_nm`, `*_ev`, `*_j`, `*_v`).
- Ecuaciones solo en `crates/physics` y `crates/app/src/physics_adapter.rs`. La UI y el engine presentan valores ya calculados.
- `AppState::tick` no toca lecturas; la física se actualiza con `refresh_physics`.
- `SceneFrame` transporta resultados finales, nunca entradas para derivar física en el renderer.

## Tareas comunes

- **Cambiar Φ de un material.** Edita `MaterialChoice::work_function_ev` en `crates/app/src/state.rs` y documenta la fuente en `docs/physics-model.md`.
- **Cambiar el modelo.** Edita `crates/physics`, añade/ajusta pruebas en el mismo archivo, y actualiza `docs/physics-model.md` en el mismo cambio.
- **Añadir una lectura visible.** Extiende `PhysicsReadout` (app), calcúlala en `build_readout`, muéstrala en `ui.rs` y pásala a `SceneFrame` solo si el canvas la necesita.
- **Persistencia.** Guarda preferencias, diseño, notas y tabla del experimento (`persistence.rs`); controles y lecturas siempre arrancan de valores conocidos y se recalculan.

## Pruebas

- `physics`: umbral, `Kmax(f)`, consistencia `λ ↔ f`, regresión + R², robustez ante entradas inválidas.
- `app`: arranque con física conectada, undo/redo con refresco, hit-test con colección/bloqueo, cámara e inspector, experimento (validación, ajuste E2E, escenarios) y persistencia con migración.
- `engine`: conteo por intensidad, velocidad por `Kmax`, cero electrones bloqueados, llegada estocástica determinista.
