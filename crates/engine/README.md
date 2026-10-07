# `fotoelectrico-engine` — renderer por capas

Propósito: dibujar la celda con Macroquad a partir de lo que `app` ya calculó. Recibe un `SceneFrame` y no calcula física.

## Contrato

`SceneFrame` trae viewport, estilo, capas visibles y lecturas listas para dibujar: `emission_possible`, `collection_possible`, `k_max_ev`, `electron_max_speed_m_s`.

Reglas visuales:

- Electrones: 0 si no hay colección; si la hay, cantidad ∝ intensidad y velocidad ∝ `√Kmax`.
- Fotones: color por banda de `λ`, densidad por intensidad.
- Flechas: orientación por voltaje aplicado.

Capas en `src/layers/`: `cell`, `photons`, `electrons`, `field`, `overlays`.

Detalles del flujo en [`docs/architecture.md`](../../docs/architecture.md).
