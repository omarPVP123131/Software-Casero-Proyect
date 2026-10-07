# PhotoLab — Laboratorio del efecto fotoeléctrico

Simulador interactivo del efecto fotoeléctrico en Rust: ajusta material, luz y voltaje, y observa en tiempo real si hay emisión, cuánta energía llevan los fotoelectrones y cómo se frenan con el potencial de retardo.

Motor físico conectado (`Kmax = hf − Φ`). La interfaz no estima valores: todo número visible sale de `fotoelectrico-physics`.

## Inicio rápido

Requisito: Rust estable (probado con 1.99.0).

```bash
cargo run -p fotoelectrico-app
```

Verificación completa:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo build --workspace
```

## Qué puedes hacer

- Elegir cátodo (K, Na, Ca, Zn, Cu, Pt) y barrer longitud de onda 180–900 nm.
- Ver emisión, `f₀`, `λ₀`, energía del fotón, `Kmax`, `V₀`, flujo de fotones, fotocorriente y velocidad del electrón.
- Comprobar que la intensidad cambia la cantidad de electrones, no su energía.
- Aplicar voltaje de frenado: si `V < −V₀` la colección se bloquea sin cambiar `Kmax`; la curva I–V muestra la rampa.
- Explorar la curva `Kmax(λ)`, comparar dos materiales y registrar la sesión.
- Medir `V₀` a varias frecuencias en la pestaña Experimento (modo ideal o con ruido), ajustar `V₀ = m·f + b`, estimar `h` y exportar CSV.
- Personalizar tema, paneles, capas del canvas, atajos y modo presentación.

Detalles de uso en [`docs/user-guide.md`](docs/user-guide.md).

## Arquitectura

| Crate | Responsabilidad | No hace |
|---|---|---|
| `fotoelectrico-physics` | Ecuaciones, constantes SI, materiales, regresión de `h` | Nada gráfico |
| `fotoelectrico-app` | Controles, lecturas, `physics_adapter` (único puente al motor) | No duplica ecuaciones |
| `fotoelectrico-engine` | Dibujo Macroquad por capas a partir de `SceneFrame` | No calcula física |

Flujo de datos:

```text
controles (λ, material, intensidad, V)
  → physics_adapter::refresh_state()
  → PhysicsReadout + curva Kmax(λ)
  → SceneFrame { emisión, colección, Kmax, velocidad }
  → engine renderiza (cantidad ∝ intensidad, velocidad ∝ √Kmax)
```

Contrato y decisiones en [`docs/architecture.md`](docs/architecture.md).

## Modelo físico

```text
f = c / λ          E = h·f
f₀ = Φ / h         λ₀ = c / f₀
Kmax = max(h·f − Φ, 0)
V₀ = Kmax / e      v = √(2·Kmax / mₑ)
```

- 100 % de intensidad = 10 mW/cm² (100 W/m²); `flujo = P / E_fotón`.
- La regresión `V₀ = m·f + b` estima `h = e·m` (ver `physics::fit`).

Especificación completa, materiales y limitaciones en [`docs/physics-model.md`](docs/physics-model.md).

## Estructura

```text
README.md               Este archivo: visión general y arranque
LICENSE                 Licencia MIT
docs/                   Documentación por propósito (ver docs/README.md)
  physics-model.md      Ecuaciones, materiales y supuestos
  architecture.md       Crates, flujo de datos e invariantes
  user-guide.md         Guía de uso de la aplicación
  development.md        Entorno, comandos y convenciones
  roadmap.md            Estado actual y siguientes pasos
crates/physics/         Núcleo físico + pruebas
crates/app/             Interfaz egui + puente al motor
crates/engine/          Renderer Macroquad por capas
```

Cada `.md` existe para responder una pregunta concreta; el índice está en [`docs/README.md`](docs/README.md).

## Estado

Física, interfaz, experimento `V₀–f` con regresión y exportación CSV, y visualización conectadas y probadas. Ver [`docs/roadmap.md`](docs/roadmap.md).

## Licencia y autoría

MIT — ver [`LICENSE`](LICENSE). Autores en el encabezado de `LICENSE`.
