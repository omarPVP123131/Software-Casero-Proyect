// Reexporta los tipos generados desde Rust con ts-rs (fuente única de verdad).
export type { Readout } from "./generated/Readout";
export type { CurvePoint } from "./generated/CurvePoint";
export type { EvaluateResponse } from "./generated/EvaluateResponse";
export type { Session } from "./generated/Session";
export type { ExperimentControls } from "./generated/ExperimentControls";
export type { ExperimentPoint } from "./generated/ExperimentPoint";
export type { ExperimentState } from "./generated/ExperimentState";
export type { SavedFit } from "./generated/SavedFit";
export type { UiPreferences } from "./generated/UiPreferences";
export type { CanvasLayers } from "./generated/CanvasLayers";
export type { MaterialChoice } from "./generated/MaterialChoice";
export type { ThemeChoice } from "./generated/ThemeChoice";
export type { Lang } from "./generated/Lang";
export type { SpeedScaleMode } from "./generated/SpeedScaleMode";

// Tipos que el backend devuelve pero no son structs con TS propio:
export interface MaterialInfo {
  id: string;
  nameEs: string;
  nameEn: string;
  symbol: string;
  workFunctionEv: number;
  thresholdFrequencyHz: number;
  thresholdWavelengthNm: number;
  source: string;
}

export interface SweepPoint {
  wavelengthNm: number;
  frequencyHz: number;
  stoppingIdealV: number;
  stoppingMeasuredV: number;
  errorV: number;
  materialId: string;
  materialName: string;
  noisePercent: number;
}

export interface SweepRequest {
  material: string;
  customPhiEv: number;
  count: number;
  noisePercent: number;
  /** u64 como string: JS pierde precisión por encima de 2^53. */
  seed: string;
}

export interface FitReport {
  nPoints: number;
  slopeVs: number;
  slopeStdVs: number;
  interceptV: number;
  interceptStdV: number;
  hExperimentalJs: number;
  hStdJs: number;
  hTheoreticalJs: number;
  errorPercentage: number;
  rSquared: number;
  residualsV: number[];
}

export interface SaveResult {
  path: string;
  bytes: number;
}
