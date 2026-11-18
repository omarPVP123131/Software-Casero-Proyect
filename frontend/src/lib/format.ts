// Formatos con unidades para lecturas (cifras tabulares en la UI).
// Portado de `crates/app/src/format.rs` para mantener el estilo original.

const SUP: Record<string, string> = {
  "0": "⁰", "1": "¹", "2": "²", "3": "³", "4": "⁴",
  "5": "⁵", "6": "⁶", "7": "⁷", "8": "⁸", "9": "⁹",
  "-": "⁻", "+": "⁺",
};

export function superscript(text: string): string {
  return text.split("").map((c) => SUP[c] ?? c).join("");
}

export function scientific(value: number, decimals = 2): string {
  if (!Number.isFinite(value)) return "—";
  if (value === 0) return "0";
  const exponent = Math.floor(Math.log10(Math.abs(value)));
  if (exponent === 0) return value.toFixed(decimals);
  const mantissa = value / 10 ** exponent;
  return `${mantissa.toFixed(decimals)}×10${superscript(String(exponent))}`;
}

export function hz(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value) || value < 0) return "—";
  return `${scientific(value, 2)} Hz`;
}

export function ev(value: number | null | undefined, decimals = 2): string {
  if (value == null || !Number.isFinite(value) || value < 0) return "—";
  return `${value.toFixed(decimals)} eV`;
}

export function volts(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value)) return "—";
  return `${value.toFixed(2)} V`;
}

export function nm(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value) || value < 0) return "—";
  return `${value.toFixed(1)} nm`;
}

export function flux(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value) || value < 0) return "—";
  if (value === 0) return "0 m⁻²·s⁻¹";
  return `${scientific(value, 2)} m⁻²·s⁻¹`;
}

/**
 * Flujo en dos líneas para las tarjetas estrechas: la unidad se separa del
 * número porque `1.11×10²⁰ m⁻²·s⁻¹` no cabe en 126 px y se salía de la card.
 * Devuelve [número, unidad] (la unidad puede ir vacía).
 */
export function fluxParts(value: number | null | undefined): [string, string] {
  const text = flux(value);
  if (text === "—" || text === "0 m⁻²·s⁻¹") return [text, ""];
  const idx = text.lastIndexOf(" ");
  return idx < 0 ? [text, ""] : [text.slice(0, idx), text.slice(idx + 1)];
}

export function speed(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value) || value < 0) return "—";
  if (value === 0) return "0 m/s";
  return value >= 1e5 ? `${scientific(value, 2)} m/s` : `${value.toFixed(0)} m/s`;
}

export function current(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value) || value < 0) return "—";
  if (value === 0) return "0 µA";
  if (value >= 1) return `${value.toFixed(3)} A`;
  if (value >= 1e-3) return `${(value * 1e3).toFixed(3)} mA`;
  if (value >= 1e-6) return `${(value * 1e6).toFixed(2)} µA`;
  return `${(value * 1e9).toFixed(1)} nA`;
}

export function planck(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value) || value < 0) return "—";
  return `${scientific(value, 2)} J·s`;
}

export function slope(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value)) return "—";
  return `${scientific(value, 2)} V·s`;
}

export function percent(value: number | null | undefined, decimals = 1): string {
  if (value == null || !Number.isFinite(value) || value < 0) return "—";
  return `${value.toFixed(decimals)} %`;
}

export function signed(value: number | null | undefined, decimals = 2, unit = " V"): string {
  if (value == null || !Number.isFinite(value)) return "—";
  if (value === 0) return `${(0).toFixed(decimals)}${unit}`;
  const sign = value < 0 ? "−" : "";
  return `${sign}${Math.abs(value).toFixed(decimals)}${unit}`;
}

/** nm → THz (para la entrada numérica sincronizada). */
export function nmToThz(nmValue: number): number {
  return 299792.458 / nmValue;
}

export function thzToNm(thz: number): number {
  return 299792.458 / thz;
}

/** nm → eV del fotón. */
export function nmToEv(nmValue: number): number {
  return 1239.841984 / nmValue;
}

export function evToNm(evValue: number): number {
  return 1239.841984 / evValue;
}
