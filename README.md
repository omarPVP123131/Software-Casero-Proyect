# Simulador del efecto fotoeléctrico — estructura limpia

Este workspace deja la estructura lista para el equipo. **La física está vacía a propósito**: tu compañero puede implementar `crates/physics` sin que esta interfaz invente ecuaciones.

## Crates

- `crates/physics`: esqueleto vacío para la implementación física.
- `crates/engine`: esqueleto para el renderer Macroquad; aún no dibuja.
- `crates/app`: interfaz egui + Macroquad, con controles y celda esquemática. Sus métricas y gráfica esperan datos del modelo físico.

## Requisitos

Instala Rust estable con Cargo. La primera compilación necesita conexión a internet para descargar Macroquad y egui.

## Ejecutar la interfaz

Desde esta carpeta:

```powershell
cargo run
```

El workspace tiene `crates/app` como miembro predeterminado, así que también puedes ser explícito:

```powershell
cargo run -p fotoelectrico-app
```

## Verificar todo el workspace

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo build --workspace
```

## Qué se muestra y qué falta conectar

Los controles de material, longitud de onda, intensidad y voltaje son visuales. Las partículas animadas son demostrativas. Los campos `PhysicsReadout` en `crates/app/src/state.rs` y la función `draw(...)` en `crates/app/src/lib.rs` forman el punto de integración: la capa física podrá entregar allí sus lecturas sin que la UI recalcule nada.

El crate `engine` está creado pero todavía no está conectado a `app`; por ahora el prototipo gráfico vive en `crates/app`.
