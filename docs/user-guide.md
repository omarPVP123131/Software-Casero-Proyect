# Guía de uso

## Controles (panel izquierdo)

- **Material del cátodo.** Fija `Φ`. Debajo se muestra su valor en eV.
- **Longitud de onda (180–900 nm).** Fija `f = c/λ` y `E = hf`. Barrer de UV a IR cruza el umbral de cada material.
- **Intensidad (%).** Fija el flujo de fotones (100 % = 10 mW/cm²). Subirla aumenta electrones y brillo, nunca `Kmax`.
- **Voltaje aplicado (−5…+5 V).** Si es más negativo que `−V₀`, la colección se bloquea: verás “Colección: No · bloqueada” y cero electrones, con el mismo `Kmax`.
- **Comparación.** Segundo material evaluado con la misma `λ` e intensidad.

## Pestañas

- **Celda.** Barra espectral, toolbar de animación y canvas. Clic en haz, cátodo, ánodo, electrones o flechas para inspeccionar; el inspector se cierra si ocultas su capa.
- **Gráfica.** Curva `Kmax(λ)` del material actual con línea de umbral `λ₀`. Zoom con rueda, desplazamiento por arrastre, valores con cursor y tabla opcional.
- **Comparar.** Principal frente a comparación: emisión, colección, `Φ`, `f₀`, `Kmax`, `V₀` y tamaño de curva.
- **Registro.** Historial de cambios con tiempo de sesión y notas del operador (no afectan la física).

## Lecturas (panel derecho)

| Lectura | Significado |
|---|---|
| Emisión | `hf > Φ` con la `λ` actual |
| Colección | Llegan al ánodo (no bloqueados por `V < −V₀`) |
| Frecuencia / Energía del fotón | `f`, `E = hf` |
| Función de trabajo / umbrales | `Φ`, `f₀`, `λ₀` del material |
| Kmax / V₀ | Energía máxima y potencial de frenado |
| Flujo de fotones | Fotones/m²·s por la intensidad actual |
| Velocidad máx. | `√(2·Kmax/mₑ)`; guía la animación |

## Atajos

`Ctrl+K` paleta · `F1` ayuda · `F11` presentación · `Ctrl+Z` / `Ctrl+Y` deshacer/rehacer. Los de animación y diagnóstico se reasignan en Preferencias.

## Ejemplos para la demo

1. Sodio, 400 nm, 55 %, 0 V → emisión sí, `Kmax > 0`, electrones en movimiento.
2. Misma configuración a 700 nm → emisión no, `Kmax = 0`, cero electrones (el flujo sigue > 0).
3. Volver a 400 nm y bajar `V` por debajo de `−V₀` → emisión sí, colección no.
4. Subir intensidad de 10 % a 90 % → mismo `Kmax`/`V₀`, más electrones.
