// Compila la física a WASM y deja los artefactos en frontend/src/lib/wasm/.
// Funciona en Windows, macOS y Linux (solo requiere wasm-pack en el PATH).
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import fs from "node:fs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outDir = path.join(root, "frontend", "src", "lib", "wasm");

const result = spawnSync(
  "wasm-pack",
  [
    "build",
    path.join(root, "crates", "wasm"),
    "--target",
    "web",
    "--release",
    "--out-dir",
    outDir,
    "--out-name",
    "photolab_wasm",
  ],
  { stdio: "inherit", cwd: root },
);

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

const gitignore = path.join(outDir, ".gitignore");
if (fs.existsSync(gitignore)) fs.unlinkSync(gitignore);

console.log("WASM listo: frontend/src/lib/wasm/photolab_wasm_bg.wasm");
