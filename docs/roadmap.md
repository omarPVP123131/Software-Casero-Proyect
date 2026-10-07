# Roadmap

Estado real a octubre de 2026. Solo se marca hecho lo que compila, pasa pruebas y está visible en la app.

## Hecho

- **v0.1 Física.** Workspace, constantes SI, `Material`, `calculate_effect`, `get_physics_readout` y pruebas de umbral/`Kmax`/conversiones.
- **v0.2 Interfaz conectada.** Controles (material, `λ`, intensidad, `V`), lecturas en vivo, curva `Kmax(λ)` y comparación con el mismo punto de operación.
- **v0.3 Experimento.** Pestaña Experimento: tabla `(f, V₀)` con validación (emisión, duplicados, un material), modos ideal/experimental con ruido determinista ±%, ajuste visible (m, b, h estimada, error %, R²), gráfica V₀–f con recta, exportación CSV del experimento y de la sesión (archivo + portapapeles).
- **v0.4 Visualización básica.** Celda, haz por intensidad, electrones por colección/intensidad con velocidad `∝ √Kmax`, flechas por voltaje, inspector por forma y HUD.

## Pendiente

- [ ] Fotocorriente calibrada contra datos reales (hoy A = 1 cm² y QE = 1 % ilustrativos) y modelo estocástico de emisión.
- [ ] Prueba de humo en el equipo de demostración y captura para el README.

## Historial

Este archivo sustituye al antiguo `Roadmap.md` de la raíz, que describía el estado inicial vacío del repositorio y ya no correspondía al código.
