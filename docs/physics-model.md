# Modelo físico

Fuente de verdad del modelo ideal implementado en `fotoelectrico-physics`. Si el código y este archivo discrepan, manda el código y este archivo debe corregirse.

## Ecuaciones

Entradas: longitud de onda `λ` (nm) y material con función de trabajo `Φ`.

```text
f = c / λ
E = h · f
f₀ = Φ / h
λ₀ = c / f₀
emite ⟺ E > Φ + eps  (eps = 1e-25 J, anti-parpadeo en el umbral)
Kmax = max(h·f − Φ, 0)
V₀ = Kmax / e
v = √(2·Kmax / mₑ)   (0 si no hay emisión)
```

La regresión del experimento `V₀` contra `f` ajusta `V₀ = m·f + b` por mínimos cuadrados y estima `h = e·m`, con error porcentual frente a `h` teórica.

## Constantes (SI)

| Símbolo | Valor | Código |
|---|---|---|
| `h` | 6.62607015e-34 J·s | `constants::PLANCK_H` |
| `c` | 299792458 m/s | `constants::SPEED_OF_LIGHT_C` |
| `e` | 1.602176634e-19 C | `constants::ELEMENTARY_CHARGE_E` |
| `mₑ` | 9.1093837e-31 kg | `constants::ELECTRON_MASS_M` |

Conversiones: `ev_to_joules`, `joules_to_ev`, `nm_to_meters`, `meters_to_nm`. Los nombres de funciones y campos llevan unidades (`*_hz`, `*_nm`, `*_ev`, `*_j`, `*_v`, `*_m_s`).

## Materiales

Valores estándar de literatura usados en la app (`MaterialChoice::work_function_ev`). Pueden variar con superficie y condiciones; si se requiere rigor experimental, citar fuente por material.

| Cátodo | Símbolo | Φ (eV) |
|---|---|---|
| Potasio | K | 2.29 |
| Sodio | Na | 2.36 |
| Calcio | Ca | 2.87 |
| Zinc | Zn | 4.30 |
| Cobre | Cu | 4.70 |
| Platino | Pt | 5.65 |

## Supuestos de la app (puente, no física fundamental)

Definidos en `crates/app/src/physics_adapter.rs`:

- **Intensidad → flujo.** 100 % = 10 mW/cm² (100 W/m²). `flujo = P / E_fotón` en fotones/m²·s. La intensidad no cambia `Kmax` ni `V₀`; solo la cantidad de electrones y el brillo del haz.
- **Colección → corriente.** Hay colección si hay emisión, intensidad > 0.1 % y `V_aplicado ≥ −V₀`. Un voltaje de frenado más negativo bloquea la llegada al ánodo sin alterar `Kmax`.
- **Curva.** `Kmax(λ)` con 61 puntos uniformes entre 180 y 900 nm, evaluando `calculate_effect` por punto.
- **Animación.** Escala visual de velocidad `√(Kmax / 1 eV)` entre 0.5 y 2.2; cantidad de electrones por intensidad y colección. Es visualización, no integración de trayectorias.

## Robustez (sin fallas)

El motor jamás devuelve NaN/inf ni se "desconecta":

- `λ` se sanea a rango finito (física: 10–10 000 nm; app: 180–900 nm). Entradas NaN/inf/≤0 saturan a valores seguros.
- `Φ` inválida se satura a 2.36 eV en vez de romper el cálculo.
- `Kmax`/`V₀`/`v` son siempre finitos y ≥ 0; en el umbral son exactamente 0.
- El haz de fotones depende solo de capa + intensidad: `λ` alta o `V` invertido jamás ocultan la luz, solo los electrones (emisión/colección).
- Si no hay electrones, la UI explica el porqué (λ > λ₀, intensidad ≈ 0, o V < −V₀) en vez de mostrar desconexión.

## Limitaciones

- Modelo ideal a 0 K efectivo: sin ruido, sin eficiencia cuántica < 1, sin efectos de superficie ni temperatura.
- La regresión sobre puntos calculados con la `h` conocida verifica el modelo; no es una medición independiente salvo que se añada incertidumbre documentada.
- El voltaje solo modela bloqueo por frenado; no hay curva corriente-voltaje completa ni efectos espaciales de carga.
