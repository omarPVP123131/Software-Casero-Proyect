import { defineConfig } from "@playwright/test";

// UI tests del canvas y de los flujos principales. Las capturas de
// referencia viven en e2e/__screenshots__ y detectan regresiones visuales
// sin ejecutar la GUI nativa (Tauri comparte el mismo WebView/frontend).
export default defineConfig({
  testDir: "./e2e",
  timeout: 45_000,
  expect: { timeout: 10_000 },
  fullyParallel: false,
  retries: 0,
  reporter: [["list"]],
  use: {
    baseURL: "http://127.0.0.1:1420",
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 1,
    colorScheme: "dark",
    reducedMotion: "reduce",
    // El runner headless-shell tiene problemas de input en contenedores;
    // el Chromium "nuevo headless" sí despacha clics con normalidad.
    launchOptions: { channel: "chromium" },
  },
  webServer: {
    command: "npm run dev -- --port 1420",
    url: "http://127.0.0.1:1420",
    reuseExistingServer: true,
    timeout: 60_000,
  },
});
