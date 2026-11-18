<script lang="ts">
  // Pestañas de gráficas: I–V · Kmax(λ) · Experimento V₀–f.
  import { lab, type Tab } from "../../lib/store.svelte";
  import { t } from "../../lib/i18n";
  import { wavelengthToColor } from "../../lib/spectrum";
  import { sScale, type PlotSpec } from "../../lib/plots";
  import Plot from "./Plot.svelte";
  import ExperimentPanel from "./ExperimentPanel.svelte";

  let { tab }: { tab: Tab } = $props();

  const s = $derived(t(lab.preferences.lang));

  function resolveTheme() {
    const styles = getComputedStyle(document.documentElement);
    const css = (name: string) => styles.getPropertyValue(name).trim();
    return {
      text: css("--text") || "#e8edf7",
      muted: css("--muted") || "#7c8aa5",
      line: css("--line-strong") || "rgba(255,255,255,.14)",
      grid: css("--line") || "rgba(255,255,255,.08)",
      panel: css("--panel-solid") || "#0f1526",
    };
  }
  const readout = $derived(lab.readout);
  const comparison = $derived(lab.comparisonReadout);
  const wlColor = $derived(wavelengthToColor(lab.controls.wavelengthNm).css);

  // ---------- I–V ----------
  const ivSpec = $derived.by(() => {
    const r = readout;
    if (!r) return null;
    const v0 = r.stoppingPotentialV;
    const vMin = Math.min(-5, -(v0 + 1));
    const vMax = 5;
    const steps = 161;
    const curve: { x: number; y: number }[] = [];
    const cmp: { x: number; y: number }[] = [];
    for (let i = 0; i < steps; i++) {
      const v = vMin + ((vMax - vMin) * i) / (steps - 1);
      let g = 0;
      if (r.emits) {
        if (v >= 0) g = 1;
        else if (v0 > 0 && v > -v0) g = (v + v0) / v0;
      }
      const currentA = r.photocurrentA === 0 ? 0 : g;
      curve.push({ x: v, y: currentA });
      if (comparison) {
        const c0 = comparison.stoppingPotentialV;
        let cg = 0;
        if (comparison.emits) {
          if (v >= 0) cg = 1;
          else if (c0 > 0 && v > -c0) cg = (v + c0) / c0;
        }
        cmp.push({ x: v, y: cg * (comparison.emits ? 1 : 0) });
      }
    }
    const series: PlotSpec["series"] = [
      {
        points: curve,
        color: wlColor,
        width: 2.4,
      },
    ];
    if (lab.controls.comparisonMaterial !== lab.controls.material) {
      series.push({
        points: cmp,
        color: "rgba(148,163,184,0.8)",
        width: 1.6,
        dash: [6, 5],
      });
    }
    const spec: PlotSpec = {
      area: { x: 58, y: 18, w: 500, h: 200 },
      view: { xMin: vMin, xMax: vMax, yMin: -0.06, yMax: 1.15 },
      series,
      xLabel: "V aplicado (V)",
      yLabel: "I (u. a.) · colección g(V)",
      zeroLine: true,
      marker:
        r.emits && v0 > 0
          ? { x: -v0, color: "#F59E0B", label: `−V₀ = ${(-v0).toFixed(2)} V` }
          : undefined,
      theme: resolveTheme(),
    };
    return { spec, v0 };
  });

  const comparisonLabel = $derived(
    lab.controls.comparisonMaterial
      ? lab.titleFor(lab.controls.comparisonMaterial)
      : "",
  );
  const comparisonIsCurrent = $derived(
    lab.controls.comparisonMaterial === lab.controls.material,
  );

  // ---------- Kmax(λ) ----------
  const kmaxSpec = $derived.by(() => {
    const r = readout;
    if (!r || !lab.response) return null;
    const materialCurve = lab.response.curve.map((p) => ({
      x: p.wavelengthNm,
      y: p.kMaxEv,
    }));
    const compareCurve = lab.response.comparisonCurve?.map((p) => ({
      x: p.wavelengthNm,
      y: p.kMaxEv,
    }));
    const series: PlotSpec["series"] = [
      { points: materialCurve, color: wlColor, width: 2.4 },
    ];
    if (compareCurve && compareCurve.length > 0 && !comparisonIsCurrent) {
      series.push({
        points: compareCurve,
        color: "rgba(148,163,184,0.85)",
        width: 1.6,
        dash: [6, 5],
      });
    }
    const spec: PlotSpec = {
      area: { x: 58, y: 18, w: 500, h: 200 },
      view: { xMin: 180, xMax: 900, yMin: 0, yMax: 5 },
      series,
      xLabel: "λ (nm)",
      yLabel: "Kmax (eV)",
      marker: {
        x: lab.controls.wavelengthNm,
        color: wlColor,
        label: `${lab.controls.wavelengthNm.toFixed(0)} nm`,
      },
      cursor: { x: lab.controls.wavelengthNm, y: r.kMaxEv },
      theme: resolveTheme(),
    };
    return spec;
  });
</script>

<div class="graphs">
  {#if tab === "iv" && ivSpec}
    <div class="graph-card rise">
      <div class="graph-head">
        <div>
          <div class="label">Curva I–V</div>
          <div class="small muted">
            Rampa de colección con el material actual; la marca −V₀ indica el
            voltaje de frenado exacto.
          </div>
        </div>
        <span class="chip mono swatch-chip"><i style:background={wlColor}></i> λ {lab.controls.wavelengthNm.toFixed(0)} nm</span>
      </div>
      <Plot spec={ivSpec.spec} height={250} exportName="iv" />
      <div class="legend small">
        <span><i style:background={wlColor}></i> {lab.titleFor(lab.controls.material)}</span>
        {#if comparisonLabel && !comparisonIsCurrent}
          <span class="muted"><i class="dash"></i> {comparisonLabel} (comparación)</span>
        {/if}
      </div>
    </div>
  {/if}

  {#if tab === "kmax" && kmaxSpec}
    <div class="graph-card rise">
      <div class="graph-head">
        <div>
          <div class="label">Kmax(λ)</div>
          <div class="small muted">
            La pendiente cambia con Φ: la recta cruza λ₀ exactamente en Kmax = 0.
          </div>
        </div>
      </div>
      <Plot spec={kmaxSpec} height={250} exportName="kmax-lambda" />
    </div>
  {/if}

  {#if tab === "experiment"}
    <ExperimentPanel />
  {/if}
</div>

<style>
  .graphs {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: var(--sp-3);
  }
  .graph-card {
    padding: var(--sp-3);
    border-radius: var(--r-md);
    border: 1px solid var(--line);
    background: var(--panel);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .graph-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--sp-3);
  }
  .legend {
    display: flex;
    gap: var(--sp-4);
    padding-left: var(--sp-2);
    color: var(--text-2);
  }
  .swatch-chip {
    white-space: nowrap;
  }
  .swatch-chip i {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .legend i {
    display: inline-block;
    width: 16px;
    height: 3px;
    border-radius: 2px;
    vertical-align: middle;
    margin-right: 4px;
  }
  .legend i.dash {
    background: repeating-linear-gradient(
      90deg,
      rgba(148, 163, 184, 0.85) 0 5px,
      transparent 5px 9px
    );
  }
</style>
