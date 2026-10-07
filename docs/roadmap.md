# Roadmap

Estado real a octubre de 2026. Solo se marca hecho lo que compila, pasa pruebas y está visible en la app.

## Hecho

- **v0.1 Física.** Workspace, constantes SI, `Material`, `calculate_effect`, `get_physics_readout` y pruebas de umbral/`Kmax`/conversiones.
- **v0.2 Interfaz conectada.** Controles (material, `λ`, intensidad, `V`), lecturas en vivo, curva `Kmax(λ)` y comparación con el mismo punto de operación.
- **v0.4 Visualización básica.** Celda, haz por intensidad, electrones por colección/intensidad con velocidad `∝ √Kmax`, flechas por voltaje, inspector por forma y HUD.

## Parcial

- **v0.3 Experimento.** `physics::fit` (regresión `V₀` contra `f` → `h = e·m`) existe y está probada, pero la UI aún no tiene tabla de puntos, ajuste visible ni exportación CSV.

## Pendiente

- [ ] Tabla `(f, V₀)` editable en la UI con unidades y validación sobre el umbral.
- [ ] Ajuste visible con pendiente, ordenada, `h` estimada y error %; exportar CSV con encabezados y unidades.
- [ ] Documentar incertidumbre/ruido si se presenta como “experimento” y no como verificación del modelo ideal.
- [ ] Curva corriente–voltaje y modelo estocástico (fuera del prototipo actual).
- [ ] Prueba de humo en el equipo de demostración y captura para el README.

## Historial

Este archivo sustituye al antiguo `Roadmap.md` de la raíz, que describía el estado inicial vacío del repositorio y ya no correspondía al código.
