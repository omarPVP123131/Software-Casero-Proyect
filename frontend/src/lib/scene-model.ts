// Geometría y partículas de la escena, deterministas en (tiempo, λ).
//
// No dependen del framerate: todo se calcula a partir de `t` absoluto. El
// número de fotones depende de la intensidad, la longitud de onda fija el
// tamaño del paquete y Kmax fija la rapidez de los electrones.

export interface SceneGeom {
  x0: number;
  y0: number;
  w: number;
  h: number;
  sourceX: number;
  cathodeX: number;
  anodeX: number;
  plateTop: number;
  plateBottom: number;
  midY: number;
  batteryY: number;
}

/**
 * Geometría de la escena. Las placas escalan con el alto disponible (antes se
 * topaban a 300 px y en lienzos de ~660 px quedaba un 30 % de alto vacío).
 * La batería baja lo justo para no encimar las etiquetas de los electrodos.
 */
export function sceneGeometry(w: number, h: number): SceneGeom {
  const pad = Math.max(18, w * 0.035);
  // Las placas crecen con el lienzo pero la batería debe quedar justo debajo:
  // el conjunto (placas + hueco + batería) ocupa el alto disponible.
  const plateH = Math.min(h * 0.6, 470);
  const midY = h * 0.4;
  const batteryGap = Math.max(50, h * 0.085);
  return {
    x0: 0,
    y0: 0,
    w,
    h,
    sourceX: pad + Math.min(64, w * 0.08),
    cathodeX: w * 0.34,
    anodeX: w * 0.76,
    plateTop: midY - plateH / 2,
    plateBottom: midY + plateH / 2,
    midY,
    batteryY: h - batteryGap,
  };
}

export interface Photon {
  /** fase 0..1 en el trayecto fuente→cátodo */
  phase: number;
  /** desfase vertical estable */
  yOff: number;
  /** 0..1 "cuántico": se absorbe o se refleja */
  seed: number;
}

/**
 * Fotones visibles para un instante `t` (s). Densidad ∝ intensidad,
 * tamaño de onda ∝ λ. Deterministas: mismos t, λ, I ⇒ mismas partículas.
 * `travel` es la duración del trayecto (s) y depende solo de λ de forma
 * visual (nunca física: la luz no "va más lenta").
 */
export function photonsAt(t: number, intensity: number, travel = 1.6): Photon[] {
  const count = Math.round(3 + 17 * Math.min(1, Math.max(0, intensity / 100)));
  const out: Photon[] = [];
  for (let i = 0; i < count; i++) {
    const speed = 0.55 + 0.45 * hash01(i * 7.13);
    const phase = ((t / travel) * speed + i / count) % 1;
    out.push({
      phase,
      yOff: (hash01(i * 3.77 + 1.7) - 0.5) * 0.72,
      seed: hash01(i * 11.3 + 5.1),
    });
  }
  return out;
}

export interface Electron {
  /** 0..1 progreso cátodo→ánodo */
  progress: number;
  lane: number;
  /** variación de energía (distribución de Maxwell simplificada) */
  energy: number;
  seed: number;
}

/**
 * Electrones emitidos. `count` ∝ intensidad·(1−frac bloqueo). Con frenado,
 * los electrones visibles regresan (progress baja hasta 0) en vez de
 * desaparecer: la UI dibuja el giro con `turn`.
 */
export function electronsAt(
  t: number,
  count: number,
  speed: number,
  retarding: number,
): Electron[] {
  const out: Electron[] = [];
  const n = Math.round(count);
  for (let i = 0; i < n; i++) {
    const energy = 0.55 + 0.45 * hash01(i * 5.31 + 2.2);
    const v = speed * (0.6 + 0.4 * energy);
    let progress = ((t * v * 0.32 + i / Math.max(1, n)) % 1 + 1) % 1;
    // Si hay frenado, el electrón no puede pasar de su punto de retorno.
    const reach = Math.max(0.04, 1 - retarding) * energy;
    let turn = 0;
    if (progress > reach) {
      const past = progress - reach;
      progress = Math.max(0, reach - past);
      turn = 1;
    }
    out.push({
      progress,
      lane: (hash01(i * 9.7 + 0.3) - 0.5) * 0.8,
      energy,
      seed: hash01(i * 2.13),
    });
  }
  return out;
}

/** Hash determinista → [0,1). */
export function hash01(n: number): number {
  const x = Math.sin(n * 127.1 + 311.7) * 43758.5453123;
  return x - Math.floor(x);
}

/** Número de electrones visibles: ∝ intensidad y colección. */
export function electronCount(intensity: number, collection: number, max = 22): number {
  return Math.round(max * Math.min(1, Math.max(0, intensity / 100)) * Math.min(1, collection));
}
