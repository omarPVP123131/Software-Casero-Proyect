// Conversión λ (nm) → color RGB perceptualmente razonable.
// Basada en la aproximación clásica de Dan Bruton (spectra de 380–780 nm).
// Fuera de ese rango (UV/IR) devolvemos violeta/rojo oscuro + `visible:false`
// para que la UI añada patrón rayado y la etiqueta "no visible".

export interface SpectrumColor {
  /** Color CSS (hex o rgb(...)). */
  css: string;
  /** Componentes 0..255 para canvas. */
  rgb: [number, number, number];
  visible: boolean;
  /** Etiqueta i18n: "UV", "IR" o "". */
  band: "uv" | "ir" | "visible";
}

function clamp01(v: number): number {
  return Math.min(1, Math.max(0, v));
}

function gamma(t: number): number {
  return 0.42 <= t && t <= 0.58 ? 1 : Math.pow(Math.abs(t - 0.5) * 2, 1.6);
}

export function wavelengthToColor(nm: number): SpectrumColor {
  if (!Number.isFinite(nm) || nm < 380) {
    return { css: "#8B5CF6", rgb: [139, 92, 246], visible: false, band: "uv" };
  }
  if (nm > 780) {
    return { css: "#7F1D1D", rgb: [127, 29, 29], visible: false, band: "ir" };
  }
  let r = 0;
  let g = 0;
  let b = 0;
  if (nm < 440) {
    r = (440 - nm) / 60;
    b = 1;
  } else if (nm < 490) {
    g = (nm - 440) / 50;
    b = 1;
  } else if (nm < 510) {
    g = 1;
    b = (510 - nm) / 20;
  } else if (nm < 580) {
    r = (nm - 510) / 70;
    g = 1;
  } else if (nm < 645) {
    r = 1;
    g = (645 - nm) / 65;
  } else {
    r = 1;
  }
  const factor = gamma((nm - 380) / 400);
  const rgb: [number, number, number] = [
    Math.round(clamp01(r * factor) * 255),
    Math.round(clamp01(g * factor) * 255),
    Math.round(clamp01(b * factor) * 255),
  ];
  return {
    css: `rgb(${rgb[0]} ${rgb[1]} ${rgb[2]})`,
    rgb,
    visible: true,
    band: "visible",
  };
}

/** Versión aclarada para acentos de UI (mezcla con blanco). */
export function accentColor(nm: number, mix = 0.35): string {
  const { rgb } = wavelengthToColor(nm);
  const m = (c: number) => Math.round(c + (255 - c) * mix);
  return `rgb(${m(rgb[0])} ${m(rgb[1])} ${m(rgb[2])})`;
}

/** Gradiente del espectro visible para fondos de slider (CSS). */
export function spectrumGradient(stops = 24): string {
  const parts: string[] = [];
  for (let i = 0; i < stops; i++) {
    const nm = 380 + (i / (stops - 1)) * 400;
    const { css } = wavelengthToColor(nm);
    parts.push(`${css} ${(i / (stops - 1)) * 100}%`);
  }
  return `linear-gradient(90deg, #4C1D95 0%, ${parts.join(", ")}, #7F1D1D 100%)`;
}

/** Color del icono de estado de emisión (acompañado de forma + texto). */
export function emissionStateColor(state: "emit" | "none" | "blocked"): string {
  switch (state) {
    case "emit":
      return "#34D399";
    case "blocked":
      return "#F59E0B";
    default:
      return "#94A3B8";
  }
}
