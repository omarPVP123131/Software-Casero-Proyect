# Interfaz gráfica — `fotoelectrico-app`

Prototipo visual de escritorio con Macroquad + egui. La app mantiene la UI desacoplada de la crate física: los controles viven en `state::LabControls` y los valores de salida se reciben en `state::PhysicsReadout`.

## Ejecutar

Desde la raíz del workspace:

```bash
cargo run -p fotoelectrico-app
```

## Límite de responsabilidad

Este crate **no calcula física**. Mientras `PhysicsReadout` esté vacío, el panel de resultados muestra que la integración está pendiente y la celda anima partículas de manera ilustrativa. Al conectar el módulo de tu compañero, la capa de aplicación puede llenar `PhysicsReadout` con los resultados; la UI ya sabe presentarlos.

El control de voltaje y la animación son controles visuales de prototipo, no una simulación del campo eléctrico ni trayectorias integradas.
