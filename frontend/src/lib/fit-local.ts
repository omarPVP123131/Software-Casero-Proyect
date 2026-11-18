// Respaldo TypeScript del ajuste V₀–f para cuando el WASM aún no está
// cargado (primer render, tests de navegador). La ruta de producción es
// SIEMPRE `experiment_fit` (Rust): misma regresión `fit_planck_constant`.
import type { ExperimentPoint, FitReport } from "./types";

export const PLANCK_H = 6.62607015e-34;
export const ELEMENTARY_CHARGE = 1.602176634e-19;

export function fitReportLocal(points: ExperimentPoint[]): FitReport | null {
  const n = points.length;
  if (n < 2) return null;
  let sx = 0;
  let sy = 0;
  let sxy = 0;
  let sxx = 0;
  for (const p of points) {
    sx += p.frequencyHz;
    sy += p.stoppingMeasuredV;
    sxy += p.frequencyHz * p.stoppingMeasuredV;
    sxx += p.frequencyHz * p.frequencyHz;
  }
  const denominator = n * sxx - sx * sx;
  if (Math.abs(denominator) < 1e-30) return null;
  const slope = (n * sxy - sx * sy) / denominator;
  const intercept = (sy - slope * sx) / n;
  const meanY = sy / n;
  let ssTot = 0;
  let ssRes = 0;
  const residuals: number[] = [];
  for (const p of points) {
    const fitted = slope * p.frequencyHz + intercept;
    residuals.push(p.stoppingMeasuredV - fitted);
    ssTot += (p.stoppingMeasuredV - meanY) ** 2;
    ssRes += (p.stoppingMeasuredV - fitted) ** 2;
  }
  const rSquared = ssTot < 1e-30 ? (ssRes < 1e-30 ? 1 : 0) : Math.min(1, Math.max(0, 1 - ssRes / ssTot));
  const dof = Math.max(1, n - 2);
  const s2 = ssRes / dof;
  const slopeStd = Math.sqrt((n * s2) / denominator);
  const interceptStd = Math.sqrt((s2 * sxx) / denominator);
  const hExperimental = slope * ELEMENTARY_CHARGE;
  return {
    nPoints: n,
    slopeVs: slope,
    slopeStdVs: Number.isFinite(slopeStd) ? slopeStd : 0,
    interceptV: intercept,
    interceptStdV: Number.isFinite(interceptStd) ? interceptStd : 0,
    hExperimentalJs: hExperimental,
    hStdJs: Number.isFinite(slopeStd) ? slopeStd * ELEMENTARY_CHARGE : 0,
    hTheoreticalJs: PLANCK_H,
    errorPercentage: (Math.abs(hExperimental - PLANCK_H) / PLANCK_H) * 100,
    rSquared,
    residualsV: residuals,
  };
}
