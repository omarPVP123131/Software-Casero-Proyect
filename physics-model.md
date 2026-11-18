# PhotoLab · Física y Motor del Efecto Fotoeléctrico

Documento técnico que describe cómo funciona la física del efecto fotoeléctrico y cómo la implementa el motor del proyecto **PhotoLab** (Software-Casero-Proyect).

> La lógica física vive en el crate `crates/physics` (`fotoelectrico-physics`) y **no se modifica**: es la única fuente de verdad. Todo lo demás (modelo, WASM, comandos Tauri, UI) la consume.

---

## 1. Descripción general

PhotoLab es un laboratorio interactivo del efecto fotoeléctrico. Permite al usuario cambiar el material del cátodo, la longitud de onda de la luz, la intensidad y el potencial aplicado, y observar en tiempo real:

- si hay emisión de electrones o no,
- la energía cinética máxima de los electrones,
- el potencial de frenado $V_0$,
- las gráficas $K_{max}(\lambda)$ y $V_0(f)$,
- un ajuste experimental para obtener la constante de Planck $h$ y la función trabajo $\Phi$.

La física está separada en capas:

```
UI (Svelte)  →  photolab-model (contrato + ajuste)  →  fotoelectrico-physics (ecuaciones)
```

Esto permite que el mismo motor se use en:

- **WASM** (canvas interactivo, sin IPC por frame),
- **comandos Tauri `invoke`** (escritorio / respaldo),
- **tests Rust** (40+ tests de física).

---

## 2. Fundamentos físicos

### 2.1 Ecuación de Einstein del efecto fotoeléctrico

Un fotón de frecuencia $f$ (o longitud de onda $\lambda$) transporta una energía:

$$E_{\text{fotón}} = h f = \frac{hc}{\lambda}$$

donde:

- $h \approx 6.62607015\times10^{-34}\ \text{J·s}$ (constante de Planck),
- $c \approx 2.99792458\times10^{8}\ \text{m/s}$ (velocidad de la luz).

Cuando el fotón incide en un metal con función trabajo $\Phi$, el electrón escapa con energía cinética máxima:

$$\boxed{K_{max} = hf - \Phi}$$

**Condición de emisión (frecuencia umbral):**

$$hf > \Phi \quad\Longleftrightarrow\quad f > f_0 = \frac{\Phi}{h} \quad\Longleftrightarrow\quad \lambda < \lambda_0 = \frac{hc}{\Phi}$$

Si $hf \le \Phi$, no hay emisión (el fotón no tiene energía suficiente).

### 2.2 Potencial de frenado

Aplicando un potencial $V$ que frena a los electrones, la energía cinética máxima se reduce. El potencial de frenado $V_0$ es el voltaje (negativo respecto al cátodo) que detiene a los electrones más energéticos:

$$eV_0 = K_{max} = hf - \Phi$$

De aquí se obtiene la relación lineal clave del experimento:

$$V_0(f) = \frac{h}{e} f - \frac{\Phi}{e}$$

Una recta de pendiente $h/e$ y ordenada en el origen $-\Phi/e$. Esta es la base del ajuste experimental para medir $h$.

### 2.3 Dependencia de la intensidad

La intensidad de la luz **no** cambia la energía de cada fotón (y por tanto no cambia $K_{max}$ ni $V_0$), pero sí el **número de fotones** → cambia la **corriente** (número de electrones emitidos por segundo). El motor la modela como factor de escala de la corriente, no de la energía.

### 2.4 Materiales y función trabajo

Cada material tiene su $\Phi$ (en eV). Ejemplos típicos del catálogo:

| Material     | $\Phi$ (eV) | $\lambda_0$ (nm) |
|--------------|:-----------:|:----------------:|
| Sodio (Na)   | ~2.28       | ~544             |
| Potasio (K)  | ~2.30       | ~539             |
| Zinc (Zn)    | ~4.30       | ~288             |
| Plata (Ag)   | ~4.73       | ~262             |
| Platino (Pt) | ~6.35       | ~195             |

(Los valores exactos los define el catálogo del crate `physics`; el frontend solo los consume vía `physics_material_catalog()`.)

---

## 3. Motor de física: arquitectura

### 3.1 Capas

```
crates/
├── physics/         fotoelectrico-physics  ← ecuaciones puras (sin UI, sin IO)
├── model/           photolab-model         ← contrato (Session/Readout/FitReport), ajuste V₀–f, CSV
├── tauri_commands/  comandos invoke puros  ← física + archivos
└── wasm/            los mismos comandos vía wasm-bindgen (motor del canvas)
```

- **`fotoelectrico-physics`**: structs y funciones puras. No conoce Svelte, Tauri ni WASM. Es la única fuente de verdad de las ecuaciones.
- **`photolab-model`**: define el contrato (`Session`, `Readout`, `FitReport`), genera los bindings TypeScript con `ts-rs`, y contiene el ajuste lineal $V_0(f)$ y la exportación CSV.
- **`tauri_commands`**: envuelve la física en comandos `invoke`.
- **`wasm`**: expone los mismos comandos a JS vía `wasm-bindgen` para el canvas.

### 3.2 Un solo camino de ejecución

```
controles Svelte → store `lab` (TS tipado)
   ├─ WASM  photolab_wasm::evaluate()   ← slider en vivo (sin IPC)
   └─ Tauri invoke physics_evaluate     ← respaldo / escritorio sin WASM
                    ↓
              photolab-model  →  fotoelectrico-physics
```

No hay IPC por frame. La física interactiva corre en WASM dentro del mismo hilo del canvas; `invoke` se reserva para disco (guardar/abrir sesión, exportar, autoguardado) y como respaldo del motor.

### 3.3 Sincronización con `dt` real

Todo el movimiento (electrones, animaciones de la escena) se integra con `dt` real acotado (máximo 50 ms) en `Scene.svelte`:

```ts
const dt = Math.min((now - last) / 1000, 0.05); // 50 ms máx.
```

**Consecuencia:** la misma física a 30, 60 o 144 Hz, con soporte de pausa y cámara lenta 0.25×. No hay dependencia de frames.

### 3.4 Optimización: la curva $K_{max}(\lambda)$ no se recalcula por frame

El proyecto original llamaba a `calculate_effect` 61×/frame. Aquí:

- La curva $K_{max}(\lambda)$ se calcula **solo al cambiar el material**.
- Las evaluaciones continuas (slider en vivo) van por WASM, coalescidas por frame.

### 3.5 Escala de velocidad (visualización)

La energía cinética se visualiza con una escala controlada:

- Escala $\sqrt{K_{max}/1\ \text{eV}}$ acotada entre 0.5 y 2.2, con leyenda permanente ("escala comprimida").
- Modo **log real** seleccionable en *Realismo*: $v = 10^5\ \text{m/s} \to 0.5\times$, $v = 10^7\ \text{m/s} \to 2.2\times$.

Esto evita que la comparación Na vs Pt aplaste la visualización, sin mentir sobre la física (la leyenda lo indica).

---

## 4. Comandos del motor (API)

| Comando | Entrada | Salida | Uso |
|---------|---------|--------|-----|
| `physics_evaluate(controlsJson, prefsJson)` | controles + preferencias | `EvaluateResponse` | evaluación puntual (slider en vivo) |
| `physics_material_catalog()` | — | `MaterialInfo[]` | catálogo de materiales |
| `experiment_fit(pointsJson)` | puntos $(f, V_0)$ | `FitReport \| null` | ajuste lineal $V_0(f)$ → $h$, $\Phi$ |
| `experiment_sweep(requestJson)` | rango de $\lambda$ | `SweepPoint[]` | barrido de la curva $K_{max}(\lambda)$ |
| `experiment_csv(experimentJson)` | experimento | `string` | exportación CSV |
| `session_normalize(sessionJson)` | sesión cruda | `Session` | normalización/migración |
| `save_session` / `load_session` / `export_file` | — | — | disco (cáscara Tauri) |
| `open_report_html(html, filename)` | HTML + nombre | ruta | informes PDF en escritorio |

### 4.1 Flujo de `physics_evaluate`

1. Recibe controles (material, $\lambda$, intensidad, $V$) y preferencias.
2. Normaliza unidades y valida rangos.
3. Llama a `fotoelectrico-physics`:
   - calcula $E_{\text{fotón}} = hc/\lambda$,
   - compara con $\Phi$,
   - si $E_{\text{fotón}} > \Phi$, calcula $K_{max} = E_{\text{fotón}} - \Phi$ y $V_0 = K_{max}/e$,
   - si no, marca no emisión.
4. Devuelve `EvaluateResponse` (energías, $V_0$, estado de emisión, explicación causal).

### 4.2 Ajuste experimental $V_0(f)$

`experiment_fit` recibe puntos experimentales $(f_i, V_{0,i})$ y realiza una regresión lineal:

$$V_0 = m f + b$$

De donde:

$$h_{\text{exp}} = m \cdot e, \qquad \Phi_{\text{exp}} = -b \cdot e$$

El `FitReport` incluye pendiente, ordenada, $h$ experimental (en J·s y eV·s), $\Phi$ experimental, y métricas de bondad del ajuste. Si no hay suficientes puntos o la regresión es degenerada, devuelve `null`.

### 4.3 Generación de tipos

Los structs de `photolab-model` llevan:

```rust
#[derive(TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
```

`ts-rs` genera 17 bindings TS en `frontend/src/lib/generated`. Los campos con sufijo de unidad llevan `#[serde(rename)]` explícito (`slopeVs`, `hExperimentalJs`, …) para que el nombre inferido por `ts-rs` coincida con el de `serde`.

---

## 5. Tests y verificación

| Suite | Comando | Resultado |
|-------|---------|-----------|
| Rust (física, modelo, ajuste, comandos) | `cargo test --workspace` | 56 ✓ |
| Lint | `cargo clippy --workspace -- -D warnings` | limpio |
| Frontend (puente WASM + store) | `npm run test` (Vitest) | 5/5 |
| E2E + snapshot visual | `npm run test:e2e` (Playwright) | 6/6 |
| Tipos Svelte | `npx svelte-check` | 0 errores |

El E2E cubre, entre otros:

- canvas pintado,
- snapshot visual de la escena (sodio 400 nm) contra el build estático → cualquier regresión de píxeles rompe CI,
- cambio a rojo sin emisión → bloqueo por frenado,
- paleta de comandos + escenario "Medir h" con ajuste,
- informe $V_0(f)$ como HTML autocontenido,
- tema de alto contraste.

---

## 6. Flujo de trabajo con la física

> **Regla de oro: `crates/physics` no se toca.**

- Cualquier cambio de contrato se hace en `crates/model`.
- Los structs llevan `#[derive(TS)]` + `#[ts(export)]`.
- Los tests del crate regeneran `frontend/src/lib/generated` vía `PHOTOLAB_TS_OUT`.
- `serde(rename_all = "camelCase")` debe coincidir con el nombre inferido por `ts-rs`.

Esto garantiza que la física original (40+ tests) permanezca intacta y que la UI siempre esté tipada contra el contrato real.

---

## 7. Accesibilidad del motor

El motor expone estados que la UI traduce a **color + icono + forma + texto** (nunca solo color). Las lecturas y la explicación causal se anuncian con `aria-live`. `prefers-reduced-motion` desactiva estelas/bloom y arranca en pausa, sin alterar la física subyacente.

---

## 8. Resumen

- **Física:** ecuación de Einstein $K_{max} = hf - \Phi$, potencial de frenado $eV_0 = K_{max}$, ajuste lineal $V_0(f) = (h/e)f - \Phi/e$.
- **Motor:** `fotoelectrico-physics` (puro) → `photolab-model` (contrato + ajuste) → WASM/`invoke`.
- **Rendimiento:** sin IPC por frame, `dt` real acotado, curva $K_{max}(\lambda)$ solo al cambiar material.
- **Verificación:** 56 tests Rust + 5 Vitest + 6 Playwright (incluye snapshot visual).
- **Principio:** la física no se modifica; los cambios de contrato van en `crates/model`.

---

Licencia MIT. La física proviene del proyecto original del mismo autor.
