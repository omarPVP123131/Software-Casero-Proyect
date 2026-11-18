// Puente de física del frontend.
//
// Estrategia (spec §1): la física se compila a WASM y corre aquí. En Tauri
// el mismo WASM se usa por defecto; `invoke` queda como respaldo verificable
// (y para comandos de disco). Nunca se llama al backend por frame:
// `evaluate()` se invoca una vez por cambio de control (debounce en el store).
import type {
  EvaluateResponse,
  ExperimentControls,
  ExperimentPoint,
  ExperimentState,
  FitReport,
  MaterialInfo,
  SweepPoint,
  SweepRequest,
  UiPreferences,
} from "./types";

export type PhysicsBackend = "wasm" | "tauri";

let backend: PhysicsBackend = "wasm";
export const physicsState: { backend: PhysicsBackend } = { backend };

interface WasmModule {
  default: (input?: unknown) => Promise<unknown>;
  physics_evaluate: (c: unknown, p: unknown) => unknown;
  physics_material_catalog: () => unknown;
  experiment_fit: (points: unknown) => unknown;
  experiment_sweep: (req: unknown) => unknown;
  experiment_csv: (state: unknown) => string;
}

/** ¿Corremos dentro de la WebView de Tauri? (`__TAURI_INTERNALS__` inyectado). */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function invokeCmd<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

let wasm: WasmModule | null = null;
let boot: Promise<void> | null = null;

/** Carga el WASM una sola vez. Si falla y estamos en Tauri, usa invoke. */
export async function initPhysics(): Promise<PhysicsBackend> {
  if (wasm || backend === "tauri") return backend;
  boot ??= (async () => {
    try {
      const mod = (await import("./wasm/photolab_wasm.js")) as unknown as WasmModule;
      try {
        await mod.default();
      } catch (fetchError) {
        // Node/Vitest no puede `fetch(file://…)`: cargamos los bytes a mano.
        // En navegador y Tauri siempre se usa fetch.
        const node = (globalThis as { process?: { versions?: { node?: string } } }).process;
        if (node?.versions?.node) {
          const { readFile } = await import("node:fs/promises");
          const { fileURLToPath } = await import("node:url");
          const url = new URL("./wasm/photolab_wasm_bg.wasm", import.meta.url);
          await mod.default(await readFile(fileURLToPath(url)));
        } else {
          throw fetchError;
        }
      }
      wasm = mod;
      backend = "wasm" as PhysicsBackend;
    } catch (error) {
      boot = null;
      if (!isTauri()) throw error;
      backend = "tauri" as PhysicsBackend;
    }
    physicsState.backend = backend;
  })();
  await boot;
  return backend;
}

function fromWasm<T>(value: unknown): T {
  // serde_wasm_bindgen devuelve objetos JS planos salvo números grandes
  // (u64 → BigInt); normalizamos por si acaso.
  return JSON.parse(
    JSON.stringify(value, (_k, v) => (typeof v === "bigint" ? Number(v) : v)),
  ) as T;
}

export async function evaluate(
  controls: ExperimentControls,
  preferences: UiPreferences,
): Promise<EvaluateResponse> {
  await initPhysics();
  if (wasm) return fromWasm<EvaluateResponse>(wasm.physics_evaluate(controls, preferences));
  return invokeCmd<EvaluateResponse>("physics_evaluate", {
    controlsJson: JSON.stringify(controls),
    prefsJson: JSON.stringify(preferences),
  });
}

export async function materialCatalog(): Promise<MaterialInfo[]> {
  await initPhysics();
  if (wasm) return fromWasm<MaterialInfo[]>(wasm.physics_material_catalog());
  return invokeCmd<MaterialInfo[]>("physics_material_catalog");
}

export async function fitExperiment(points: ExperimentPoint[]): Promise<FitReport | null> {
  await initPhysics();
  if (wasm) return fromWasm<FitReport | null>(wasm.experiment_fit(points));
  return invokeCmd<FitReport | null>("experiment_fit", {
    pointsJson: JSON.stringify(points),
  });
}

export async function sweepExperiment(request: SweepRequest): Promise<SweepPoint[]> {
  await initPhysics();
  if (wasm) return fromWasm<SweepPoint[]>(wasm.experiment_sweep(request));
  return invokeCmd<SweepPoint[]>("experiment_sweep", {
    requestJson: JSON.stringify(request),
  });
}

export async function experimentCsv(state: ExperimentState): Promise<string> {
  await initPhysics();
  if (wasm) return wasm.experiment_csv(state);
  return invokeCmd<string>("experiment_csv", {
    experimentJson: JSON.stringify(state),
  });
}
