# `fotoelectrico-physics` — núcleo físico

Propósito: única fuente de verdad para el modelo del efecto fotoeléctrico. Sin dependencias gráficas. Todo cálculo interno en SI; la presentación convierte a eV, nm y V.

## API

```rust
use fotoelectrico_physics::{Material, calculate_effect, get_physics_readout};

let material = Material::new("Sodio", 2.36); // Φ en eV
let r = calculate_effect(&material, 400.0);  // λ en nm
assert!(r.emits_electron);

let readout = get_physics_readout(&material, 400.0);
// readout.photon_energy_ev, .k_max_ev, .stopping_potential_v,
// .threshold_frequency_hz, .electron_max_speed_m_s, ...
```

Módulos: `constants` (h, c, e, mₑ + conversiones), `material` (`f₀ = Φ/h`, `λ₀ = c/f₀`), `photon` (`E = hf`, `Kmax`, `V₀`), `fit` (regresión `V₀` contra `f` → `h = e·m`).

## Probar

```bash
cargo test -p fotoelectrico-physics
```

Cubre: no emisión bajo el umbral, `Kmax = 0` en el umbral, `Kmax` crece con `f`, consistencia `λ ↔ f` y regresión con error < 1 % en datos ideales.

Modelo completo, tabla de materiales y limitaciones en [`docs/physics-model.md`](../../docs/physics-model.md).
