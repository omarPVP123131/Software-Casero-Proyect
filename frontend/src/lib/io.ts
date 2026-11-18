// Guardado / carga de sesión y exportación. En Tauri usa diálogos nativos y
// comandos de disco; en navegador cae a descarga/subida de archivos.
import type { Session } from "./types";
import { isTauri } from "./physics";
import { downloadBlob } from "./plots";

export async function saveSession(session: Session): Promise<string | null> {
  const json = JSON.stringify(session, null, 2);
  if (isTauri()) {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const path = await save({
      title: "Guardar sesión de PhotoLab",
      defaultPath: "sesion.photolab.json",
      filters: [{ name: "PhotoLab session", extensions: ["json"] }],
    });
    if (!path) return null;
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<{ path: string }>("save_session", {
      sessionJson: json,
      path,
    });
    return result.path;
  }
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "sesion.photolab.json";
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 2000);
  return "sesion.photolab.json (descargada)";
}

export async function openSession(): Promise<Session | null> {
  if (isTauri()) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const path = await open({
      title: "Abrir sesión de PhotoLab",
      multiple: false,
      filters: [{ name: "PhotoLab session", extensions: ["json"] }],
    });
    if (!path || Array.isArray(path)) return null;
    const { invoke } = await import("@tauri-apps/api/core");
    const normalized = await invoke<string>("session_normalize", {
      sessionJson: await (await import("@tauri-apps/plugin-fs")).readTextFile(path),
    } as never);
    return JSON.parse(typeof normalized === "string" ? normalized : JSON.stringify(normalized)) as Session;
  }
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json,.photolab.json,application/json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return resolve(null);
      try {
        resolve(JSON.parse(await file.text()) as Session);
      } catch {
        resolve(null);
      }
    };
    input.click();
  });
}

/** Autoguardado Tauri en el directorio de config, si está disponible. */
export async function autosave(session: Session): Promise<void> {
  if (!isTauri()) {
    try {
      localStorage.setItem("photolab.autosave", JSON.stringify(session));
    } catch {
      /* sin almacenamiento */
    }
    return;
  }
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("autosave_session", { sessionJson: JSON.stringify(session), path: null });
}

export async function readAutosave(): Promise<Session | null> {
  if (!isTauri()) {
    try {
      const raw = localStorage.getItem("photolab.autosave");
      return raw ? (JSON.parse(raw) as Session) : null;
    } catch {
      return null;
    }
  }
  const { invoke } = await import("@tauri-apps/api/core");
  const raw = await invoke<string | null>("read_autosave");
  return raw ? (JSON.parse(raw) as Session) : null;
}

/**
 * Abre un informe HTML autocontenido para imprimirlo como PDF.
 *
 * - **Tauri**: lo escribe en `<config>/reports/` y lo abre con la aplicación
 *   del sistema (`open_report_html`). La ventana del WebView no puede usar
 *   `window.open`, así que este es el camino fiable; el diálogo de impresión
 *   del navegador es el que genera el PDF.
 * - **Navegador**: ventana emergente con `window.print()`; si el popup está
 *   bloqueado se descarga el HTML.
 *
 * Devuelve `"opened" | "downloaded" | "blocked"` para que la UI informe.
 */
export async function openReport(
  html: string,
  filename = "informe-photolab.html",
): Promise<"opened" | "downloaded" | "blocked"> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke<string>("open_report_html", { html, filename });
    return "opened";
  }
  const win = window.open("", "_blank", "width=900,height=700");
  if (!win) {
    downloadBlob(new Blob([html], { type: "text/html;charset=utf-8" }), filename);
    return "downloaded";
  }
  win.document.write(html);
  win.document.close();
  return "opened";
}
