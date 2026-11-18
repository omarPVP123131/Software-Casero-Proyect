<script lang="ts">
  // Panel derecho: estado grande (EMITE / NO EMITE / BLOQUEADO) con color,
  // icono y forma; tarjetas de métricas con mini sparklines; explicación
  // causal con KaTeX; copiar/fijar valores; lector de pantalla vía aria-live.
  import { untrack } from "svelte";
  import { lab } from "../lib/store.svelte";
  import { t } from "../lib/i18n";
  import { wavelengthToColor } from "../lib/spectrum";
  import {
    current,
    ev,
    flux,
    fluxParts,
    hz,
    nm,
    percent,
    planck,
    scientific,
    speed,
    volts,
  } from "../lib/format";
  import { causalExplanation } from "../lib/causal";
  import Eq from "./Eq.svelte";

  const s = $derived(t(lab.preferences.lang));
  const r = $derived(lab.readout);
  const causal = $derived(
    r
      ? causalExplanation(
          r,
          { wavelengthNm: lab.controls.wavelengthNm, appliedVoltageV: lab.controls.appliedVoltageV },
          lab.titleFor(lab.controls.material),
          lab.preferences.lang,
        )
      : null,
  );

  let spark: Record<string, number[]> = $state({});

  // Históricos recientes para las sparklines. `untrack` evita que el efecto
  // se re-dispare al escribir `spark` (leerlo dentro sería un ciclo).
  $effect(() => {
    if (!r) return;
    const kmax = r.kMaxEv;
    const currentValue = r.photocurrentA;
    const collection = r.collectionFactor;
    untrack(() => {
      const next: Record<string, number[]> = { ...spark };
      const push = (key: string, value: number) => {
        const arr = [...(next[key] ?? []), value];
        if (arr.length > 40) arr.shift();
        next[key] = arr;
      };
      push("kmax", kmax);
      push("current", currentValue);
      push("collection", collection);
      spark = next;
    });
  });

  const stateLabel = $derived(
    lab.emissionState === "emit" ? s.emits : lab.emissionState === "blocked" ? s.blocked : s.noEmit,
  );
  const stateClass = $derived(
    lab.emissionState === "emit" ? "ok" : lab.emissionState === "blocked" ? "warn" : "off",
  );
  const stateIcon = $derived(
    lab.emissionState === "emit" ? "⬤" : lab.emissionState === "blocked" ? "⊘" : "○",
  );

  function sparkline(points: number[] | undefined, color: string): string {
    if (!points || points.length < 2) return "";
    const max = Math.max(...points, 1e-12);
    const min = Math.min(...points, 0);
    const w = 84;
    const h = 22;
    return points
      .map((p, i) => {
        const x = (i / (points.length - 1)) * w;
        const y = h - ((p - min) / Math.max(1e-12, max - min)) * h;
        return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
      })
      .join(" ");
  }

  async function copy(label: string, value: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(value);
      lab.status = `${label} copiado: ${value}`;
    } catch {
      lab.status = "No se pudo copiar al portapapeles.";
    }
  }

  // Secciones secundarias colapsables: en 1280×800 el contenido mide ~966 px
  // contra 722 px de alto, así que comparación y capas arrancan plegadas
  // (el usuario puede abrirlas y la elección se recuerda en la sesión).
  let openCompare = $state(false);
  let openLayers = $state(false);

  const metrics = $derived.by(() => {
    if (!r) return [];
    return [
      { key: "E", label: s.photonEnergy, long: s.photonEnergyLong, value: ev(r.photonEnergyEv), latex: "E = hf", spark: null, color: "var(--wl)" },
      { key: "f", label: s.frequency, value: hz(r.frequencyHz), latex: "f = c/λ", spark: null, color: "var(--wl)" },
      { key: "phi", label: s.workFunction, long: s.workFunctionLong, value: ev(r.workFunctionEv), latex: "\\Phi", spark: null, color: "var(--warn)" },
      { key: "l0", label: s.thresholdWl, value: nm(r.thresholdWavelengthNm), latex: "\\lambda_0 = c/f_0", spark: null, color: "var(--muted)" },
      { key: "k", label: s.kmax, value: ev(r.kMaxEv), full: `${ev(r.kMaxEv)} · ${scientific(r.kMaxJ, 2)} J`, latex: "K_{\\max} = hf-\\Phi", spark: spark.kmax, color: "#67E8F9" },
      { key: "v0", label: s.stoppingV, long: s.stoppingVLong, value: volts(r.stoppingPotentialV), latex: "V_0 = K_{\\max}/e", spark: spark.collection, color: "#F59E0B" },
      { key: "v", label: s.speed, long: s.speedLong, value: speed(r.electronMaxSpeedMS), latex: "v = \\sqrt{2K_{\\max}/m_e}", spark: null, color: "#67E8F9" },
      { key: "flux", label: s.photonFlux, long: s.photonFluxLong, value: fluxParts(r.photonFluxPerM2S)[0], unit: fluxParts(r.photonFluxPerM2S)[1], full: flux(r.photonFluxPerM2S), latex: "\\Phi_{fot} = P/E", spark: null, color: "var(--wl)" },
      { key: "i", label: s.photocurrent, long: s.photocurrentLong, value: current(r.photocurrentA), latex: "I = e\\Phi A\\,\\mathrm{QE}\\,g(V)", spark: spark.current, color: "#A78BFA" },
    ];
  });
</script>

<aside class="readouts" aria-label="Lecturas del experimento">
  <div class="state-hero" class:ok={stateClass === "ok"} class:warn={stateClass === "warn"} class:off={stateClass === "off"}>
    <div class="state-icon" aria-hidden="true">{stateIcon}</div>
    <div class="state-text">
      <div class="state-label" data-testid="emission-state">{stateLabel}</div>
      <div class="state-sub small mono">
        {lab.titleFor(lab.controls.material)} · {lab.controls.wavelengthNm.toFixed(0)} nm ·
        {lab.controls.appliedVoltageV.toFixed(2)} V
      </div>
    </div>
  </div>
  <span class="sr-only" aria-live="polite" data-testid="sr-state">
    {stateLabel}. {causal?.body ?? ""}
  </span>

  {#if causal}
    <div class="causal card rise">
      <div class="causal-head">
        <span class="label">{s.guidedCausal}</span>
        <span class="eq-mini"><Eq latex={r?.emits ? "h\\,f > \\Phi" : "h\\,f < \\Phi"} /></span>
      </div>
      <p class="causal-title">{causal.title}</p>
      <p class="causal-body small">{causal.body}</p>
      <div class="latex-wrap small"><Eq latex={causal.latex} display block /></div>
    </div>
  {/if}

  <div class="metrics">
    {#each metrics as metric (metric.key)}
      <div class="metric card">
        <div class="metric-head">
          <span class="metric-label small muted" title={(metric as { long?: string }).long ?? metric.label}>{metric.label}</span>
          <div class="metric-buttons">
            <button
              class="mini-btn"
              title={s.copyValue}
              aria-label="Copiar {metric.label}"
              onclick={() => copy(metric.label, metric.value)}
            >⧉</button>
            <button
              class="mini-btn"
              class:active={lab.pinned.some((p) => p.label === metric.label)}
              title={s.pinnedValue}
              aria-label="Fijar {metric.label}"
              onclick={() => lab.pin(metric.label, metric.value)}
            >⚲</button>
          </div>
        </div>
        <div class="metric-value mono" style:color={metric.color} title={(metric as { full?: string }).full ?? metric.value}>
          {metric.value}{#if (metric as { unit?: string }).unit}<span class="metric-unit">{(metric as { unit?: string }).unit}</span>{/if}
        </div>
        {#if metric.spark && metric.spark.length > 2}
          <svg class="spark" viewBox="0 0 84 22" aria-hidden="true">
            <path d={sparkline(metric.spark, metric.color)} fill="none" style:stroke={metric.color} stroke-width="1.4" opacity="0.7" />
          </svg>
        {/if}
      </div>
    {/each}
  </div>

  {#if lab.pinned.length > 0}
    <div class="pinned card">
      <div class="label">{s.pinned}</div>
      {#each lab.pinned as p (p.label)}
        <div class="pinned-row mono small">
          <span class="muted">{p.label}</span>
          <span>{p.value}</span>
          <button class="mini-btn" onclick={() => lab.pin(p.label, p.value)} aria-label="Quitar fijado">✕</button>
        </div>
      {/each}
    </div>
  {/if}

  <!-- comparación -->
  {#if lab.comparisonReadout}
    {@const c = lab.comparisonReadout}
    <div class="compare card">
      <button class="fold-head" aria-expanded={openCompare} onclick={() => (openCompare = !openCompare)}>
        <span class="label" title="Comparación · {lab.titleFor(lab.controls.comparisonMaterial)}">{s.comparison} · {lab.titleFor(lab.controls.comparisonMaterial)}</span>
        <span class="chev" class:rot={openCompare}>›</span>
      </button>
      {#if openCompare}
      <div class="compare-grid mono small">
        <div class="cmp-row"><span class="muted">Φ</span><span>{ev(c.workFunctionEv)}</span></div>
        <div class="cmp-row"><span class="muted">λ₀</span><span>{nm(c.thresholdWavelengthNm)}</span></div>
        <div class="cmp-row"><span class="muted">Kmax</span><span>{ev(c.kMaxEv)}</span></div>
        <div class="cmp-row">
          <span class="muted">{c.emits ? "emite" : "no emite"}</span>
          <span class="cmp-mark" class:cmp-emit={c.emits}>{c.emits ? "✓" : "✕"}</span>
        </div>
      </div>
      {/if}
    </div>
  {/if}

  <!-- layers -->
  <div class="layers card">
    <button class="fold-head" aria-expanded={openLayers} onclick={() => (openLayers = !openLayers)}>
      <span class="label">{s.layers}</span>
      <span class="chev" class:rot={openLayers}>›</span>
    </button>
    {#if openLayers}
    {#each [
      ["field", s.layerField],
      ["energyLabels", s.layerLabels],
      ["trajectories", s.layerTrajectories],
      ["grid", s.layerGrid],
      ["currentMeter", s.layerMeter],
      ["energyLadder", s.layerLadder],
    ] as [key, label] (key)}
      <label class="layer-row small">
        <span>{label}</span>
        <input
          type="checkbox"
          checked={lab.preferences.layers[key as keyof typeof lab.preferences.layers]}
          onchange={() => {
            const layers = { ...lab.preferences.layers };
            const k = key as keyof typeof layers;
            layers[k] = !layers[k];
            lab.setPreferences({ layers }, `Capa ${label}`);
          }}
        />
      </label>
    {/each}
    {/if}
  </div>
</aside>

<style>
  .readouts {
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    padding: var(--sp-2) var(--sp-2) var(--sp-3);
    overflow-y: auto;
    height: 100%;
  }
  .readouts > * {
    flex: 0 0 auto;
  }
  .fold-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    padding: 0;
    background: transparent;
    border: 0;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .fold-head:hover .label {
    color: var(--text);
  }
  .chev {
    color: var(--muted);
    transition: transform var(--dur-2) var(--ease);
  }
  .chev.rot {
    transform: rotate(90deg);
  }
  .state-hero {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
    padding: var(--sp-3);
    border-radius: var(--r-md);
    border: 1px solid var(--line);
    background: var(--panel);
  }
  .state-hero.ok {
    border-color: color-mix(in srgb, var(--ok) 50%, transparent);
    background: linear-gradient(135deg, var(--ok-soft), transparent 60%), var(--panel);
  }
  .state-hero.warn {
    border-color: color-mix(in srgb, var(--warn) 50%, transparent);
    background: linear-gradient(135deg, var(--warn-soft), transparent 60%), var(--panel);
  }
  .state-hero.off {
    border-color: var(--line-strong);
  }
  .state-icon {
    font-size: 1.6em;
    line-height: 1;
    color: var(--off);
  }
  .state-hero.ok .state-icon {
    color: var(--ok);
  }
  .state-hero.warn .state-icon {
    color: var(--warn);
  }
  .state-label {
    font-size: 1.28em;
    font-weight: 700;
    letter-spacing: 0.04em;
  }
  .state-hero.ok .state-label {
    color: var(--ok);
  }
  .state-hero.warn .state-label {
    color: var(--warn);
  }
  .state-sub {
    font-size: 0.7em;
    color: var(--muted);
    margin-top: 2px;
  }
  .causal {
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .causal-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .eq-mini {
    color: var(--wl);
    font-size: 0.78em;
  }
  .causal-title {
    margin: 0;
    font-weight: 600;
  }
  .causal-body {
    margin: 0;
    color: var(--text-2);
    line-height: 1.5;
  }
  .latex-wrap {
    font-size: 0.8em;
  }
  .metrics {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(118px, 1fr));
    gap: var(--sp-2);
  }
  .metric {
    padding: 7px 9px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .metric-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .metric-label {
    font-size: 0.74em;
    letter-spacing: 0.05em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .metric-unit {
    display: block;
    font-size: 0.78em;
    font-weight: 500;
    opacity: 0.75;
    line-height: 1.15;
  }
  .metric-value {
    font-size: 0.82em;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .metric-buttons {
    display: flex;
    gap: 0;
    opacity: 0;
    transition: opacity var(--dur-1) var(--ease);
  }
  .metric:hover .metric-buttons,
  .metric:focus-within .metric-buttons {
    opacity: 1;
  }
  .mini-btn {
    border: 0;
    background: transparent;
    color: var(--muted);
    font-size: 0.78em;
    padding: 0 3px;
  }
  .mini-btn:hover,
  .mini-btn.active {
    color: var(--wl);
  }
  .spark {
    width: 100%;
    height: 16px;
  }
  .pinned,
  .compare,
  .layers {
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .pinned-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .pinned-row span:nth-child(2) {
    margin-left: auto;
  }
  .compare-grid {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 0.8em;
  }
  .cmp-row {
    display: flex;
    justify-content: space-between;
    gap: 10px;
  }
  .cmp-mark {
    color: var(--off);
    font-weight: 700;
  }
  .cmp-emit {
    color: var(--ok);
  }
  .layer-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
</style>
