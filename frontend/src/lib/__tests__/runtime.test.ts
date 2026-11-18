// Integración real: WASM + store, sin navegador. Verifica que el contrato
// Rust→TS se cumple y que la física visible es la del motor.
import { describe, expect, it } from "vitest";
import { lab } from "../store.svelte";
import { physicsState } from "../physics";

const tick = (ms = 30) => new Promise((r) => setTimeout(r, ms));

describe("motor WASM conectado al store", () => {
  it("evalúa sodio 400 nm con la física real", async () => {
    await lab.init();
    await tick();
    expect(physicsState.backend).toBe("wasm");
    const r = lab.readout!;
    expect(r.emits).toBe(true);
    expect(r.photonEnergyEv).toBeCloseTo(3.1, 1);
    expect(r.kMaxEv).toBeCloseTo(0.74, 1);
    expect(lab.response!.curve).toHaveLength(61);
    expect(lab.catalog).toHaveLength(6);
  });

  it("rojo 700 nm no emite y reporta el déficit", async () => {
    lab.update((c) => { c.wavelengthNm = 700; });
    await tick();
    expect(lab.emissionState).toBe("none");
    expect(lab.readout!.kMaxEv).toBe(0);
    expect(lab.readout!.deficitEv).toBeCloseTo(0.59, 1);
  });

  it("frenado bloquea sin tocar Kmax", async () => {
    lab.update((c) => { c.wavelengthNm = 400; c.appliedVoltageV = -2; });
    await tick();
    expect(lab.emissionState).toBe("blocked");
    expect(lab.readout!.kMaxEv).toBeCloseTo(0.74, 1);
    expect(lab.readout!.photocurrentA).toBe(0);
  });

  it("barrido + ajuste recupera h", async () => {
    lab.update((c) => { c.appliedVoltageV = 0; });
    await lab.autoSweep(6);
    expect(lab.experiment.points).toHaveLength(6);
    expect(lab.fit).not.toBeNull();
    expect(lab.fit!.errorPercentage).toBeLessThan(1);
    expect(lab.fit!.rSquared).toBeGreaterThan(0.999);
  });

  it("coalesce múltiples sets en un solo evaluate", async () => {
    const before = lab.response;
    lab.update((c) => { c.intensityPercent = 61; });
    lab.update((c) => { c.intensityPercent = 62; });
    lab.update((c) => { c.intensityPercent = 63; });
    await tick();
    expect(lab.controls.intensityPercent).toBe(63);
    expect(lab.response).not.toBe(before);
  });
});
