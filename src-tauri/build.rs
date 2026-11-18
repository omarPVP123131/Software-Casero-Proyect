//! Build script del backend Tauri.
//!
//! `tauri-build` genera `OUT_DIR/tauri-build-context` (que consume
//! `tauri::generate_context!`) y compila los recursos y el manifiesto de
//! capacidades. Sin este script el crate no compila.

fn main() {
    tauri_build::build()
}
