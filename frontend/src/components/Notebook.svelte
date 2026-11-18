<script lang="ts">
  // Cuaderno de laboratorio: notas por sesión, capturas fijadas y export a PDF.
  import { lab } from "../lib/store.svelte";
  import { openReport } from "../lib/io";

  let notesEl: HTMLTextAreaElement | null = $state(null);
  let snapshots = $state<{ label: string; dataUrl: string; at: number }[]>([]);
  let caption = $state("");

  function capture(): void {
    const canvas = lab.sceneCanvas;
    if (!canvas) return;
    const label = caption.trim() || `λ ${lab.controls.wavelengthNm.toFixed(0)} nm · V ${lab.controls.appliedVoltageV.toFixed(2)} V`;
    snapshots = [
      ...snapshots,
      { label, dataUrl: canvas.toDataURL("image/png"), at: Date.now() },
    ].slice(-6);
    caption = "";
    lab.pushHistory(`Captura fijada: ${label}`);
  }

  function remove(index: number): void {
    snapshots = snapshots.filter((_, i) => i !== index);
  }

  async function exportPdf(): Promise<void> {
    const images = snapshots
      .map(
        (s) =>
          `<figure style="margin:12px 0"><img src="${s.dataUrl}" style="width:100%;border:1px solid #ddd;border-radius:8px"/><figcaption style="font-size:12px;color:#555">${escapeHtml(s.label)}</figcaption></figure>`,
      )
      .join("");
    const html = `<!doctype html><html lang="es"><head><meta charset="utf-8"><title>PhotoLab · cuaderno</title>
      <style>body{font-family:Inter,system-ui,sans-serif;margin:40px;color:#111;max-width:820px}
      h1{font-size:20px} pre{white-space:pre-wrap;font-family:ui-monospace,monospace;background:#f4f4f5;padding:12px;border-radius:8px}
      .mono{font-family:ui-monospace,monospace;font-size:12px}</style></head><body>
      <h1>PhotoLab — cuaderno de laboratorio</h1>
      <p class="mono">${new Date().toLocaleString()} · material ${lab.titleFor(lab.controls.material)} · λ ${lab.controls.wavelengthNm.toFixed(0)} nm · I ${lab.controls.intensityPercent.toFixed(0)} % · V ${lab.controls.appliedVoltageV.toFixed(2)} V</p>
      <p class="mono">h medida: ${lab.fit ? `${lab.fit.hExperimentalJs.toExponential(3)} J·s (error ${lab.fit.errorPercentage.toFixed(2)} %)` : "sin ajuste"}</p>
      <h2 style="font-size:15px">Notas</h2><pre>${escapeHtml(lab.notes)}</pre>
      <h2 style="font-size:15px">Capturas</h2>${images || "<p>Sin capturas</p>"}
      <scr{""}ipt>window.print?.()</scr{""}ipt></body></html>`;
    const how = await openReport(html, "cuaderno-photolab.html");
    lab.pushHistory(
      how === "downloaded"
        ? "Cuaderno descargado como HTML (ventana emergente bloqueada)"
        : "Cuaderno abierto para imprimir a PDF",
    );
  }

  function escapeHtml(text: string): string {
    return text.replace(/[<>&]/g, (c) => ({ "<": "&lt;", ">": "&gt;", "&": "&amp;" })[c] ?? c);
  }
</script>

<div class="notebook">
  <div class="nb-head">
    <div>
      <div class="label">Cuaderno de laboratorio</div>
      <div class="small muted">Las notas se guardan dentro de la sesión `.photolab.json`.</div>
    </div>
    <button class="btn" onclick={exportPdf}>Exportar PDF</button>
  </div>
  <textarea
    bind:this={notesEl}
    bind:value={lab.notes}
    placeholder="Observaciones, predicciones, conclusiones…"
    rows="4"
    ondblclick={() => notesEl?.focus()}
    aria-label="Notas de laboratorio"
  ></textarea>
  <div class="capture-row">
    <input type="text" bind:value={caption} placeholder="Pie de la captura (λ, V…)" aria-label="Pie de la captura" />
    <button class="btn" onclick={capture} disabled={!lab.sceneCanvas}>Fijar captura del canvas</button>
  </div>
  {#if snapshots.length > 0}
    <div class="thumbs">
      {#each snapshots as snap, i (snap.at)}
        <figure class="thumb">
          <img src={snap.dataUrl} alt={snap.label} />
          <figcaption class="xsmall">
            {snap.label}
            <button class="mini-btn" onclick={() => remove(i)} aria-label="Quitar captura">✕</button>
          </figcaption>
        </figure>
      {/each}
    </div>
  {/if}
  <div class="hist">
    <div class="label">Historial</div>
    {#each [...lab.history].slice(-6).reverse() as entry}
      <div class="hist-row small">
        <span class="mono muted">{entry.at.toFixed(1)} s</span>
        <span class="small">{entry.text}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .notebook {
    padding: var(--sp-3);
    border-radius: var(--r-md);
    border: 1px solid var(--line);
    background: var(--panel);
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  .nb-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--sp-3);
  }
  textarea {
    width: 100%;
    resize: vertical;
    font-size: 0.86em;
    min-height: 72px;
  }
  .capture-row {
    display: flex;
    gap: 8px;
  }
  .capture-row input {
    flex: 1;
    font-size: 0.84em;
  }
  .thumbs {
    display: flex;
    gap: 8px;
    overflow-x: auto;
  }
  .thumb {
    position: relative;
    margin: 0;
    min-width: 140px;
  }
  .thumb img {
    width: 160px;
    border-radius: var(--r-sm);
    border: 1px solid var(--line);
  }
  .thumb .mini-btn {
    background: transparent;
    border: 0;
    color: var(--muted);
    padding: 0 2px;
  }
  .thumb .mini-btn:hover {
    color: var(--danger);
  }
  figcaption {
    color: var(--muted);
    font-size: 0.7em;
  }
  .hist {
    border-top: 1px solid var(--line);
    padding-top: var(--sp-2);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .hist-row {
    display: flex;
    gap: 10px;
    color: var(--text-2);
  }
</style>
