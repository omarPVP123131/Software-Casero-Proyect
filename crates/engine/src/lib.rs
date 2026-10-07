//! Crate de renderizado del simulador.
//!
//! Este crate es intencionalmente un esqueleto: define la separación prevista
//! para escena y renderer, pero todavía no contiene renderizado ejecutable.
//! La interfaz gráfica actual está en `fotoelectrico-app`.

#![forbid(unsafe_code)]

pub mod renderer;
pub mod scene;
