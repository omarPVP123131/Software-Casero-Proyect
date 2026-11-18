// Estado único de la sesión (spec §1): un store con el contrato tipado
// generado desde Rust. Toda mutación de controles recalcula la física UNA vez
// (microtask coalescing: varios sets en el mismo tick → un solo evaluate).
import type {
  EvaluateResponse,
  ExperimentControls,
  ExperimentPoint,
  ExperimentState,
  FitReport,
  MaterialChoice,
  MaterialInfo,
  Readout,
  SavedFit,
  Session,
  UiPreferences,
} from "./types";
import { evaluate, fitExperiment, materialCatalog, sweepExperiment } from "./physics";
import { fitReportLocal } from "./fit-local";

export type Tab = "cell" | "iv" | "kmax" | "experiment";

export interface HistoryEntry {
  at: number;
  text: string;
}

export const DEFAULT_CONTROLS: ExperimentControls = {
  material: "sodium",
  comparisonMaterial: "copper",
  customPhiEv: 3.0,
  comparisonCustomPhiEv: 3.0,
  wavelengthNm: 400,
  intensityPercent: 55,
  appliedVoltageV: 0,
  cathodeAreaCm2: 1,
  quantumEfficiencyPercent: 1,
};

export const DEFAULT_PREFERENCES: UiPreferences = {
  theme: "dark",
  lang: "es",
  fontScale: 1,
  speedScale: "compressed",
  motion: "full",
  layers: {
    field: true,
    energyLabels: true,
    trajectories: true,
    grid: true,
    currentMeter: true,
    energyLadder: true,
  },
  controlsPanelOpen: true,
  readoutsPanelOpen: true,
  controlsPanelWidth: 300,
  readoutsPanelWidth: 308,
  presentationMode: false,
};

export const DEFAULT_EXPERIMENT: ExperimentState = {
  points: [],
  experimentalMode: false,
  noisePercent: 2,
  nextSeed: "11400714819323198485",
  savedFits: [],
};

function cloneSession(session: Session): Session {
  return JSON.parse(JSON.stringify(session)) as Session;
}

class LabStore {
  controls = $state<ExperimentControls>(structuredClone(DEFAULT_CONTROLS));
  preferences = $state<UiPreferences>(structuredClone(DEFAULT_PREFERENCES));
  experiment = $state<ExperimentState>(structuredClone(DEFAULT_EXPERIMENT));
  notes = $state("");
  response = $state<EvaluateResponse | null>(null);
  catalog = $state<MaterialInfo[]>([]);
  fit = $state<FitReport | null>(null);
  tab = $state<Tab>("cell");
  animating = $state(true);
  slowMotion = $state(false);
  timeScale = $state(1);
  animTime = $state(0);
  status = $state("");
  history = $state<HistoryEntry[]>([{ at: 0, text: "Sesión iniciada · fotoelectrico-physics" }]);
  undoStack: Session[] = [];
  redoStack: Session[] = [];
  pinned = $state<{ label: string; value: string }[]>([]);
  tutorial = $state<{ active: boolean; step: number }>({ active: false, step: 0 });
  sceneCanvas: HTMLCanvasElement | null = null;
  plottingCanvas: HTMLCanvasElement | null = null;
  private pending: Promise<void> | null = null;
  private queueAgain = false;
  ready = false;

  async init(): Promise<void> {
    if (this.ready) return;
    this.ready = true;
    this.catalog = await materialCatalog();
    await this.refresh();
    this.refit();
  }

  snapshot(): Session {
    return cloneSession({
      schemaVersion: 1,
      app: "photolab",
      controls: this.controls,
      preferences: this.preferences,
      experiment: this.experiment,
      notes: this.notes,
    } as Session);
  }

  loadSession(session: Session): void {
    this.controls = { ...DEFAULT_CONTROLS, ...session.controls };
    this.preferences = { ...structuredClone(DEFAULT_PREFERENCES), ...session.preferences };
    const experiment = { ...structuredClone(DEFAULT_EXPERIMENT), ...session.experiment };
    // Las sesiones antiguas guardaban la semilla como número; el contrato TS
    // la expone como string para no perder los 64 bits.
    experiment.nextSeed = String(experiment.nextSeed ?? DEFAULT_EXPERIMENT.nextSeed);
    this.experiment = experiment;
    this.notes = session.notes ?? "";
    this.pushHistory("Sesión cargada");
    void this.refresh();
    this.refit();
  }

  materialInfo(id: MaterialChoice): MaterialInfo | undefined {
    return this.catalog.find((m) => m.id === id);
  }

  titleFor(id: MaterialChoice): string {
    const en = this.preferences.lang === "en";
    if (id === "custom") return en ? "Custom" : "Personalizado";
    const info = this.materialInfo(id);
    if (!info) return id;
    return `${en ? info.nameEn : info.nameEs} (${info.symbol})`;
  }

  get readout(): Readout | null {
    return this.response?.readout ?? null;
  }

  get comparisonReadout(): Readout | null {
    return this.response?.comparison ?? null;
  }

  get emissionState(): "emit" | "none" | "blocked" {
    const r = this.readout;
    if (!r) return "none";
    if (r.blocked) return "blocked";
    return r.emits ? "emit" : "none";
  }

  pushHistory(text: string): void {
    this.history.push({ at: this.animTime, text });
    if (this.history.length > 120) this.history.splice(0, this.history.length - 120);
  }

  private recordUndo(label: string): void {
    this.undoStack.push(this.snapshot());
    if (this.undoStack.length > 60) this.undoStack.shift();
    this.redoStack = [];
    this.pushHistory(label);
  }

  /** Mutación de controles con undo + recálculo coalescido. */
  update(mutate: (controls: ExperimentControls) => void, label = "Controles actualizados"): void {
    this.recordUndo(label);
    const next = { ...this.controls };
    mutate(next);
    this.controls = next;
    this.refresh();
  }

  undo(): void {
    const previous = this.undoStack.pop();
    if (!previous) return;
    this.redoStack.push(this.snapshot());
    this.controls = previous.controls;
    this.preferences = previous.preferences;
    this.experiment = previous.experiment;
    this.notes = previous.notes;
    this.pushHistory("Deshacer");
    this.refresh();
    this.refit();
  }

  redo(): void {
    const next = this.redoStack.pop();
    if (!next) return;
    this.undoStack.push(this.snapshot());
    this.controls = next.controls;
    this.preferences = next.preferences;
    this.experiment = next.experiment;
    this.notes = next.notes;
    this.pushHistory("Rehacer");
    this.refresh();
    this.refit();
  }

  setPreferences(patch: Partial<UiPreferences>, label?: string): void {
    this.recordUndo(label ?? "Preferencias");
    this.preferences = { ...this.preferences, ...patch };
  }

  /**
   * Recalcula la física. Coalesce llamadas del mismo tick (los sliders
   * pueden emitir 60 eventos/s, pero el motor corre ~1 vez por frame como
   * máximo y la curva solo cambia con el material).
   */
  refresh(): void {
    if (this.pending) {
      this.queueAgain = true;
      return;
    }
    this.pending = Promise.resolve()
      .then(() => evaluate(this.controls, this.preferences))
      .then((response) => {
        this.response = response;
      })
      .catch((error) => {
        this.status = `motor: ${String(error)}`;
      })
      .finally(() => {
        this.pending = null;
        if (this.queueAgain) {
          this.queueAgain = false;
          this.refresh();
        }
      });
  }

  /** Autoguardado ligero: solo preferencias de UI (localStorage). */
  persistLocal(): void {
    try {
      localStorage.setItem(
        "photolab.ui",
        JSON.stringify({ preferences: $state.snapshot(this.preferences) }),
      );
    } catch {
      /* almacenamiento lleno o bloqueado */
    }
  }

  restoreLocal(): void {
    try {
      const raw = localStorage.getItem("photolab.ui");
      if (!raw) return;
      const parsed = JSON.parse(raw) as { preferences?: Partial<UiPreferences> };
      if (parsed.preferences) {
        this.preferences = {
          ...this.preferences,
          ...parsed.preferences,
          layers: { ...this.preferences.layers, ...(parsed.preferences.layers ?? {}) },
        };
      }
    } catch {
      /* JSON corrupto: seguir con defaults */
    }
  }

  // ---------- Experimento V₀–f ----------

  async refit(): Promise<void> {
    if (this.experiment.points.length < 2) {
      this.fit = null;
      return;
    }
    this.fit = (await fitExperiment(this.experiment.points)) ?? fitReportLocal(this.experiment.points);
  }

  async addManualPoint(): Promise<void> {
    const r = this.readout;
    if (!r) return;
    if (!r.emits) {
      this.status = "Sin emisión a esta λ (λ > λ₀): baja la longitud de onda.";
      return;
    }
    if (this.experiment.points.length >= 24) {
      this.status = "Tabla llena (24 puntos). Quita alguno.";
      return;
    }
    const duplicate = this.experiment.points.some(
      (p) => Math.abs(p.wavelengthNm - this.controls.wavelengthNm) < 0.5,
    );
    if (duplicate) {
      this.status = "Ya hay un punto con esa λ: cámbiala al menos 1 nm.";
      return;
    }
    const first = this.experiment.points[0];
    const materialId = this.controls.material;
    if (first && first.materialId !== materialId) {
      this.status = `La tabla es de ${first.materialName}: vacíala para medir otro material.`;
      return;
    }
    this.recordUndo("Agregar punto V₀–f");
    const errorV = this.experiment.experimentalMode
      ? Math.abs(r.stoppingPotentialV) * (this.experiment.noisePercent / 100)
      : 0;
    const measured =
      errorV === 0 ? r.stoppingPotentialV : Math.max(0, r.stoppingPotentialV + (Math.random() * 2 - 1) * errorV);
    const info = this.materialInfo(materialId);
    this.experiment = {
      ...this.experiment,
      points: [
        ...this.experiment.points,
        {
          wavelengthNm: this.controls.wavelengthNm,
          frequencyHz: r.frequencyHz,
          stoppingIdealV: r.stoppingPotentialV,
          stoppingMeasuredV: measured,
          errorV,
          materialId,
          materialName: info?.nameEs ?? "Material",
          noisePercent: errorV > 0 ? this.experiment.noisePercent : 0,
        },
      ],
    };
    this.status = `Punto ${this.experiment.points.length} agregado: λ = ${this.controls.wavelengthNm.toFixed(0)} nm.`;
    await this.refit();
  }

  async autoSweep(count = 6): Promise<void> {
    this.recordUndo("Barrido automático V₀–f");
    const points = await sweepExperiment({
      material: this.controls.material,
      customPhiEv: this.controls.customPhiEv,
      count,
      noisePercent: this.experiment.experimentalMode ? this.experiment.noisePercent : 0,
      seed: this.experiment.nextSeed,
    });
    if (points.length === 0) {
      this.status = "El umbral deja menos de dos puntos útiles: usa otro material.";
      return;
    }
    const mapped: ExperimentPoint[] = points.map((p) => ({
      wavelengthNm: p.wavelengthNm,
      frequencyHz: p.frequencyHz,
      stoppingIdealV: p.stoppingIdealV,
      stoppingMeasuredV: p.stoppingMeasuredV,
      errorV: p.errorV,
      materialId: p.materialId,
      materialName: p.materialName,
      noisePercent: p.noisePercent,
    }));
    this.experiment = {
      ...this.experiment,
      points: mapped,
      nextSeed: String(BigInt(this.experiment.nextSeed) + 1n),
    };
    this.status = `Barrido: ${mapped.length} puntos (${mapped[0].materialName}).`;
    await this.refit();
  }

  experimentalModeToggle(): void {
    this.experiment = {
      ...this.experiment,
      experimentalMode: !this.experiment.experimentalMode,
    };
    this.pushHistory(
      this.experiment.experimentalMode ? "Modo experimental (ruido)" : "Modo ideal (V₀ exacto)",
    );
  }

  removePoint(index: number): void {
    this.recordUndo("Quitar punto");
    this.experiment = {
      ...this.experiment,
      points: this.experiment.points.filter((_, i) => i !== index),
    };
    void this.refit();
  }

  clearPoints(): void {
    this.recordUndo("Vaciar tabla");
    this.experiment = { ...this.experiment, points: [] };
    this.fit = null;
  }

  saveFit(): void {
    if (!this.fit) return;
    const first = this.experiment.points[0];
    const saved: SavedFit = {
      materialId: first?.materialId ?? "",
      materialName: first?.materialName ?? "",
      nPoints: this.fit.nPoints,
      slope: this.fit.slopeVs,
      intercept: this.fit.interceptV,
      hExperimental: this.fit.hExperimentalJs,
      errorPercentage: this.fit.errorPercentage,
      rSquared: this.fit.rSquared,
    };
    const others = this.experiment.savedFits.filter((f) => f.materialId !== saved.materialId);
    this.experiment = {
      ...this.experiment,
      savedFits: [...others, saved].slice(-6),
    };
    this.status = "Ajuste guardado para comparar materiales.";
  }

  pin(label: string, value: string): void {
    if (this.pinned.some((p) => p.label === label)) {
      this.pinned = this.pinned.filter((p) => p.label !== label);
    } else {
      this.pinned = [...this.pinned, { label, value }].slice(-4);
    }
  }
}

export const lab = new LabStore();
