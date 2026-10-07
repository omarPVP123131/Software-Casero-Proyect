# Simulador del Efecto Fotoeléctrico

> **Simulación interactiva y experimental del efecto fotoeléctrico desarrollada en Rust para la asignatura de Física — UAEMéx.**

Proyecto académico desarrollado por estudiantes de la **Universidad Autónoma del Estado de México (UAEMéx), Centro Universitario Valle de Chalco**, cuyo objetivo es construir un simulador interactivo capaz de representar el fenómeno fotoeléctrico mediante un modelo físico computacional y permitir la realización de experimentos virtuales relacionados con la **constante de Planck**.

El proyecto será desarrollado completamente en **Rust**, utilizando **Macroquad** como base del motor de representación y **egui** para la interfaz de usuario.

> 🚧 **Estado actual: Pre-alpha / En desarrollo**
>
> El proyecto se encuentra actualmente en fase de planificación y arquitectura. La implementación aún no ha comenzado.

---

## 📚 Información académica

|                          |                                           |
| ------------------------ | ----------------------------------------- |
| **Institución**          | Universidad Autónoma del Estado de México |
| **Centro Universitario** | Valle de Chalco                           |
| **Asignatura**           | Física                                    |
| **Proyecto**             | Simulador del Efecto Fotoeléctrico        |
| **Lenguaje**             | Rust                                      |
| **Toolchain**            | Rust `1.99.0`                             |
| **Renderizado**          | OpenGL                                    |
| **Framework gráfico**    | Macroquad                                 |
| **Interfaz**             | egui                                      |
| **Licencia**             | MIT                                       |
| **Release objetivo**     | `v1.0.0` — 8 de octubre de 2026           |

### 👥 Equipo

* **Omar Palomares Velasco**
* **Leonardo Miguel Vega Carbajal**

---

# 🎯 Objetivo

El proyecto busca transformar el modelo matemático del efecto fotoeléctrico en una experiencia experimental interactiva.

El usuario podrá modificar las condiciones del experimento y observar cómo cambian las variables físicas del sistema, permitiendo estudiar experimentalmente relaciones como:

$$
E_\gamma = hf
$$

$$
K_{\max}=hf-\Phi
$$

y

$$
V_0=\frac{K_{\max}}{e}
$$

La meta final será utilizar los datos generados por la simulación para obtener una estimación experimental de la constante de Planck:

$$
\boxed{h_{\text{exp}}=me}
$$

donde \(m\) corresponde a la pendiente obtenida mediante regresión lineal de:

$$
V_0 \text{ vs. } f
$$

---

# 🔬 ¿Qué se pretende simular?

La simulación representará una celda fotoeléctrica simplificada formada por un **cátodo emisor** y un **ánodo colector**.

Conceptualmente:

```text
                    Fotones
                ↓   ↓   ↓   ↓
              ↘  ↘  ↘  ↘  ↘

        ┌──────────────────────────┐
        │         CÁTODO           │
        │       Material M         │
        └──────────────────────────┘
                    ↑
                 e⁻ │
                   ↗
                 e⁻ │
                   ↗
                 e⁻ │
                   ↗
        ┌──────────────────────────┐
        │          ÁNODO           │
        └──────────────────────────┘

                 Campo eléctrico
                     E →
```

La simulación tendrá como propósito visualizar:

* Fotones incidentes.
* Emisión de fotoelectrones.
* Movimiento de los electrones.
* Influencia del campo eléctrico.
* Corriente fotoeléctrica.
* Potencial de frenado.
* Relación entre frecuencia e intensidad.
* Dependencia con la función de trabajo del material.

---

# ⚙️ Variables experimentales

El usuario podrá interactuar con los principales parámetros del experimento.

### Frecuencia

$$
f
$$

Determina la energía individual de cada fotón:

$$
E_\gamma=hf
$$

### Longitud de onda

$$
\lambda
$$

Relacionada con la frecuencia mediante:

$$
c=f\lambda
$$

### Intensidad

La intensidad representará la tasa de llegada de fotones al material.

En el modelo ideal, aumentar la intensidad incrementará el número de electrones emitidos, pero **no la energía máxima individual de los fotoelectrones**.

### Voltaje

$$
V
$$

Permitirá modificar el campo eléctrico entre el cátodo y el ánodo y estudiar tanto la aceleración como el frenado de los electrones.

### Material

La selección del material modificará su función de trabajo:

$$
\Phi
$$

y, por consecuencia, su frecuencia umbral:

$$
f_0=\frac{\Phi}{h}
$$

---

# 🧠 Modelo físico

El simulador estará basado en la explicación cuántica propuesta por **Albert Einstein** para el efecto fotoeléctrico.

## Energía del fotón

$$
E_\gamma=hf
$$

o equivalentemente:

$$
E_\gamma=\frac{hc}{\lambda}
$$

---

## Ecuación fotoeléctrica

Cuando la energía del fotón supera la función de trabajo del material:

$$
hf>\Phi
$$

el electrón puede escapar de la superficie.

La energía cinética máxima será:

$$
\boxed{K_{\max}=hf-\Phi}
$$

---

## Frecuencia umbral

La frecuencia mínima necesaria para producir emisión será:

$$
\boxed{f_0=\frac{\Phi}{h}}
$$

Para:

$$
f<f_0
$$

el modelo no producirá emisión fotoeléctrica.

---

## Potencial de frenado

El potencial necesario para detener los electrones más energéticos satisface:

$$
eV_0=K_{\max}
$$

por lo tanto:

$$
\boxed{
V_0=
\frac{h}{e}f-\frac{\Phi}{e}
}
$$

Esta ecuación será fundamental para el experimento virtual de determinación de \(h\).

---

# 🧪 Experimento virtual de Millikan

Una de las funciones principales previstas será la reproducción computacional del método utilizado históricamente para estudiar la relación entre la frecuencia de la luz y el potencial de frenado.

### Procedimiento previsto

1. Seleccionar un material.
2. Seleccionar una frecuencia superior a la frecuencia umbral.
3. Simular la emisión de fotoelectrones.
4. Aplicar un potencial de frenado.
5. Incrementar el potencial hasta detener los electrones que alcanzan el ánodo.
6. Registrar \(V_0\).
7. Repetir el procedimiento para diferentes frecuencias.
8. Construir la gráfica:

$$
V_0 \text{ vs. } f
$$

9. Realizar una regresión lineal.
10. Obtener la pendiente:

$$
m=\frac{h}{e}
$$

11. Calcular:

$$
\boxed{h_{\text{exp}}=me}
$$

Finalmente, el resultado podrá compararse con el valor aceptado de la constante de Planck.

---

# 📊 Análisis experimental

El proyecto contempla la generación de diferentes curvas características:

### Corriente vs. voltaje

$$
I(V)
$$

Permitirá estudiar el comportamiento de la corriente fotoeléctrica al modificar el potencial aplicado.

### Energía cinética máxima vs. frecuencia

$$
K_{\max}(f)
$$

Permitirá observar la relación lineal predicha por Einstein.

### Potencial de frenado vs. frecuencia

$$
V_0(f)
$$

Será la gráfica principal para la determinación experimental de \(h\).

### Corriente de saturación vs. intensidad

$$
I_{\text{sat}}(I)
$$

Permitirá estudiar la relación entre intensidad luminosa y número de electrones emitidos.

---

# 🧮 Modelo computacional

Para mantener un equilibrio entre rendimiento, claridad y utilidad educativa, el proyecto utilizará inicialmente un modelo bidimensional.

### Distribución de energía

Los electrones emitidos podrán recibir una energía cinética inicial dentro de:

$$
K\in[0,K_{\max}]
$$

mediante una distribución uniforme.

### Distribución angular

La emisión se modelará inicialmente mediante una distribución angular basada en la **ley del coseno de Lambert**.

### Movimiento

El estado de cada electrón estará representado mediante variables como:

$$
\vec{x}=(x,y)
$$

$$
\vec{v}=(v_x,v_y)
$$

y su evolución será calculada mediante un integrador numérico.

Para un campo eléctrico uniforme:

$$
\vec{F}=q\vec{E}
$$

y:

$$
\vec{a}=\frac{q\vec{E}}{m_e}
$$

---

# 🧱 Arquitectura

El proyecto se organizará como un **Cargo Workspace** con separación entre el modelo físico, el motor de representación y la aplicación.

```text
fotoelectrico/
│
├── Cargo.toml
│
├── crates/
│   │
│   ├── physics/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── constants.rs
│   │   │   ├── material.rs
│   │   │   ├── photon.rs
│   │   │   ├── electron.rs
│   │   │   ├── cell.rs
│   │   │   ├── emission.rs
│   │   │   ├── integrator.rs
│   │   │   ├── sweep.rs
│   │   │   ├── fit.rs
│   │   │   └── sim.rs
│   │   │
│   │   └── tests/
│   │
│   ├── engine/
│   │   └── src/
│   │
│   └── app/
│       └── src/
│
└── docs/
```

### `physics`

Núcleo científico del proyecto.

Se encargará de:

* Constantes físicas.
* Materiales.
* Fotones.
* Electrones.
* Emisión.
* Campo eléctrico.
* Integración numérica.
* Experimentos.
* Barridos.
* Regresión.
* Cálculos físicos.

El objetivo es mantener este crate **independiente del renderizado**.

---

### `engine`

Capa encargada de la representación gráfica.

Tecnologías previstas:

* **Macroquad**
* **OpenGL**

Será responsable de representar:

* Celda fotoeléctrica.
* Fotones.
* Electrones.
* Trayectorias.
* Elementos visuales de la simulación.

---

### `app`

Capa superior de la aplicación.

Se encargará de:

* Loop principal.
* Interfaz.
* Controles experimentales.
* Integración entre `physics` y `engine`.
* Gráficas.
* Exportación de datos.

La interfaz utilizará **egui**.

---

# 🛠️ Tecnologías

| Tecnología      | Uso                   |
| --------------- | --------------------- |
| **Rust 1.97.1** | Lenguaje principal    |
| **Cargo**       | Gestión y compilación |
| **Macroquad**   | Framework gráfico     |
| **OpenGL**      | Renderizado           |
| **egui**        | Interfaz gráfica      |
| **CSV**         | Exportación de datos  |

---

# 🚀 Compilación

## Requisitos

Antes de comenzar necesitas:

* Rust `1.97.1` o superior.
* Cargo.
* Una GPU/controlador compatible con el backend gráfico utilizado por Macroquad.

Comprueba la instalación:

```bash
rustc --version
cargo --version
```

---

## Clonar

```bash
git clone https://github.com/tu-usuario/fotoelectrico.git
cd fotoelectrico
```

---

## Compilar

```bash
cargo build
```

---

## Ejecutar

```bash
cargo run
```

Para una compilación optimizada:

```bash
cargo run --release
```

---

## Tests

Cuando los módulos de física estén implementados:

```bash
cargo test --workspace
```

---

## Clippy

El proyecto buscará mantener una base de código limpia mediante:

```bash
cargo clippy --workspace -- -D warnings
```

---

# 🔬 Materiales

La versión inicial contempla materiales con diferentes funciones de trabajo:

| Material     | \(\Phi\) |
| ------------ | -------: |
| Potasio (K)  |  2.29 eV |
| Sodio (Na)   |  2.36 eV |
| Calcio (Ca)  |  2.90 eV |
| Zinc (Zn)    |  4.30 eV |
| Cobre (Cu)   |  4.70 eV |
| Platino (Pt) |  6.35 eV |

> Los valores definitivos y sus fuentes bibliográficas serán documentados antes de la release `v1.0.0`.

---

# 📐 Alcance de la primera versión

La versión `v1.0.0` tendrá como objetivo proporcionar un laboratorio virtual funcional para estudiar el efecto fotoeléctrico.

### Incluido

* [ ] Celda fotoeléctrica virtual.
* [ ] Fotones.
* [ ] Fotoelectrones.
* [ ] Campo eléctrico.
* [ ] Movimiento de partículas.
* [ ] Frecuencia configurable.
* [ ] Longitud de onda configurable.
* [ ] Intensidad configurable.
* [ ] Voltaje configurable.
* [ ] Selección de materiales.
* [ ] Corriente fotoeléctrica.
* [ ] Potencial de frenado.
* [ ] Gráficas experimentales.
* [ ] Regresión lineal.
* [ ] Determinación experimental de \(h\).
* [ ] Exportación CSV.
* [ ] Tests del núcleo físico.

> Estas funcionalidades representan el **objetivo de desarrollo**, no funcionalidades actualmente implementadas.

---

# 🚫 Fuera de alcance

La primera versión no pretende ser una simulación microscópica completa de un material real.

No se modelarán inicialmente:

* Estructura detallada de bandas.
* Densidad de estados.
* Interacciones electrón-electrón.
* Carga espacial.
* Dispersión microscópica detallada.
* Física cuántica completa del sólido.
* Geometrías tridimensionales complejas.
* Electrodinámica cuántica.

El objetivo es construir un **modelo educativo computacional**, no un simulador de materia condensada de alta fidelidad.

---

# 🗺️ Roadmap

## `v0.1.0` — Fundamentos

* [ ] Crear Cargo Workspace.
* [ ] Implementar constantes físicas.
* [ ] Implementar materiales.
* [ ] Implementar fotones.
* [ ] Implementar electrones.
* [ ] Implementar modelo de emisión.
* [ ] Crear pruebas unitarias.

## `v0.2.0` — Simulación

* [ ] Implementar celda fotoeléctrica.
* [ ] Implementar campo eléctrico.
* [ ] Implementar integración temporal.
* [ ] Implementar trayectorias.
* [ ] Implementar generación estocástica.
* [ ] Validar conservación y unidades físicas.

## `v0.3.0` — Renderizado

* [ ] Integrar Macroquad.
* [ ] Integrar OpenGL.
* [ ] Renderizar celda.
* [ ] Renderizar fotones.
* [ ] Renderizar electrones.
* [ ] Renderizar trayectorias.

## `v0.4.0` — Interfaz

* [ ] Integrar egui.
* [ ] Controles experimentales.
* [ ] Selector de materiales.
* [ ] Controles de simulación.
* [ ] Información física en tiempo real.

## `v0.5.0` — Laboratorio virtual

* [ ] Medición de corriente.
* [ ] Medición de potencial de frenado.
* [ ] Barridos automáticos.
* [ ] Registro de datos.
* [ ] Exportación CSV.

## `v0.6.0` — Análisis

* [ ] Gráfica \(I-V\).
* [ ] Gráfica \(K_{\max}-f\).
* [ ] Gráfica \(V_0-f\).
* [ ] Regresión lineal.
* [ ] Determinación de \(h\).
* [ ] Cálculo de error experimental.

## `v1.0.0` — Release académica

**Objetivo: 8 de octubre de 2026**

* [ ] Simulación completa.
* [ ] Interfaz terminada.
* [ ] Experimento de Millikan.
* [ ] Análisis de resultados.
* [ ] Documentación.
* [ ] Tests.
* [ ] Validación física.
* [ ] Reporte académico.
* [ ] Release estable.

---

# 📖 Referencia teórica

El proyecto se fundamenta principalmente en:

* Einstein y la explicación cuántica del efecto fotoeléctrico.
* Experimentos de Robert Millikan.
* Relación entre frecuencia y energía de los fotones.
* Función de trabajo de materiales.
* Potencial de frenado.
* Métodos de regresión lineal.
* Análisis experimental de incertidumbres.

Las referencias bibliográficas y fuentes de los parámetros físicos serán incorporadas en `docs/`.

---

# ⚠️ Estado del proyecto

Actualmente:

```text
Planificación
    │
    ▼
Arquitectura
    │
    ▼
Implementación      ← ACTUAL
    │
    ▼
Validación
    │
    ▼
Release v1.0.0
```

**Nada de la simulación se encuentra implementado todavía.**

El diseño descrito en este documento representa la arquitectura y funcionalidad **planificada** para el desarrollo del proyecto.

---

# 📄 Licencia

Este proyecto será distribuido bajo la **Licencia MIT**.

Desarrollado con fines académicos para la asignatura de **Física** en la:

**Universidad Autónoma del Estado de México (UAEMéx)**
**Centro Universitario Valle de Chalco**

---

<div align="center">

## ⚛️ Simulador del Efecto Fotoeléctrico

### Rust · Física Computacional · OpenGL · UAEMéx

**Omar Palomares Velasco · Leonardo Miguel Vega Carbajal**

`v1.0.0` · Octubre 2026

</div>
