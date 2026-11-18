// @vitest-environment jsdom
// Exportación de informes: en navegador debe abrir una ventana y escribir el
// HTML; si el popup está bloqueado, descarga. En Tauri usa `open_report_html`.
import { afterEach, describe, expect, it, vi } from "vitest";
import { openReport } from "../io";

const HTML = "<!doctype html><html><body>informe</body></html>";

afterEach(() => {
  vi.unstubAllGlobals();
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
});

describe("openReport", () => {
  it("en navegador escribe el informe en la ventana emergente", async () => {
    const written: string[] = [];
    const fakeWin = {
      document: {
        write: (s: string) => written.push(s),
        close: () => undefined,
      },
    };
    vi.stubGlobal("open", vi.fn(() => fakeWin));
    await expect(openReport(HTML, "informe.html")).resolves.toBe("opened");
    expect(written.join("")).toContain("informe");
  });

  it("si el popup está bloqueado descarga el HTML", async () => {
    vi.stubGlobal("open", vi.fn(() => null));
    // jsdom no implementa createObjectURL ni click de descarga: se aíslan.
    vi.stubGlobal("URL", { ...URL, createObjectURL: () => "blob:x", revokeObjectURL: () => undefined });
    await expect(openReport(HTML, "informe.html")).resolves.toBe("downloaded");
  });
});
