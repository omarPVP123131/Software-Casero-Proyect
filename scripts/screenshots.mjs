// Capturas de documentación (README) del build de producción.
//
//   cd frontend && npm run build
//   cd .. && npx vite preview --port 1420 --host 127.0.0.1 &   # o `tauri dev`
//   node scripts/screenshots.mjs
//
// Requiere `playwright` (ya presente en frontend/node_modules); se ejecuta con
// `cd frontend && node ../scripts/screenshots.mjs` o ajustando NODE_PATH.
import { chromium } from "playwright";
import { mkdirSync } from "node:fs";

const BASE = process.env.PHOTOLAB_URL ?? "http://127.0.0.1:1420/";
const OUT = new URL("../assets/", import.meta.url).pathname;
mkdirSync(OUT, { recursive: true });

const browser = await chromium.launch({ channel: "chromium" });
const page = await browser.newPage({
  viewport: { width: 1600, height: 1000 },
  deviceScaleFactor: 2,
  colorScheme: "dark",
});
await page.goto(BASE);
await page.getByTestId("emission-state").waitFor();
await page.waitForTimeout(1200);
await page.screenshot({ path: `${OUT}ui-dark-1600.png` });
await page.locator(".scene-wrap").screenshot({ path: `${OUT}scene-hero.png` });

// tema claro + IR (λ no visible)
await page.getByRole("button", { name: "light" }).click();
await page.getByRole("button", { name: /^IR$/ }).click();
await page.waitForTimeout(900);
await page.screenshot({ path: `${OUT}ui-light-ir-1600.png` });

// alto contraste + pestaña Experimento con el escenario "Medir h"
await page.getByRole("button", { name: "contrast" }).click();
await page.getByRole("button", { name: /^Verde$/ }).click();
await page.keyboard.press("Control+k");
await page.getByRole("option", { name: /Medir h/ }).click();
await page.waitForTimeout(1600);
await page.screenshot({ path: `${OUT}ui-contrast-experiment.png` });

await browser.close();
console.log(`capturas → ${OUT}`);
