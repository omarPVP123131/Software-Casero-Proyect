# Roadmap

Estado real a octubre de 2026. Solo se marca hecho lo que compila, pasa pruebas y está visible en la app.

## Hecho

- **v0.1 Física.** Workspace, constantes SI, `Material`, `calculate_effect`, `get_physics_readout` y pruebas de umbral/`Kmax`/conversiones.
- **v0.2 Interfaz conectada.** Controles (material, `λ`, intensidad, `V`), lecturas en vivo, curva `Kmax(λ)` y comparación con el mismo punto de operación.
- **v0.3 Experimento.** Pestaña Experimento: tabla `(f, V₀)` con validación (emisión, duplicados, un material), modos ideal/experimental con ruido determinista ±%, ajuste visible (m, b, h estimada, error %, R²), gráfica V₀–f con recta y barras de error, curva I–V, fotocorriente calibrable, exportación CSV del experimento y de la sesión (archivo + portapapeles). Tabla y notas persistentes.
- **v0.4 Visualización.** Celda, haz por intensidad, electrones con llegada estocástica (Poisson visual determinista) y velocidad `∝ √Kmax`, flechas por voltaje, inspector por forma, escenarios de demo y HUD.

## Pendiente

- [ ] Calibrar QE/área contra un cátodo real documentado (hoy editables, por defecto 1 cm² y 1 %).
- [ ] Capturas de demo en `docs/demo-*.png` + prueba de humo (protocolo en `docs/demo-checklist.md`).

## Historial

Este archivo sustituye al antiguo `Roadmap.md` de la raíz, que describía el estado inicial vacío del repositorio y ya no correspondía al código.
