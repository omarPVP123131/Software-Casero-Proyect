# Protocolo de demo (prueba de humo, 5 min)

Verifica en el equipo de exposición que la app compila, abre y mide bien.
Marca cada paso; si un valor difiere > 2 %, no presentes: revisa el modelo.

## 1. Arranque

```bash
cargo run -p fotoelectrico-app
```

- [ ] La ventana abre a 1440×900 sin pánico en consola.
- [ ] El footer dice `MOTOR FÍSICO CONECTADO`.
- [ ] La pestaña Celda muestra haz, cátodo, ánodo y electrones en movimiento.

## 2. Umbral (sodio, 55 %, 0 V)

| λ (nm) | Emisión esperada | V₀ esperado (V) |
|---|---|---|
| 400 | Sí | ≈ 0.74 |
| 525 | Límite (≈ λ₀ = 525.4) | ≈ 0.00 |
| 700 | No | 0.00 |

- [ ] A 700 nm el panel explica “λ > λ0” y el haz sigue visible.
- [ ] Escenario “Cruzar el umbral”: mueve λ ±20 nm y cruza emisión/no-emisión.

## 3. Frenado (sodio, 400 nm, 55 %)

- [ ] Escenario “Frenado total”: colección “No · bloqueada”, corriente 0, `Kmax` intacta (≈ 0.74 eV).
- [ ] Sube V a 0: corriente ≈ 17.7 µA (A = 1 cm², QE = 1 %).

## 4. Contraste de materiales

- [ ] Escenario “K frente a Pt” a 400 nm: potasio emite, platino no.

## 5. Experimento (modo ideal, sodio)

Captura 300, 350, 400 y 450 nm.

- [ ] R² = 1.0000 y error de `h` < 0.01 %.
- [ ] “Guardar CSV del experimento” crea `experimento_v0_f.csv` con 4 filas.

## 6. Captura para el README

1. Escenario “Cruzar el umbral”, pestaña Celda, tema Midnight.
2. Captura a 1440×900 y guárdala como `docs/demo-celda.png`.
3. Repite en Experimento con 4+ puntos y la recta visible (`docs/demo-experimento.png`).

## 7. Cierre

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

- [ ] Todo verde antes de exponer.
