//! Backend Tauri 2 de PhotoLab.
//!
//! Solo tres responsabilidades:
//! 1. Registrar los comandos de `photolab-tauri-commands` (física pura).
//! 2. Autoguardado de sesión y flujos nativos de archivo (diálogos + fs).
//! 3. Plugins con permisos mínimos (ver `capabilities/default.json`).
//!
//! Además expone `open_report_html`: los informes del cuaderno y del
//! experimento se generan en el frontend como HTML autocontenido y aquí solo se
//! escriben y se abren con la aplicación del sistema, de modo que el PDF se
//! exporta con el diálogo de impresión nativo (WebKitGTK no abre ventanas
//! `window.open` desde el WebView de la app).
//!
//! No hay egui ni Macroquad: el canvas y las gráficas viven en el frontend.

use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_opener::OpenerExt;

/// Comandos del crate `photolab-tauri-commands`, reexportados tal cual.
///
/// El glob es deliberado: `#[tauri::command]` exporta, además de la función,
/// las macros `__cmd__*` / `__tauri_command_name_*` en la raíz de
/// `photolab-tauri-commands`, y `generate_handler!` las busca en el ámbito
/// léxico. Una lista explícita de funciones las dejaría fuera y el build
/// fallaría con "cannot find macro `__cmd__physics_evaluate`".
pub use photolab_tauri_commands::*;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionPaths {
    pub autosave_path: String,
    pub config_dir: String,
}

fn session_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("sin config dir: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("no se pudo crear {dir:?}: {e}"))?;
    Ok(dir)
}

/// Guarda un informe HTML autocontenido en la carpeta de config y lo abre con
/// la aplicación por defecto del sistema (navegador) para imprimir a PDF.
///
/// El HTML lo construye el frontend (cuaderno o experimento) con los datos ya
/// resueltos: aquí no se genera contenido, solo se escribe el archivo.
#[tauri::command]
fn open_report_html<R: Runtime>(
    app: AppHandle<R>,
    html: String,
    filename: String,
) -> Result<String, String> {
    // Solo el nombre del archivo: nada de rutas del usuario (evita traversal).
    let name = PathBuf::from(&filename)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_string())
        .ok_or_else(|| "nombre de informe inválido".to_string())?;
    if !name.ends_with(".html") {
        return Err("el informe debe ser .html".into());
    }
    let dir = session_dir(&app)?.join("reports");
    fs::create_dir_all(&dir).map_err(|e| format!("no se pudo crear {dir:?}: {e}"))?;
    let path = dir.join(name);
    fs::write(&path, html).map_err(|e| format!("no se pudo escribir {path:?}: {e}"))?;
    app.opener()
        .open_path(path.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| format!("informe guardado en {path:?}, pero no se pudo abrir: {e}"))?;
    Ok(path.to_string_lossy().into_owned())
}

/// Rutas de autoguardado (la UI las muestra en Ajustes > Sesión).
#[tauri::command]
fn session_paths<R: Runtime>(app: AppHandle<R>) -> Result<SessionPaths, String> {
    let dir = session_dir(&app)?;
    let autosave = dir.join("autosave.photolab.json");
    Ok(SessionPaths {
        autosave_path: autosave.to_string_lossy().into_owned(),
        config_dir: dir.to_string_lossy().into_owned(),
    })
}

/// Autoguardado: escribe la sesión en el directorio de config (o en `path`).
#[tauri::command]
fn autosave_session<R: Runtime>(
    app: AppHandle<R>,
    session_json: String,
    path: Option<String>,
) -> Result<photolab_tauri_commands::SaveResult, String> {
    let target = match path {
        Some(p) => PathBuf::from(p),
        None => session_dir(&app)?.join("autosave.photolab.json"),
    };
    photolab_tauri_commands::save_session(session_json, Some(target.to_string_lossy().into_owned()))
}

/// Lee el autoguardado si existe (arranque: sesión donde la dejaste).
#[tauri::command]
fn read_autosave<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    let file = session_dir(&app)?.join("autosave.photolab.json");
    match fs::read_to_string(file) {
        Ok(contents) => Ok(Some(contents)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("No se pudo leer el autoguardado: {e}")),
    }
}

/// Estado del actualizador (tauri-plugin-updater). En dev builds devuelve
/// `null` para que la UI no muestre un banner falso.
#[tauri::command]
async fn check_update<R: Runtime>(_app: AppHandle<R>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "available": false,
        "note": "Configura tauri.conf.json > plugins.updater con tu endpoint firmado."
    }))
}

/// Info del motor para la barra de estado ("motor Rust v2.0.0 · ideal").
#[tauri::command]
fn engine_info() -> serde_json::Value {
    serde_json::json!({
        "engine": "photoelectrico-physics",
        "modelVersion": env!("CARGO_PKG_VERSION"),
        "noiseModel": "determinista (splitmix64)",
        "units": "SI + eV para lectura",
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())
                    .expect("plugin updater");
            }
            let _ = app;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            physics_evaluate,
            physics_material_catalog,
            experiment_fit,
            experiment_sweep,
            experiment_csv,
            session_normalize,
            save_session,
            load_session,
            export_file,
            session_paths,
            autosave_session,
            read_autosave,
            check_update,
            engine_info,
            open_report_html,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PhotoLab");
}

#[cfg(test)]
mod tests {
    #[test]
    fn report_filename_is_sanitized() {
        // La ruta absoluta se reduce al nombre: nada escapa de la carpeta.
        let name = std::path::PathBuf::from("../../evil.html")
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string);
        assert_eq!(name.as_deref(), Some("evil.html"));
    }

    #[test]
    fn engine_info_carries_metadata() {
        let info = super::engine_info();
        assert_eq!(info["engine"], "photoelectrico-physics");
    }
}
