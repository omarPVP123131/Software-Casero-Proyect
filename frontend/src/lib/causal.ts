// Explicación causal automática (y texto del modo guiado).
// Cada frase se genera desde la física calculada en Rust; nunca hardcodea
// resultados: usa `readout` para decidir el caso.
import type { Readout } from "./types";

export interface Causal {
  title: string;
  body: string;
  latex: string;
}

export function causalExplanation(
  r: Readout,
  controls: { wavelengthNm: number; appliedVoltageV: number },
  materialName: string,
  lang: "es" | "en",
): Causal {
  const lambda0 = r.thresholdWavelengthNm;
  const e = r.photonEnergyEv.toFixed(2);
  const phi = r.workFunctionEv.toFixed(2);
  if (!r.emits) {
    return lang === "en"
      ? {
          title: "No electrons are emitted",
          body: `λ = ${controls.wavelengthNm.toFixed(0)} nm > λ₀ = ${lambda0.toFixed(0)} nm for ${materialName}. The photon carries ${e} eV but ${phi} eV are needed: it is ${r.deficitEv.toFixed(2)} eV short. Increasing intensity adds photons, not energy per photon.`,
          latex: "h\\,f < \\Phi \\;\\Rightarrow\\; K_{\\max} = 0",
        }
      : {
          title: "No hay electrones porque λ > λ₀",
          body: `La luz de ${controls.wavelengthNm.toFixed(0)} nm lleva ${e} eV por fotón, pero arrancar un electrón de ${materialName} cuesta Φ = ${phi} eV. Faltan ${r.deficitEv.toFixed(2)} eV. Subir la intensidad añade fotones, no energía por fotón.`,
          latex: "h\\,f < \\Phi \\;\\Rightarrow\\; K_{\\max} = 0",
        };
  }
  if (r.blocked) {
    return lang === "en"
      ? {
          title: "Emitted, but blocked",
          body: `Electrons leave the cathode with Kmax = ${r.kMaxEv.toFixed(2)} eV, but the applied voltage (${controls.appliedVoltageV.toFixed(2)} V) is more negative than −V₀ = ${(-r.stoppingPotentialV).toFixed(2)} V. They are turned back before reaching the anode: current is zero while Kmax stays the same.`,
          latex: "V < -V_0 \\;\\Rightarrow\\; I = 0,\\; K_{\\max}\\ \\text{igual}",
        }
      : {
          title: "Se emiten, pero están bloqueados",
          body: `Los electrones salen con Kmax = ${r.kMaxEv.toFixed(2)} eV, pero el voltaje aplicado (${controls.appliedVoltageV.toFixed(2)} V) es más negativo que −V₀ = ${(-r.stoppingPotentialV).toFixed(2)} V. Se frenan antes de llegar al ánodo: la corriente es cero y Kmax no cambia.`,
          latex: "V < -V_0 \\;\\Rightarrow\\; I = 0,\\; K_{\\max}\\ \\text{igual}",
        };
  }
  return lang === "en"
      ? {
          title: "Electrons are emitted",
          body: `Each photon of ${controls.wavelengthNm.toFixed(0)} nm carries ${e} eV. After paying Φ = ${phi} eV, the electron leaves with up to Kmax = ${r.kMaxEv.toFixed(2)} eV (v ≈ ${(r.electronMaxSpeedMS / 1e6).toFixed(2)}×10⁶ m/s). Intensity changes how many electrons, not their energy.`,
          latex: "K_{\\max} = h\\,f - \\Phi",
        }
      : {
          title: "Los electrones se emiten",
          body: `Cada fotón de ${controls.wavelengthNm.toFixed(0)} nm lleva ${e} eV. Tras pagar Φ = ${phi} eV, el electrón sale con hasta Kmax = ${r.kMaxEv.toFixed(2)} eV (v ≈ ${(r.electronMaxSpeedMS / 1e6).toFixed(2)}×10⁶ m/s). La intensidad cambia cuántos electrones salen, no su energía.`,
          latex: "K_{\\max} = h\\,f - \\Phi",
        };
}

export interface TutorialStep {
  title: string;
  body: string;
  control: "light" | "intensity" | "voltage" | "material" | "experiment";
  prediction?: string;
  action?: () => void;
}

export function tutorialSteps(lang: "es" | "en"): TutorialStep[] {
  if (lang === "en") {
    return [
      {
        title: "1 · Threshold: change wavelength",
        body: "Start below the sodium threshold (e.g. 600 nm). Watch the energy ladder: the photon level sits under Φ. Lower λ until it crosses the dashed line — emission begins.",
        control: "light",
        prediction: "What happens if you make λ shorter?",
      },
      {
        title: "2 · Intensity vs energy",
        body: "Now above threshold, double the intensity. More electrons per second, same Kmax. The energy per photon is set by frequency alone.",
        control: "intensity",
        prediction: "What happens if you double the intensity?",
      },
      {
        title: "3 · Stopping potential",
        body: "Push the voltage below −V₀. The electrons slow down and turn back; current drops to zero while Kmax is untouched.",
        control: "voltage",
        prediction: "What happens below −V₀?",
      },
      {
        title: "4 · Measure h",
        body: "Open the V₀–f experiment, run the auto sweep and fit. The slope is h/e, the same for every material.",
        control: "experiment",
        prediction: "If you repeat the fit with another cathode, does the slope change?",
      },
    ];
  }
  return [
    {
      title: "1 · El umbral: mueve la longitud de onda",
      body: "Empieza por debajo del umbral del sodio (p. ej. 600 nm). Mira el diagrama de energías: el nivel del fotón está por debajo de Φ. Baja λ hasta que cruce la línea discontinua: empieza la emisión.",
      control: "light",
      prediction: "¿Qué pasa si acortas la longitud de onda?",
    },
    {
      title: "2 · Intensidad vs energía",
      body: "Ahora que hay emisión, duplica la intensidad. Verás más electrones por segundo, pero Kmax no cambia: la energía de cada fotón depende solo de la frecuencia.",
      control: "intensity",
      prediction: "¿Qué pasará si duplico la intensidad?",
    },
    {
      title: "3 · Potencial de frenado",
      body: "Baja el voltaje por debajo de −V₀. Los electrones se frenan y regresan; la corriente cae a cero pero Kmax sigue igual.",
      control: "voltage",
      prediction: "¿Qué pasa por debajo de −V₀?",
    },
    {
      title: "4 · Medir h",
      body: "Abre el experimento V₀–f, corre el barrido automático y ajusta. La pendiente es h/e, igual para todos los materiales.",
      control: "experiment",
      prediction: "Si repites el ajuste con otro cátodo, ¿cambia la pendiente?",
    },
  ];
}
