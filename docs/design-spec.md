# PhotoLab · sistema de diseño "laboratorio óptico moderno"

Documento de referencia de la interfaz implementada en `frontend/src`. Cada
regla aquí tiene su token en `frontend/src/app.css` (bloque `:root` y temas).

> Capturas reales generadas con Playwright desde el build de producción:
> `assets/ui-dark-1600.png`, `assets/ui-light-ir-1600.png`,
> `assets/ui-contrast-experiment.png`, `assets/scene-hero.png`.

---

## 1. Principios

1. **Instrumento, no juguete.** Cifras tabulares, unidades siempre visibles y
   un único motor de física (`fotoelectrico-physics`) detrás de cada número.
2. **La luz manda.** El color de acento de la UI (slider, borde activo, foco,
   haz) es el color real de la longitud de onda (`spectrum.ts`, λ→RGB Bruton).
3. **Nada compite con la escena.** Fondos profundos, glass sutil y una sola
   zona de "color fuerte" por pantalla: la luz y el estado de emisión.
4. **Accesible por defecto.** Color + icono + forma + texto. Nunca solo color.

---

## 2. Tokens

### Espaciado (múltiplos de 4)

| Token | Valor | Uso |
|---|---|---|
| `--sp-1` | 4 px | separaciones finas |
| `--sp-2` | 8 px | gaps de tarjetas |
| `--sp-3` | 12 px | padding interno de paneles |
| `--sp-4` | 16 px | padding de secciones |
| `--sp-5/6/8` | 20/24/32 px | bloques grandes |

### Radios

`--r-sm: 8px` (controles), `--r-md: 12px` (tarjetas), `--r-lg: 16px`
(paneles), `--r-full` (chips y badges).

### Elevaciones (3)

- `--el-1`: borde + sombra mínima (reposo de tarjetas).
- `--el-2`: menús, tooltips, sparklines elevadas.
- `--el-3`: paleta de comandos, tutorial, modales.
- En tema claro se reducen; en alto contraste **desaparecen** (bordes
  blancos de 1 px en su lugar).

### Movimiento

| Token | Valor | Uso |
|---|---|---|
| `--dur-1` | 120 ms | hover / press |
| `--dur-2` | 200 ms | entrada de tarjetas (`.rise`), chevrons |
| `--dur-3` | 320 ms | colapso de paneles |
| `--ease` | `cubic-bezier(.2,.8,.2,1)` | curva única |

Con `prefers-reduced-motion: reduce` (o tema de alto contraste) se desactivan
estelas, bloom y `backdrop-filter`, y la animación arranca **pausada**
(reanudable con `Espacio` o ▶).

### Tipografía

- **Inter** 400/500/600/700 — UI. Escala global `--font-scale` (botones A−/A+,
  persistida en la sesión y en `localStorage`).
- **JetBrains Mono** 400/600 con `font-variant-numeric: tabular-nums` — todas
  las lecturas, ejes y tablas; las columnas no bailan al cambiar valores.
- **KaTeX** — ecuaciones (`Eq.svelte`, helper `renderEq`), nunca imágenes.
- Tamaños: 0.68em (micro etiquetas) → 1.28em (estado EMITE).

---

## 3. Temas

| Tema | Selector | Notas |
|---|---|---|
| `dark` (por defecto) | `[data-theme="dark"]` | `#07090F → #0D1220`, glass `blur(18px)` |
| `light` | `[data-theme="light"]` | fondos `#EEF2F8/#FFFFFF`, mismos acentos con contraste AA |
| `contrast` | `[data-theme="contrast"]` | negro/`#FFF`, sin blur ni sombras, acentos `#7DD3FC`/`#FBBF24` |

El canvas tiene **su propio fondo** (`--canvas-bg-0/1`) oscuro incluso en tema
claro: es una cámara de vacío, no una hoja (mejor contraste para el haz). El
resto de la UI respeta el tema.

### Paleta apta para daltonismo

El estado de emisión se codifica con **forma + icono + texto**:

| Estado | Color (oscuro) | Icono/forma | Texto |
|---|---|---|---|
| EMITE | `#60A5FA` (azul) | `⬤` círculo lleno | EMITE |
| BLOQUEADO | `#F59E0B` (ámbar) | `⊘` barra de frenado | BLOQUEADO |
| NO EMITE | `#94A3B8` (gris) | `○` círculo vacío | NO EMITE |

Se evita el par rojo/verde como única señal. En el canvas, "emisión" además
tiene destello en el cátodo, "bloqueo" invierte la flecha del campo, y "sin
emisión" dibuja absorción/rebote con el cartel `E = … < Φ = …`.

---

## 4. Componentes clave

### Slider de espectro (`controls/SpectrumSlider.svelte`)
El elemento más distintivo: el riel **es** el espectro visible 380–780 nm con
UV/IR a los extremos; el thumb es una barra vertical del color exacto de λ;
una marca discontinua ámbar señala λ₀ del material; en UV/IR aparecen rayas y
la etiqueta "no visible". Entrada numérica sincronizada en nm/THz/eV y chips
rápidos (UV 254, violeta 405, verde 532, rojo 650, IR 850).

### Tarjetas de material
Cada cátodo (K, Na, Ca, Zn, Cu, Pt) muestra Φ, λ₀ y un mini-espectro con la
marca del umbral. La comparación (`select`) pinta una segunda curva
discontinua en las gráficas.

### Canvas (`Scene.svelte`)
- Fotones: paquetes de onda con **λ visible como longitud de onda** y densidad
  ∝ intensidad. UV/IR: paquete rayado.
- Electrones: partículas con estela; cantidad ∝ flujo; rapidez ∝ escala
  (√(K/1 eV) acotada 0.5–2.2 o log real, con leyenda permanente). Con frenado
  se desaceleran y **regresan** (no desaparecen).
- Diagrama de energías: barra apilada **Φ + Kmax = E** con déficit marcado
  si `E < Φ`. Es el apoyo didáctico central.
- Medidor de corriente, campo eléctrico con flechas, polaridad animada de la
  batería, medidor I lateral y lectura bajo el cursor.

### Gráficas (`graphs/Plot.svelte`)
Canvas 2D con ejes **siempre** etiquetados con unidades, cursor con lectura
`(x, y)`, zoom (rueda) y pan (arrastre), reset, export **PNG** y **SVG**
(vectorial, independiente del canvas). La pestaña Experimento añade barras de
error, intervalo de confianza de la recta, h ± σ, error % y R², residuos y
export CSV/PDF.

### Lecturas (`ReadoutsPanel.svelte`)
Estado grande + explicación causal en una frase generada desde el motor
(`causal.ts`) con KaTeX; tarjetas con sparkline del histórico reciente,
copiar y fijar valores; comparación con el segundo material; capas.

### Tutorial (`Tutorial.svelte`)
Pasos con pregunta de predicción obligatoria ("¿qué pasará si duplico la
intensidad?") respondida **antes** de revelar el resultado; botón "Preparar
paso" que deja los controles en el estado correcto.

---

## 5. Distribución

```
┌──────────────────────────────────────────────────────────────┐
│ logo · escenario · ⌘K · sesión · presentación · undo · tema  │
├────────┬─────────────────────────────────────┬───────────────┤
│Control.│  Celda │ I–V │ Kmax(λ) │ Experim.   │  Lecturas     │
│ 300 px │            Canvas 2D                │  308 px       │
│(drag)  │                                     │  (drag)       │
└────────┴─────────────────────────────────────┴───────────────┘
│ motor OK · WASM · ideal · 60 fps · escala · unidades         │
```

- Ancho mínimo de ventana: **1280×800** (`tauri.conf.json`; mínimo absoluto
  1024×680). Por debajo de 1100 px de CSS los paneles laterales se colapsan.
- Los separadores son accesibles (`role="separator"`, `aria-orientation`).
- Modo presentación (F11): oculta paneles, sube `--font-scale` a 1.15.

---

## 6. Atajos

| Acción | Atajo |
|---|---|
| Paleta de comandos | `Ctrl/⌘ + K` |
| Deshacer / rehacer | `Ctrl/⌘+Z` · `Ctrl/⌘+Shift+Z` |
| Pausa / reanudar | `Espacio` |
| Reiniciar tiempo | `R` |
| Paso de 1/30 s | `Shift + →` |
| Pestañas | `1` `2` `3` `4` |
| Presentación | `F11` |
| Ajuste fino de sliders | `Shift` + flechas (×10) |

Los sliders numéricos se pueden **arrastrar** sobre su etiqueta (scrub) y
**doble clic** restablece.

---

## 7. De egui + Macroquad a Tauri 2

```
Controles Svelte → store session (TS tipado ts-rs)
       → photolab-wasm (Rust, mismo crate que Tauri)
       → photolab-model → fotoelectrico-physics
```

- **Un solo toolkit de UI**: desaparece el doble bucle eventos/DPI del
  `egui-macroquad` original; ya no hay textura espejo ni ventana compartida.
- **Sin IPC por frame**: la física corre en WASM en el mismo hilo del canvas;
  `invoke` se reserva a disco (guardar/abrir sesión, autoguardado, export) y
  como respaldo del motor si el WASM no carga.
- **Tipos generados**: si cambia un struct de Rust, `svelte-check` falla
  (`scripts/gen-types.sh`).
- **Permisos mínimos**: `capabilities/default.json` solo concede core, dialog,
  fs (lectura/escritura de texto) y opener.

---

## 8. Rendimiento y accesibilidad

- Canvas en `requestAnimationFrame` con **dt real acotado** (50 ms), pausa y
  slow-motion; nada depende del número de frame.
- `evaluate()` coalescido por tick: 60 eventos de slider/s → ~1 cálculo por
  frame; la curva Kmax(λ) solo cambia cuando cambia el material.
- Partículas derivadas determinísticamente de `(t, λ, I)`: sin asignaciones por
  partícula y estable para snapshots de Playwright.
- `aria-live` con el estado y la explicación causal; foco visible en todo;
  navegación completa por teclado (tabs con flechas, slider con flechas,
  paleta con ↑/↓/↵).
- `prefers-reduced-motion` respetado: sin estelas, sin bloom, animación
  inicialmente pausada.
