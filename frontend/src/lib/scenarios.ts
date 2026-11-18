// Escenarios de demostración de un clic. Todos parten de la física del
// motor; aquí solo se ajustan los controles (λ, I, V, material).
import type { ExperimentControls, MaterialChoice } from "./types";

export type ScenarioId = "threshold" | "intensity" | "stopping" | "measure_h";

export interface Scenario {
  id: ScenarioId;
  labelEs: string;
  labelEn: string;
  detailEs: string;
  apply: (controls: ExperimentControls) => ExperimentControls;
}

export const SCENARIOS: Scenario[] = [
  {
    id: "threshold",
    labelEs: "Umbral del sodio",
    labelEn: "Sodium threshold",
    detailEs: "Sodio a 525 nm (λ₀ ≈ 525 nm): mueve ±20 nm y cruza el umbral.",
    apply: (c) => ({
      ...c,
      material: "sodium" as MaterialChoice,
      wavelengthNm: 525,
      intensityPercent: 60,
      appliedVoltageV: 0,
    }),
  },
  {
    id: "intensity",
    labelEs: "Intensidad vs energía",
    labelEn: "Intensity vs energy",
    detailEs: "Sodio a 400 nm: sube la intensidad; Kmax no debe moverse.",
    apply: (c) => ({
      ...c,
      material: "sodium" as MaterialChoice,
      wavelengthNm: 400,
      intensityPercent: 20,
      appliedVoltageV: 0,
    }),
  },
  {
    id: "stopping",
    labelEs: "Potencial de frenado",
    labelEn: "Stopping potential",
    detailEs: "Sodio a 300 nm con V = −2 V: emisión sí, corriente cero.",
    apply: (c) => ({
      ...c,
      material: "sodium" as MaterialChoice,
      wavelengthNm: 300,
      intensityPercent: 70,
      appliedVoltageV: -2,
    }),
  },
  {
    id: "measure_h",
    labelEs: "Medir h",
    labelEn: "Measure h",
    detailEs: "Cobre a 220 nm: prepara la tabla para medir h con V₀–f.",
    apply: (c) => ({
      ...c,
      material: "copper" as MaterialChoice,
      wavelengthNm: 220,
      intensityPercent: 60,
      appliedVoltageV: 0,
    }),
  },
];
