# Guía de uso

## Controles (panel izquierdo)

- **Material del cátodo.** Fija `Φ`. Debajo se muestra su valor en eV.
- **Longitud de onda (180–900 nm).** Fija `f = c/λ` y `E = hf`. Barrer de UV a IR cruza el umbral de cada material.
- **Intensidad (%).** Fija el flujo de fotones (100 % = 10 mW/cm²). Subirla aumenta electrones y brillo, nunca `Kmax`.
- **Voltaje aplicado (−5…+5 V).** Si es más negativo que `−V₀`, la colección se bloquea: verás “Colección: No · bloqueada” y cero electrones, con el mismo `Kmax`.
- **Modelo de corriente.** Área del cátodo (cm²) y eficiencia cuántica (%) calibran la fotocorriente `I = e·Φ·A·QE·g(V)`.
- **Escenarios de demo.** “Cruzar el umbral”, “Frenado total” y “K frente a Pt” ajustan varios controles de un clic.
- **Comparación.** Segundo material evaluado con la misma `λ` e intensidad.

## Pestañas

- **Celda.** Barra espectral, toolbar de animación y canvas. Clic en haz, cátodo, ánodo, electrones o flechas para inspeccionar; el inspector se cierra si ocultas su capa.
- **Gráfica.** Curva `Kmax(λ)` del material actual con línea de umbral `λ₀`. Zoom con rueda, desplazamiento por arrastre, valores con cursor y tabla opcional.
- **Experimento.** Mide `V₀` a varias frecuencias: elige modo ideal o experimental (±ruido), captura el punto actual o lanza un barrido automático, ve la tabla (clic ○/● o en la gráfica para resaltar), el ajuste `V₀ = m·f + b` (pendiente, `h = e·m`, error %, R²), la gráfica con recta, barras de error, zoom y paneo, la curva I–V y exporta CSV. Guarda ajustes por material para comparar pendientes; la tabla sobrevive al cierre.
- **Comparar.** Principal frente a comparación: emisión, colección, `Φ`, `f₀`, `Kmax`, `V₀` y tamaño de curva.
- **Registro.** Historial de cambios con tiempo de sesión y notas del operador (no afectan la física), más exportación de la sesión completa a CSV. Las notas y la tabla del experimento se guardan al cerrar y se restauran al abrir.

## Lecturas (panel derecho)

| Lectura | Significado |
|---|---|
| Emisión | `hf > Φ` con la `λ` actual |
| Colección | Llegan al ánodo (no bloqueados por `V < −V₀`) |
| Frecuencia / Energía del fotón | `f`, `E = hf` |
| Función de trabajo / umbrales | `Φ`, `f₀`, `λ₀` del material |
| Kmax / V₀ | Energía máxima y potencial de frenado |
| Flujo de fotones | Fotones/m²·s por la intensidad actual |
| Fotocorriente | `I = e·Φ·A·QE·g(V)` estimada (A = 1 cm², QE = 1 % demo) |
| Velocidad máx. | `√(2·Kmax/mₑ)`; guía la animación |

## Atajos

`Ctrl+K` paleta · `F1` ayuda · `F11` presentación · `Ctrl+Z` / `Ctrl+Y` deshacer/rehacer. Los de animación y diagnóstico se reasignan en Preferencias.

## Ejemplos para la demo

1. Sodio, 400 nm, 55 %, 0 V → emisión sí, `Kmax > 0`, electrones en movimiento.
2. Misma configuración a 700 nm → emisión no, `Kmax = 0`, cero electrones (el flujo sigue > 0).
3. Volver a 400 nm y bajar `V` por debajo de `−V₀` → emisión sí, colección no.
4. Subir intensidad de 10 % a 90 % → mismo `Kmax`/`V₀`, más electrones.
5. En Experimento (modo ideal, sodio): captura 300, 350, 400 y 450 nm → R² ≈ 1 y error ≈ 0 %. Activa modo experimental ±3 % y repite: el error deja de ser cero.
