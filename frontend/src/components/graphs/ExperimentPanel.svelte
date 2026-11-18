<script lang="ts">
  // Experimento V₀–f: tabla editable, barras de error, ajuste con intervalo
  // de confianza, h ± σ, error % y R², residuos y exportación CSV.
  import { lab } from "../../lib/store.svelte";
  import { t } from "../../lib/i18n";
  import { experimentCsv } from "../../lib/physics";
  import { downloadBlob, type PlotSpec } from "../../lib/plots";
  import { openReport } from "../../lib/io";
  import { planck, slope, signed } from "../../lib/format";
  import Plot from "./Plot.svelte";
  import Eq from "../Eq.svelte";

  const s = $derived(t(lab.preferences.lang));
  const fit = $derived(lab.fit);
  const points = $derived(lab.experiment.points);

  const fitSpec = $derived.by(() => {
    if (!fit || points.length < 2) return null;
    const xs = points.map((p) => p.frequencyHz / 1e14);
    const ys = points.map((p) => p.stoppingMeasuredV);
    const linePoints: { x: number; y: number }[] = [];
    const xMin = Math.min(...xs) * 0.94;
    const xMax = Math.max(...xs) * 1.06;
    for (let i = 0; i <= 40; i++) {
      const x = xMin + ((xMax - xMin) * i) / 40;
      linePoints.push({ x, y: fit.slopeVs * (x * 1e14) + fit.interceptV });
    }
    const yMin = Math.min(0, ...ys) ;
    const yMax = Math.max(...ys) * 1.12 + 0.1;
    return {
      area: { x: 58, y: 18, w: 500, h: 200 },
      view: { xMin, xMax, yMin, yMax },
      series: [
        { points: linePoints, color: "#67E8F9", width: 2, label: "ajuste" },
        {
          points: xs.map((x, i) => ({ x, y: ys[i] })),
          color: "#F59E0B",
          kind: "scatter" as const,
          errorY: points.map((p) => p.errorV),
        },
      ],
      xLabel: "f (×10¹⁴ Hz)",
      yLabel: "V₀ (V)",
      theme: resolveTheme(),
    } as PlotSpec;
  });

  const residualSpec = $derived.by(() => {
    if (!fit || points.length < 2) return null;
    const xs = points.map((p) => p.frequencyHz / 1e14);
    const residuals = fit.residualsV;
    const maxR = Math.max(1e-3, ...residuals.map((r) => Math.abs(r)));
    return {
      area: { x: 58, y: 18, w: 500, h: 120 },
      view: {
        xMin: Math.min(...xs) * 0.94,
        xMax: Math.max(...xs) * 1.06,
        yMin: -maxR * 1.4,
        yMax: maxR * 1.4,
      },
      series: [
        {
          points: xs.map((x, i) => ({ x, y: residuals[i] })),
          color: "#A78BFA",
          kind: "scatter" as const,
        },
      ],
      xLabel: "f (×10¹⁴ Hz)",
      yLabel: "residuo (V)",
      zeroLine: true,
      theme: resolveTheme(),
    } as PlotSpec;
  });

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

  async function exportCsv(): Promise<void> {
    const csv = await experimentCsv(lab.experiment);
    downloadBlob(new Blob([csv], { type: "text/csv;charset=utf-8" }), "experimento-v0-f.csv");
    lab.status = "CSV exportado (con ajuste en el pie).";
  }

  async function exportPdf(): Promise<void> {
    // Informe print-friendly: el navegador/WebView guarda como PDF.
    const rows = points
      .map(
        (p) =>
          `<tr><td>${p.wavelengthNm.toFixed(1)}</td><td>${(p.frequencyHz / 1e14).toFixed(3)}</td><td>${p.stoppingMeasuredV.toFixed(3)} ± ${p.errorV.toFixed(3)}</td><td>${p.stoppingIdealV.toFixed(3)}</td></tr>`,
      )
      .join("");
    const fitBlock = fit
      ? `<p><b>h medida:</b> ${planck(fit.hExperimentalJs)} ± ${(fit.hStdJs ?? 0).toExponential(2)} J·s ·
         <b>Error:</b> ${fit.errorPercentage.toFixed(3)} % · <b>R²:</b> ${fit.rSquared.toFixed(6)} ·
         <b>Pendiente:</b> ${slope(fit.slopeVs)}</p>`
      : "<p>Sin ajuste: se necesitan ≥ 2 puntos.</p>";
    const html = `<!doctype html><html lang="es"><head><meta charset="utf-8">
      <title>PhotoLab · informe V₀–f</title>
      <style>body{font-family:Inter,system-ui,sans-serif;margin:40px;color:#111}h1{font-size:20px}
      table{border-collapse:collapse;margin:16px 0;width:100%}td,th{border-bottom:1px solid #ddd;padding:6px;text-align:left;font-size:12px}
      .eq{font-family:ui-monospace,monospace;background:#f4f4f5;padding:10px;border-radius:8px}</style></head><body>
      <h1>PhotoLab · Experimento V₀–f</h1>
      <p>${new Date().toLocaleString()}</p>
      <p class="eq">V₀ = m·f + b &nbsp;·&nbsp; h = e·m &nbsp;·&nbsp; h teórica = 6.626×10⁻³⁴ J·s</p>
      <table><thead><tr><th>λ (nm)</th><th>f (×10¹⁴ Hz)</th><th>V₀ medido (V)</th><th>V₀ ideal (V)</th></tr></thead>
      <tbody>${rows}</tbody></table>
      ${fitBlock}
      <p>${lab.notes ? `<b>Notas:</b><br>${lab.notes.replace(/[<>&]/g, "")}` : ""}</p>
      <scr{""}ipt>window.print?.()</scr{""}ipt></body></html>`;
    const how = await openReport(html, "informe-v0-f.html");
    lab.status =
      how === "downloaded"
        ? "Informe descargado como HTML: ábrelo e imprímelo a PDF."
        : "Informe abierto en el visor del sistema: imprime a PDF desde ahí.";
  }

  function editValue(index: number, key: "stoppingMeasuredV" | "errorV", raw: string): void {
    const value = Number(raw);
    if (!Number.isFinite(value)) return;
    const next = points.map((p, i) => (i === index ? { ...p, [key]: value } : p));
    lab.experiment = { ...lab.experiment, points: next };
    void lab.refit();
    lab.status = "Punto editado: ajuste actualizado.";
  }
</script>

<div class="experiment rise">
  <div class="exp-head">
    <div>
      <div class="label">Experimento V₀–f</div>
      <div class="small muted">
        Mide V₀ a varias frecuencias: la pendiente de la recta es h/e.
      </div>
    </div>
    <div class="exp-actions">
      <button class="btn" onclick={() => lab.experimentalModeToggle()}>
        {lab.experiment.experimentalMode ? s.experimentMode : s.ideal}
      </button>
      <label class="noise">
        <span class="small muted">ruido ±{lab.experiment.noisePercent.toFixed(0)}%</span>
        <input
          type="range"
          min="0"
          max="10"
          step="0.5"
          value={lab.experiment.noisePercent}
          oninput={(e) => {
            lab.experiment = {
              ...lab.experiment,
              noisePercent: Number((e.currentTarget as HTMLInputElement).value),
            };
          }}
        />
      </label>
    </div>
  </div>

  <div class="exp-grid">
    <div class="table-col">
      <table class="data-table mono">
        <thead>
          <tr>
            <th>λ (nm)</th>
            <th>f (×10¹⁴ Hz)</th>
            <th>V₀ (V)</th>
            <th>σ (V)</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each points as point, i (i)}
            <tr class:selected={false}>
              <td>{point.wavelengthNm.toFixed(1)}</td>
              <td>{(point.frequencyHz / 1e14).toFixed(3)}</td>
              <td>
                <input
                  class="cell-input"
                  type="number"
                  step="0.001"
                  value={point.stoppingMeasuredV.toFixed(3)}
                  onchange={(e) => editValue(i, "stoppingMeasuredV", e.currentTarget.value)}
                  aria-label={`V₀ del punto ${i + 1}`}
                />
              </td>
              <td>
                <input
                  class="cell-input"
                  type="number"
                  step="0.001"
                  min="0"
                  value={point.errorV.toFixed(3)}
                  onchange={(e) => editValue(i, "errorV", e.currentTarget.value)}
                  aria-label={`Error del punto ${i + 1}`}
                />
              </td>
              <td>
                <button class="btn ghost icon" onclick={() => lab.removePoint(i)} aria-label="Quitar punto">✕</button>
              </td>
            </tr>
          {:else}
            <tr><td colspan="5" class="empty muted">Sin puntos: agrega el actual o haz un barrido.</td></tr>
          {/each}
        </tbody>
      </table>
      <div class="table-actions">
        <button class="btn primary" onclick={() => lab.addManualPoint()}>{s.addPoint}</button>
        <button class="btn" onclick={() => lab.autoSweep(6)}>{s.autoSweep}</button>
        <button class="btn ghost" onclick={() => lab.clearPoints()}>{s.clear}</button>
      </div>
      {#if lab.status}
        <div class="status small" role="status">{lab.status}</div>
      {/if}

      {#if fit}
        <div class="fit-card card">
          <div class="fit-grid mono">
            <span class="muted">h medida</span>
            <span class="strong">{planck(fit.hExperimentalJs)}</span>
            <span class="muted">± σ</span>
            <span>{(fit.hStdJs ?? 0).toExponential(2)} J·s</span>
            <span class="muted">error vs teórico</span>
            <span class:tone-good={fit.errorPercentage < 2}>{fit.errorPercentage.toFixed(3)} %</span>
            <span class="muted">pendiente</span>
            <span>{slope(fit.slopeVs)}</span>
            <span class="muted">ordenada b</span>
            <span>{signed(fit.interceptV, 3)}</span>
            <span class="muted">R²</span>
            <span>{fit.rSquared.toFixed(6)}</span>
          </div>
          <div class="fit-actions">
            <button class="btn" onclick={() => lab.saveFit()}>{s.saveFit}</button>
            <button class="btn" onclick={exportCsv}>{s.exportCsv}</button>
            <button class="btn" onclick={exportPdf}>{s.exportPdf}</button>
          </div>
          {#if lab.experiment.savedFits.length > 1}
            <div class="saved small muted">
              Comparación de pendientes (deben coincidir: es h/e):
              {#each lab.experiment.savedFits as saved}
                <span class="chip mono">{saved.materialName}: {slope(saved.slope)}</span>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="chart-col">
      {#if fitSpec && fit}
        <Plot spec={fitSpec} height={260} exportName="ajuste-v0-f" />
        <div class="ci small muted mono">
          V₀ = ({slope(fit.slopeVs)} ± {(fit.slopeStdVs ?? 0).toExponential(1)})·f + ({signed(fit.interceptV, 2)} ± {(fit.interceptStdV ?? 0).toFixed(3)})
        </div>
      {:else}
        <div class="placeholder card">
          <Eq latex="V_0 = m\\,f + b \\;,\\; h = e\\,m" display />
          <p class="xsmall muted">La pendiente es h/e para <em>cualquier</em> material.</p>
          <p class="small muted">Agrega al menos 2 puntos para ver el ajuste, el valor de h y los residuos.</p>
        </div>
      {/if}
      {#if residualSpec}
        <Plot spec={residualSpec} height={150} exportName="residuos-v0-f" />
      {/if}
    </div>
  </div>
</div>

<style>
  .experiment {
    padding: var(--sp-3);
    border-radius: var(--r-md);
    border: 1px solid var(--line);
    background: var(--panel);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  .exp-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--sp-3);
    flex-wrap: wrap;
  }
  .exp-actions {
    display: flex;
    gap: var(--sp-3);
    align-items: center;
  }
  .noise {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
  }
  .noise input {
    width: 96px;
  }
  .exp-grid {
    display: grid;
    grid-template-columns: minmax(300px, 0.9fr) minmax(320px, 1.1fr);
    gap: var(--sp-4);
  }
  @media (max-width: 980px) {
    .exp-grid {
      grid-template-columns: 1fr;
    }
  }
  .data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82em;
  }
  .data-table th {
    text-align: left;
    font-weight: 600;
    color: var(--muted);
    padding: 4px 6px;
    border-bottom: 1px solid var(--line);
    font-size: 0.92em;
  }
  .data-table td {
    padding: 2px 6px;
    border-bottom: 1px solid var(--line);
  }
  .cell-input {
    width: 72px;
    padding: 2px 4px;
    font-family: var(--font-mono);
    font-size: 0.95em;
    background: transparent;
    border: 1px solid transparent;
  }
  .cell-input:hover {
    border-color: var(--line-strong);
  }
  .table-actions {
    display: flex;
    gap: var(--sp-2);
    margin-top: var(--sp-2);
    flex-wrap: wrap;
  }
  .empty {
    padding: var(--sp-4) 6px;
  }
  .status {
    margin-top: var(--sp-2);
    color: var(--text-2);
    padding: 6px 8px;
    border-radius: var(--r-sm);
    background: var(--panel-2);
  }
  .fit-card {
    margin-top: var(--sp-3);
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .fit-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 2px var(--sp-3);
    font-size: 0.82em;
  }
  .fit-grid .strong {
    font-weight: 600;
    color: var(--text);
  }
  .tone-good {
    color: #34d399;
  }
  .fit-actions {
    display: flex;
    gap: var(--sp-2);
    flex-wrap: wrap;
  }
  .saved {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    align-items: center;
  }
  .placeholder {
    padding: var(--sp-6);
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    align-items: center;
  }
  .ci {
    padding: 0 var(--sp-2) var(--sp-2);
    color: var(--muted);
  }
</style>
