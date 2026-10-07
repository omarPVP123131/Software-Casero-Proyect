# Plan paso a paso — Simulador del efecto fotoeléctrico

## Estado inicial

El repositorio actualmente contiene el `README.md` con la propuesta, pero todavía no incluye `Cargo.toml`, código Rust ni pruebas. La meta es avanzar en capas: primero comprobar la física con pruebas y después añadir interfaz, animación y análisis.

## Qué debería incluir el primer prototipo

Para tener algo que se pueda demostrar sin intentar construir todo de una vez:

1. Elegir un material y cambiar la frecuencia **o** la longitud de onda de la luz.
2. Mostrar si hay emisión, la frecuencia umbral, la energía cinética máxima y el potencial de frenado.
3. Cambiar la intensidad y explicar que cambia la cantidad de fotones/electrones, no la energía máxima de cada fotoelectrón.
4. Recorrer varias frecuencias, mostrar una tabla de `V₀` contra `f` y estimar `h` con una regresión lineal.
5. Añadir la animación de la celda después de que los cálculos anteriores estén probados.

La animación de trayectorias, las curvas completas de corriente y el modelo estocástico más detallado pueden quedar para una versión posterior.

## Fases de trabajo

### 1. Preparar el proyecto

- Instalar Rust con `rustup` y comprobar `rustc --version` y `cargo --version`.
- Crear un Cargo workspace con tres crates, siguiendo la arquitectura del README:
  - `crates/physics`: cálculos y modelos; sin dependencias gráficas.
  - `crates/engine`: dibujo y animación con Macroquad.
  - `crates/app`: controles, integración y entrada principal.
- Crear `docs/` para fuentes, decisiones del modelo y guía de uso.
- Hacer que el primer objetivo compile con `cargo build --workspace` antes de añadir funcionalidades.

### 2. Implementar el núcleo físico

Mantener las unidades explícitas en los nombres y usar SI para los cálculos internos. Al mostrar resultados, convertir a eV, nm o V según corresponda.

Constantes iniciales:

- Constante de Planck `h` en J·s.
- Velocidad de la luz `c` en m/s.
- Carga elemental `e` en C.
- Masa del electrón `mₑ` en kg.

Modelos y cálculos:

- `Material`: nombre y función de trabajo `Φ`.
- `Photon`: frecuencia o longitud de onda.
- `Kmax = max(hf − Φ, 0)`.
- `f₀ = Φ/h`.
- `V₀ = Kmax/e = (hf − Φ)/e` cuando existe emisión.

Conviene escoger **una sola variable de entrada** en la interfaz —por ejemplo, longitud de onda— y derivar la frecuencia con `f = c/λ`. Así se evita que el usuario introduzca dos valores incompatibles.

### 3. Añadir pruebas antes de dibujar

Pruebas mínimas para `physics`:

- Una frecuencia menor que `f₀` no produce fotoelectrones.
- En el umbral, `Kmax` es cero; por encima del umbral, es positiva.
- Al aumentar la frecuencia, aumenta `Kmax`.
- Cambiar intensidad no cambia `Kmax` ni `V₀` en el modelo ideal.
- La conversión entre longitud de onda y frecuencia es consistente.
- La función de trabajo y las conversiones eV↔J se aplican correctamente.

Para reducir errores, las funciones deberían tener nombres que indiquen sus unidades, por ejemplo `frequency_hz`, `wavelength_m` y `work_function_ev`.

### 4. Construir la interfaz mínima

En `app`, añadir controles para material, longitud de onda/frecuencia, intensidad y voltaje aplicado. Mostrar en pantalla:

- Material y función de trabajo.
- Frecuencia umbral.
- Energía del fotón.
- Emisión: sí/no.
- `Kmax` y potencial de frenado.

Primero presentar números y un esquema sencillo de la celda. Después añadir fotones y electrones animados con Macroquad. Si el movimiento se implementa, definir claramente qué electrodo es positivo y el sentido del campo; el electrón tiene carga negativa, por lo que su aceleración va en sentido opuesto a `E`.

### 5. Hacer el experimento de `V₀` contra `f`

- Generar varios puntos por encima de la frecuencia umbral del material.
- Guardar cada par `(f, V₀)` en una tabla.
- Ajustar `V₀ = mf + b` mediante mínimos cuadrados.
- Calcular `h_estimado = e·m` y comparar con el valor de referencia mediante error porcentual.
- Etiquetar frecuencia en Hz, potencial en V y pendiente en V·s.

**Importante:** si los puntos se calculan directamente con el valor conocido de `h`, la regresión devolverá ese mismo valor salvo redondeo. Para llamarlo experimento virtual, hay que añadir una incertidumbre o ruido de medición documentado; si no, presentarlo como una verificación del modelo ideal, no como una medición independiente.

### 6. Exportar, validar y documentar

- Exportar los datos de la tabla a CSV con encabezados y unidades.
- Documentar el modelo ideal, sus limitaciones y las fuentes de las funciones de trabajo.
- Validar el proyecto con `cargo fmt --check`, `cargo test --workspace` y `cargo clippy --workspace -- -D warnings`.
- Probar la compilación y la ejecución en el equipo donde se hará la demostración.
- Actualizar el README para marcar como implementadas solo las funciones que realmente estén listas.

## Orden recomendado de entregas

1. **v0.1 — Física:** workspace, materiales, ecuaciones y pruebas.
2. **v0.2 — Interfaz:** controles y resultados en tiempo real, sin animación compleja.
3. **v0.3 — Experimento:** tabla, regresión de `V₀` contra `f`, cálculo de `h` y CSV.
4. **v0.4 — Visualización:** celda, fotones, electrones y campo eléctrico.
5. **v1.0 — Entrega académica:** validación, fuentes, documentación y presentación.

## Si se mantiene la fecha objetivo del README

El README indica como objetivo el **8 de octubre de 2026**. Desde el **6 de octubre de 2026**, quedan dos días: es más realista presentar un prototipo con física probada, controles y una tabla/regresión que prometer todas las funciones de `v1.0`. La animación avanzada y las curvas adicionales deberían ser secundarias hasta que el núcleo compile y pase sus pruebas.

## Correcciones que conviene hacer en el README

- Escribir con claridad `f₀ = Φ/h`.
- Escribir `V₀ = (hf − Φ)/e` y `h_estimado = e·m`, donde `m` es la pendiente de `V₀` contra `f`.
- Unificar las versiones de Rust indicadas: el README menciona tanto `1.99.0` como `1.97.1`; usar una única versión comprobada o indicar simplemente Rust estable.
- Sustituir la URL de clonación de ejemplo (`tu-usuario/fotoelectrico.git`) por la URL real del repositorio.
- Añadir referencias para las funciones de trabajo; pueden variar según la superficie y las condiciones del material.
