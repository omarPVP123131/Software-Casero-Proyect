// Regenera los tipos TypeScript desde Rust (ts-rs).
// Funciona en Windows, macOS y Linux.
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const outDir = path.join(root, "frontend", "src", "lib", "generated");

const env = { ...process.env, PHOTOLAB_TS_OUT: outDir };
const result = spawnSync(
  "cargo",
  ["test", "-p", "photolab-model", "--lib", "--", "export_bindings", "--quiet"],
  { stdio: "inherit", cwd: root, env },
);

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

console.log("Tipos TS regenerados en frontend/src/lib/generated/");
