<script lang="ts">
  // Gráfica interactiva reutilizable: ejes con unidades, cursor con lectura,
  // zoom/pan (rueda y arrastre) y export PNG/SVG. El dibujo es Canvas 2D
  // (uPlot no hace falta: pocas series y queremos KaTeX/etiquetas flexibles).
  import { onMount } from "svelte";
  import {
    canvasToPng,
    downloadBlob,
    drawPlot,
    plotToSvg,
    sScale,
    type PlotSpec,
    type PlotView,
  } from "../../lib/plots";
  import { lab } from "../../lib/store.svelte";

  let {
    spec,
    height = 260,
    aspect = 2.6,
    exportName = "grafica",
    homeView,
  }: {
    spec: PlotSpec;
    height?: number;
    aspect?: number;
    exportName?: string;
    homeView?: PlotView;
  } = $props();

  let canvas: HTMLCanvasElement | null = null;
  let wrap: HTMLDivElement | null = null;
  let W = $state(640);
  let H = $state(height);
  let view = $state<PlotView>(structuredClone(spec.view));
  let hover = $state<{ x: number; y: number } | null>(null);
  let dragging: { startX: number; startY: number; view: PlotView } | null = null;
  let dpr = 1;

  const theme = $derived.by(() => {
    const styles = getComputedStyle(document.documentElement);
    const css = (name: string) => styles.getPropertyValue(name).trim();
    return {
      text: css("--text") || "#e8edf7",
      muted: css("--muted") || "#7c8aa5",
      line: css("--line-strong") || "rgba(255,255,255,.14)",
      grid: css("--line") || "rgba(255,255,255,.08)",
      panel: css("--panel-solid") || "#0f1526",
    };
  });

  const area = $derived({ x: 58, y: 18, w: Math.max(120, W - 58 - 20), h: Math.max(80, H - 18 - 56) });

  const cursor = $derived.by(() => {
    if (!hover || !area) return null;
    const x = view.xMin + ((hover.x - area.x) / area.w) * (view.xMax - view.xMin);
    const y = view.yMin + ((area.y + area.h - hover.y) / area.h) * (view.yMax - view.yMin);
    if (hover.x < area.x || hover.x > area.x + area.w) return null;
    return { x, y };
  });

  function draw(): void {
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    drawPlot(ctx, {
      ...spec,
      view: {
        ...view,
        // Mantiene las etiquetas de ejes relativas cuando cambia el zoom
      },
      area,
      theme,
      cursor: hover ? cursor : null,
    } as PlotSpec);
  }

  function setup(): void {
    if (!wrap || !canvas) return;
    const rect = wrap.getBoundingClientRect();
    W = Math.max(320, rect.width);
    H = height;
    dpr = Math.min(2, window.devicePixelRatio || 1);
    canvas.width = W * dpr;
    canvas.height = H * dpr;
    canvas.style.width = `${W}px`;
    canvas.style.height = `${H}px`;
    lab.plottingCanvas = canvas;
    draw();
  }

  let ro: ResizeObserver | null = null;

  onMount(() => {
    setup();
    ro = new ResizeObserver(() => setup());
    if (wrap) ro.observe(wrap);
    return () => ro?.disconnect();
  });

  $effect(() => {
    // redibuja con cambios de datos, vista, tema o cursor
    spec;
    view;
    hover;
    theme;
    W;
    draw();
  });

  function resetView(): void {
    view = structuredClone(homeView ?? spec.view);
  }

  function onWheel(event: WheelEvent): void {
    event.preventDefault();
    const factor = event.deltaY > 0 ? 1.12 : 1 / 1.12;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const mx = event.clientX - rect.left;
    const anchorX = view.xMin + ((mx - area.x) / area.w) * (view.xMax - view.xMin);
    const clamp = (v: number, min: number, max: number) => Math.min(max, Math.max(min, v));
    const newView: PlotView = {
      xMin: anchorX - (anchorX - view.xMin) * factor,
      xMax: anchorX + (view.xMax - anchorX) * factor,
      yMin: view.yMin,
      yMax: view.yMax,
    };
    // No permitir zoom out más allá de 30× el rango original
    const base = homeView ?? spec.view;
    const span = newView.xMax - newView.xMin;
    if (span > (base.xMax - base.xMin) * 30) {
      newView.xMin = base.xMin;
      newView.xMax = base.xMax;
    }
    view = {
      xMin: clamp(newView.xMin, base.xMin - 1e6, base.xMax),
      xMax: clamp(newView.xMax, base.xMin, base.xMax + 1e6),
      yMin: newView.yMin,
      yMax: newView.yMax,
    };
  }

  async function exportPng(): Promise<void> {
    if (!canvas) return;
    const blob = await canvasToPng(canvas);
    downloadBlob(blob, `${exportName}.png`);
  }

  function exportSvg(): void {
    const svg = plotToSvg({ ...spec, view, area, theme } as PlotSpec, W, H);
    downloadBlob(new Blob([svg], { type: "image/svg+xml" }), `${exportName}.svg`);
  }

  const xToPixel = (x: number) => sScale(x, view.xMin, view.xMax, area.x, area.x + area.w);
</script>

<div class="plot-wrap" bind:this={wrap}>
  <div class="plot-tools">
    <span class="hint small muted" data-testid="plot-hint">Rueda: zoom · Arrastrar: pan</span>
    <div class="spacer"></div>
    <button class="btn ghost icon" onclick={resetView} title="Restablecer vista" aria-label="Restablecer vista">⟲</button>
    <button class="btn ghost icon" onclick={exportPng} title="Exportar PNG" aria-label="Exportar PNG">PNG</button>
    <button class="btn ghost icon" onclick={exportSvg} title="Exportar SVG" aria-label="Exportar SVG">SVG</button>
  </div>
  <div
    class="canvas-box"
    role="application"
    aria-label="Gráfica interactiva"
    onwheel={onWheel}
    onpointerdown={(e) => {
      dragging = { startX: e.clientX, startY: e.clientY, view: { ...view } };
      (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
    }}
    onpointermove={(e) => {
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      hover = { x: e.clientX - rect.left, y: e.clientY - rect.top };
      if (dragging) {
        const dx = e.clientX - dragging.startX;
        const dy = e.clientY - dragging.startY;
        const sx = (dragging.view.xMax - dragging.view.xMin) / area.w;
        const sy = (dragging.view.yMax - dragging.view.yMin) / area.h;
        view = {
          xMin: dragging.view.xMin - dx * sx,
          xMax: dragging.view.xMax - dx * sx,
          yMin: dragging.view.yMin + dy * sy,
          yMax: dragging.view.yMax + dy * sy,
        };
      }
    }}
    onpointerup={() => (dragging = null)}
    onpointerleave={() => {
      hover = null;
      dragging = null;
    }}
  >
    <canvas bind:this={canvas}></canvas>
  </div>
</div>

<style>
  .plot-wrap {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  .plot-tools {
    display: flex;
    align-items: center;
    padding: 0 6px;
    gap: 2px;
  }
  .spacer {
    flex: 1;
  }
  .canvas-box {
    position: relative;
    cursor: crosshair;
    touch-action: none;
    border-radius: var(--r-md);
    border: 1px solid var(--line);
    background: var(--panel-solid);
    overflow: hidden;
  }
  canvas {
    display: block;
  }
  .hint {
    padding-left: 6px;
  }
</style>
