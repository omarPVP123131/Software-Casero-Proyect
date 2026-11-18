<script lang="ts">
  // Canvas principal: fuente de luz, cátodo, ánodo, batería, fotones,
  // electrones y el diagrama de energías. Todo el movimiento se deriva de
  // `lab.animTime` (integrado con dt), nunca del número de frame.
  import { onMount } from "svelte";
  import { lab } from "../lib/store.svelte";
  import { t } from "../lib/i18n";
  import { wavelengthToColor } from "../lib/spectrum";
  import {
    electronsAt,
    electronCount,
    hash01,
    photonsAt,
    sceneGeometry,
    type SceneGeom,
  } from "../lib/scene-model";

  let canvas: HTMLCanvasElement | null = null;
  let wrap: HTMLDivElement | null = null;
  let W = $state(900);
  let H = $state(430);
  let fps = $state(60);
  let hover = $state<{ x: number; y: number } | null>(null);
  let dpr = 1;
  let ro: ResizeObserver | null = null;
  const lang = $derived(lab.preferences.lang);

  const s = $derived(t(lab.preferences.lang));
  const wl = $derived(lab.controls.wavelengthNm);
  const color = $derived(wavelengthToColor(wl));
  const readout = $derived(lab.readout);
  const reduced = $derived(lab.preferences.motion === "reduced");
  const layers = $derived(lab.preferences.layers);
  const voltage = $derived(lab.controls.appliedVoltageV);
  const v0 = $derived(readout?.stoppingPotentialV ?? 0);
  const blocked = $derived(readout?.blocked ?? false);
  const retarding = $derived(
    voltage < 0 && v0 > 0 ? Math.min(1, -voltage / Math.max(v0, 1e-6)) : 0,
  );

  // El texto del canvas no hereda el tamaño de fuente del DOM: aplicamos la
  // preferencia de accesibilidad con un tope para no desbordar la escalera.
  const fscale = $derived(Math.min(1.2, Math.max(0.85, lab.preferences.fontScale)));
  function f(px: number): number {
    return Math.round(px * fscale * 10) / 10;
  }

  function setup(): void {
    if (!canvas || !wrap) return;
    const rect = wrap.getBoundingClientRect();
    W = Math.max(320, rect.width);
    H = Math.max(280, rect.height);
    dpr = Math.min(2, window.devicePixelRatio || 1);
    canvas.width = Math.round(W * dpr);
    canvas.height = Math.round(H * dpr);
    canvas.style.width = `${W}px`;
    canvas.style.height = `${H}px`;
    lab.sceneCanvas = canvas;
  }

  onMount(() => {
    setup();
    ro = new ResizeObserver(() => setup());
    if (wrap) ro.observe(wrap);
    return () => ro?.disconnect();
  });

  // ---------- dibujo ----------
  let lastFrame = 0;
  let frameCount = 0;
  let fpsWindow = 0;
  let raf = 0;

  function css(name: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  }

  function draw(): void {
    raf = requestAnimationFrame(draw);
    const c = canvas;
    if (!c) return;
    const ctx = c.getContext("2d");
    if (!ctx) return;
    const now = performance.now();
    frameCount++;
    if (now - fpsWindow > 500) {
      fps = Math.round((frameCount * 1000) / (now - fpsWindow));
      frameCount = 0;
      fpsWindow = now;
    }
    const t0 = lab.animTime;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, W, H);
    const g = sceneGeometry(W, H);

    // fondo de cámara
    const bg = ctx.createLinearGradient(0, 0, W, H);
    bg.addColorStop(0, css("--canvas-bg-0") || "#05070d");
    bg.addColorStop(1, css("--canvas-bg-1") || "#0a1020");
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, W, H);

    // halo radial tenue del color de la luz detrás del haz
    const halo = ctx.createRadialGradient(
      g.sourceX + 40,
      g.midY,
      10,
      g.sourceX + 40,
      g.midY,
      W * 0.55,
    );
    const [r, gg, b] = color.rgb;
    halo.addColorStop(0, `rgba(${r},${gg},${b},${reduced ? 0.1 : 0.16})`);
    halo.addColorStop(1, "rgba(0,0,0,0)");
    ctx.fillStyle = halo;
    ctx.fillRect(0, 0, W, H);

    if (layers.grid) drawGrid(ctx, g);

    drawBattery(ctx, g);
    drawSource(ctx, g);
    drawElectrodes(ctx, g);
    if (layers.field) drawField(ctx, g);
    drawBeam(ctx, g, t0);
    if (layers.trajectories) drawElectrons(ctx, g, t0);
    if (layers.energyLadder) drawLadder(ctx, g);
    if (layers.currentMeter) drawMeter(ctx, g);
    drawOverlayText(ctx, g);
  }

  function drawGrid(ctx: CanvasRenderingContext2D, g: SceneGeom): void {
    ctx.save();
    ctx.strokeStyle = "rgba(148,163,184,0.06)";
    ctx.lineWidth = 1;
    const step = 40;
    for (let x = 0; x < g.w; x += step) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, g.h);
      ctx.stroke();
    }
    for (let y = 0; y < g.h; y += step) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(g.w, y);
      ctx.stroke();
    }
    ctx.restore();
  }

  function drawSource(ctx: CanvasRenderingContext2D, g: SceneGeom): void {
    const x = g.sourceX;
    const y = g.midY;
    const [r, gg, b] = color.rgb;
    ctx.save();
    // cuerpo de la fuente
    ctx.fillStyle = css("--panel-solid") || "#0f1526";
    ctx.strokeStyle = css("--line-strong") || "rgba(255,255,255,0.14)";
    ctx.lineWidth = 1.2;
    roundRect(ctx, x - 26, y - 40, 52, 80, 12);
    ctx.fill();
    ctx.stroke();
    // emisor brillante
    const grad = ctx.createRadialGradient(x, y, 2, x, y, 24);
    grad.addColorStop(0, `rgba(${r},${gg},${b},1)`);
    grad.addColorStop(0.4, `rgba(${r},${gg},${b},0.5)`);
    grad.addColorStop(1, `rgba(${r},${gg},${b},0)`);
    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.arc(x, y, 24, 0, Math.PI * 2);
    ctx.fill();
    ctx.fillStyle = color.visible ? "rgba(255,255,255,0.9)" : `rgb(${r},${gg},${b})`;
    ctx.beginPath();
    ctx.arc(x, y, 5.5, 0, Math.PI * 2);
    ctx.fill();
    // etiqueta λ
    ctx.font = `500 ${f(11)}px JetBrains Mono, monospace`;
    ctx.fillStyle = css("--muted") || "#7c8aa5";
    ctx.textAlign = "center";
    ctx.fillText(`${wl.toFixed(0)} nm`, x, y + 60);
    if (!color.visible) {
      ctx.fillStyle = css("--warn") || "#f59e0b";
      ctx.fillText(s.noVisible.toUpperCase(), x, y + 74);
    }
    ctx.restore();
  }

  function drawElectrodes(ctx: CanvasRenderingContext2D, g: SceneGeom): void {
    const metal = css("--metal") || "#8a94a8";
    const cath = lang === "en" ? "CATHODE" : "CÁTODO";
    const anod = lang === "en" ? "ANODE" : "ÁNODO";
    for (const [x, label] of [
      [g.cathodeX, cath],
      [g.anodeX, anod],
    ] as const) {
      const grad = ctx.createLinearGradient(x - 9, 0, x + 9, 0);
      grad.addColorStop(0, "rgba(255,255,255,0.28)");
      grad.addColorStop(0.5, metal);
      grad.addColorStop(1, "rgba(0,0,0,0.45)");
      ctx.fillStyle = grad;
      roundRect(ctx, x - 9, g.plateTop, 18, g.plateBottom - g.plateTop, 4);
      ctx.fill();
      ctx.strokeStyle = "rgba(255,255,255,0.10)";
      ctx.stroke();
      // textura metálica rayada sutil
      ctx.save();
      ctx.globalAlpha = 0.12;
      ctx.strokeStyle = "#0b0f18";
      for (let y = g.plateTop + 6; y < g.plateBottom - 4; y += 10) {
        ctx.beginPath();
        ctx.moveTo(x - 7, y);
        ctx.lineTo(x + 7, y + 3);
        ctx.stroke();
      }
      ctx.restore();
      ctx.font = `600 ${f(10)}px Inter, sans-serif`;
      ctx.fillStyle = css("--muted") || "#7c8aa5";
      ctx.textAlign = "center";
      ctx.fillText(label, x, g.plateTop - 10);
    }
  }

  function drawBeam(ctx: CanvasRenderingContext2D, g: SceneGeom, time: number): void {
    const intensity = lab.controls.intensityPercent;
    const beamTop = g.midY - 16;
    const beamH = 32;
    const photons = photonsAt(time, intensity);
    const [r, gg, b] = color.rgb;
    ctx.save();
    // haz continuo (brillo ∝ intensidad)
    const beamGrad = ctx.createLinearGradient(g.sourceX, 0, g.cathodeX, 0);
    beamGrad.addColorStop(0, `rgba(${r},${gg},${b},${0.05 + 0.12 * (intensity / 100)})`);
    beamGrad.addColorStop(1, `rgba(${r},${gg},${b},${0.10 + 0.18 * (intensity / 100)})`);
    ctx.fillStyle = beamGrad;
    ctx.fillRect(g.sourceX, beamTop, g.cathodeX - g.sourceX, beamH);

    // paquetes de onda
    for (const ph of photons) {
      const x = g.sourceX + ph.phase * (g.cathodeX - g.sourceX);
      const y = g.midY + ph.yOff * (beamH / 2);
      const lambdaPx = 6 + Math.min(30, wl / 22);
      const alpha = 0.35 + 0.65 * (intensity / 100);
      ctx.strokeStyle = `rgba(${r},${gg},${b},${alpha})`;
      ctx.lineWidth = 1.8;
      ctx.beginPath();
      if (color.visible) {
        for (let k = -1; k <= 1; k += 0.5) {
          const xx = x + k * lambdaPx;
          const yy = y + Math.sin((k / 1.5) * Math.PI * 2) * 4.2;
          if (k === -1) ctx.moveTo(xx, yy);
          else ctx.lineTo(xx, yy);
        }
      } else {
        // UV/IR: paquete rayado, sin onda visible
        ctx.moveTo(x - lambdaPx / 2, y - 4);
        ctx.lineTo(x + lambdaPx / 2, y + 4);
      }
      ctx.stroke();
      if (!reduced) {
        ctx.fillStyle = `rgba(${r},${gg},${b},0.16)`;
        ctx.beginPath();
        ctx.arc(x, y, lambdaPx * 0.8, 0, Math.PI * 2);
        ctx.fill();
      }
    }

    // interacción en el cátodo
    const emits = readout?.emits ?? false;
    if (!emits && lab.controls.intensityPercent > 0.5) {
      // absorción/rebote con destello lento
      const pulse = 0.5 + 0.5 * Math.sin(time * 3);
      ctx.strokeStyle = `rgba(${r},${gg},${b},${0.25 + 0.35 * pulse})`;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(g.cathodeX, g.midY, 16 + pulse * 6, Math.PI * 0.5, Math.PI * 1.5);
      ctx.stroke();
    } else if (emits && !blocked) {
      const flash = 0.35 + 0.65 * hash01(Math.floor(time * 14));
      ctx.fillStyle = `rgba(${r},${gg},${b},${0.25 * flash})`;
      ctx.beginPath();
      ctx.arc(g.cathodeX, g.midY, 22, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();
  }

  function drawElectrons(ctx: CanvasRenderingContext2D, g: SceneGeom, time: number): void {
    const r0 = readout;
    if (!r0 || !r0.emits) return;
    const count = electronCount(
      lab.controls.intensityPercent,
      r0.collectionFactor,
      22,
    );
    if (count <= 0) return;
    const scale = r0.speedScale;
    const electrons = electronsAt(time, count, scale, retarding);
    const [r, gg, b] = color.rgb;
    ctx.save();
    for (const e of electrons) {
      const laneY = g.midY + e.lane * 90 + e.lane * 20;
      const x = g.cathodeX + 12 + e.progress * (g.anodeX - g.cathodeX - 24);
      const y = laneY + Math.sin((x / g.w) * 6 + e.seed * 9) * 8;
      // estela
      if (!reduced) {
        const trail = ctx.createLinearGradient(x - 22, y, x, y);
        trail.addColorStop(0, "rgba(103,232,249,0)");
        trail.addColorStop(1, "rgba(103,232,249,0.5)");
        ctx.strokeStyle = trail;
        ctx.lineWidth = 2.4;
        ctx.beginPath();
        ctx.moveTo(x - 22, y);
        ctx.lineTo(x, y);
        ctx.stroke();
      }
      ctx.fillStyle = "#67E8F9";
      ctx.beginPath();
      ctx.arc(x, y, 3 + e.energy * 1.6, 0, Math.PI * 2);
      ctx.fill();
      if (!reduced) {
        ctx.fillStyle = "rgba(103,232,249,0.18)";
        ctx.beginPath();
        ctx.arc(x, y, 9, 0, Math.PI * 2);
        ctx.fill();
      }
    }
    ctx.restore();
  }

  function drawField(ctx: CanvasRenderingContext2D, g: SceneGeom): void {
    const v = voltage;
    const strength = Math.min(1, Math.abs(v) / 5);
    if (strength < 0.03) return;
    const dir = v < 0 ? -1 : 1; // campo apunta hacia donde empuja la carga
    ctx.save();
    ctx.strokeStyle =
      dir > 0 ? "rgba(96,165,250,0.28)" : "rgba(245,158,11,0.30)";
    ctx.fillStyle = ctx.strokeStyle;
    ctx.lineWidth = 1.1;
    const rows = 7;
    const cols = 6;
    for (let i = 0; i < rows; i++) {
      const y = g.plateTop + ((i + 0.5) / rows) * (g.plateBottom - g.plateTop);
      for (let j = 0; j < cols; j++) {
        const x =
          g.cathodeX + 26 + (j / (cols - 1)) * (g.anodeX - g.cathodeX - 52);
        const len = 8 + 10 * strength;
        ctx.beginPath();
        ctx.moveTo(x, y);
        ctx.lineTo(x + dir * len, y);
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(x + dir * len, y);
        ctx.lineTo(x + dir * (len - 4), y - 3);
        ctx.lineTo(x + dir * (len - 4), y + 3);
        ctx.closePath();
        ctx.fill();
      }
    }
    ctx.font = `500 ${f(10)}px Inter, sans-serif`;
    ctx.fillStyle = dir > 0 ? "rgba(96,165,250,0.8)" : "rgba(245,158,11,0.9)";
    ctx.textAlign = "left";
    ctx.fillText(
      dir > 0 ? `E ↓ aceleración (V = ${v.toFixed(2)} V)` : `E ↑ frenado (V = ${v.toFixed(2)} V)`,
      g.cathodeX + 26,
      g.plateBottom + 16,
    );
    ctx.restore();
  }

  function drawBattery(ctx: CanvasRenderingContext2D, g: SceneGeom): void {
    const midX = (g.cathodeX + g.anodeX) / 2;
    const y = g.batteryY;
    ctx.save();
    ctx.strokeStyle = "rgba(148,163,184,0.35)";
    ctx.lineWidth = 1.4;
    ctx.beginPath();
    ctx.moveTo(g.cathodeX, g.plateBottom);
    ctx.lineTo(g.cathodeX, y);
    ctx.lineTo(midX - 34, y);
    ctx.moveTo(g.anodeX, g.plateBottom);
    ctx.lineTo(g.anodeX, y);
    ctx.lineTo(midX + 34, y);
    ctx.stroke();
    // cuerpo batería
    const v = voltage;
    ctx.fillStyle = css("--panel-solid") || "#0f1526";
    ctx.strokeStyle = css("--line-strong") || "rgba(255,255,255,0.14)";
    roundRect(ctx, midX - 34, y - 14, 68, 28, 8);
    ctx.fill();
    ctx.stroke();
    ctx.font = `600 ${f(12)}px JetBrains Mono, monospace`;
    ctx.fillStyle = v === 0 ? css("--muted") : v > 0 ? "#60a5fa" : "#f59e0b";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(`${v > 0 ? "+" : ""}${v.toFixed(2)} V`, midX, y);
    // polaridad animada
    const phase = (lab.animTime * 40) % 40;
    ctx.fillStyle = v > 0 ? "#60a5fa" : v < 0 ? "#f59e0b" : "rgba(148,163,184,0.4)";
    for (const dir of [-1, 1]) {
      const x = midX + dir * (40 + (phase % 40));
      ctx.beginPath();
      ctx.arc(x, y, 2.4, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();
  }

  // Diagrama de energías: barra apilada Φ (gastado) + Kmax (sobrante) = E.
  // Si no hay emisión, E queda corta antes de Φ y se marca el déficit.
  function drawLadder(ctx: CanvasRenderingContext2D, g: SceneGeom): void {
    const r0 = readout;
    if (!r0) return;
    // Se coloca en el hueco libre a la izquierda del cátodo: si el lienzo es
    // estrecho (1280×800) el panel se encimaba con la placa y su etiqueta.
    const w = Math.max(168, Math.min(208, g.cathodeX - 64));
    const x0 = 14;
    const y0 = 54;
    const h = 120;
    const pad = 14;
    const axisX = x0 + pad;
    const axisW = w - pad * 2;
    const maxEv = Math.max(7.2, Math.ceil(r0.photonEnergyEv) + 0.5, Math.ceil(r0.workFunctionEv) + 0.3);
    const px = (ev: number) => axisX + (Math.min(ev, maxEv) / maxEv) * axisW;
    const barY = y0 + 46;
    const barH = 13;
    const isDark = document.documentElement.dataset.theme !== "light";
    ctx.save();
    // panel de vidrio
    ctx.fillStyle = isDark ? "rgba(9,13,24,0.72)" : "rgba(255,255,255,0.86)";
    ctx.strokeStyle = css("--line") || "rgba(255,255,255,0.08)";
    ctx.lineWidth = 1;
    roundRect(ctx, x0, y0, w, h, 10);
    ctx.fill();
    ctx.stroke();

    ctx.font = `500 ${f(9.5)}px JetBrains Mono, monospace`;
    ctx.textBaseline = "middle";
    ctx.textAlign = "left";
    ctx.fillStyle = css("--muted") || "#7c8aa5";
    ctx.fillText(lang === "en" ? "energies (eV)" : "energías (eV)", axisX, y0 + 16);

    // eje 0..maxEv
    const tickStep = maxEv > 6 ? 2 : 1;
    ctx.strokeStyle = css("--line-strong") || "rgba(255,255,255,0.14)";
    ctx.fillStyle = css("--muted") || "#7c8aa5";
    ctx.textAlign = "center";
    for (let ev = 0; ev <= maxEv; ev += tickStep) {
      const x = px(ev);
      ctx.beginPath();
      ctx.moveTo(x, barY + barH + 3);
      ctx.lineTo(x, barY + barH + 6);
      ctx.stroke();
      ctx.fillText(String(ev), x, barY + barH + 14);
    }

    // Φ gastado + Kmax sobrante (= E total)
    const phiX = px(r0.workFunctionEv);
    const eX = px(r0.photonEnergyEv);
    ctx.fillStyle = "rgba(245,158,11,0.55)";
    roundRect(ctx, axisX, barY, Math.max(0, phiX - axisX), barH, 4);
    ctx.fill();
    if (r0.emits) {
      ctx.fillStyle = "rgba(103,232,249,0.9)";
      roundRect(ctx, phiX, barY, Math.max(1, eX - phiX), barH, 4);
      ctx.fill();
    } else {
      // E no alcanza Φ: barra hasta E + hueco rayado hasta Φ
      ctx.fillStyle = `rgba(${color.rgb[0]},${color.rgb[1]},${color.rgb[2]},0.55)`;
      roundRect(ctx, axisX, barY, Math.max(0, eX - axisX), barH, 4);
      ctx.fill();
      ctx.save();
      ctx.strokeStyle = css("--warn") || "#f59e0b";
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.moveTo(eX, barY - 4);
      ctx.lineTo(phiX, barY - 4);
      ctx.stroke();
      ctx.restore();
    }

    // marca Φ
    ctx.strokeStyle = css("--warn") || "#f59e0b";
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(phiX, barY - 10);
    ctx.lineTo(phiX, barY + barH + 4);
    ctx.stroke();
    ctx.setLineDash([]);
    ctx.fillStyle = css("--warn") || "#f59e0b";
    ctx.textAlign = "left";
    ctx.fillText(`Φ ${r0.workFunctionEv.toFixed(2)}`, axisX, barY - 14);

    // marca E
    ctx.fillStyle = `rgb(${color.rgb[0]},${color.rgb[1]},${color.rgb[2]})`;
    ctx.beginPath();
    ctx.arc(eX, barY - 10, 3, 0, Math.PI * 2);
    ctx.fill();
    ctx.textAlign = "right";
    ctx.fillText(`E ${r0.photonEnergyEv.toFixed(2)}`, axisX + axisW, barY - 14);

    // resultado
    ctx.font = `600 ${f(10)}px Inter, sans-serif`;
    ctx.textAlign = "left";
    ctx.fillStyle = r0.emits ? "#67E8F9" : css("--warn") || "#f59e0b";
    ctx.fillText(
      r0.emits
        ? `Kmax = E − Φ = ${r0.kMaxEv.toFixed(2)} eV`
        : `faltan ${r0.deficitEv.toFixed(2)} eV para Φ`,
      axisX,
      y0 + h - 12,
    );
    ctx.restore();
  }

  function drawMeter(ctx: CanvasRenderingContext2D, g: SceneGeom): void {
    const r0 = readout;
    if (!r0) return;
    const x = g.w - 58;
    const top = g.plateTop;
    const h = g.plateBottom - g.plateTop;
    const frac = Math.min(1, Math.max(0, r0.collectionFactor * (r0.photocurrentA > 0 ? 1 : 0)));
    ctx.save();
    ctx.strokeStyle = css("--line-strong") || "rgba(255,255,255,0.14)";
    ctx.strokeRect(x, top, 8, h);
    ctx.fillStyle = "rgba(103,232,249,0.75)";
    const fillH = h * frac;
    ctx.fillRect(x + 1, top + h - fillH, 6, fillH);
    ctx.font = `500 ${f(9.5)}px Inter, sans-serif`;
    ctx.fillStyle = css("--muted") || "#7c8aa5";
    ctx.textAlign = "center";
    ctx.fillText("I", x + 4, top - 8);
    ctx.fillText("0", x + 4, g.plateBottom + 12);
    ctx.restore();
  }

  function drawOverlayText(ctx: CanvasRenderingContext2D, g: SceneGeom): void {
    const r0 = readout;
    if (!r0) return;
    const isDark = document.documentElement.dataset.theme !== "light";
    const boxW = 268;
    const boxX = g.w / 2 - boxW / 2;
    const boxY = 14;
    ctx.save();
    ctx.fillStyle = isDark ? "rgba(9,13,24,0.72)" : "rgba(255,255,255,0.85)";
    ctx.strokeStyle = css("--line") || "rgba(255,255,255,0.08)";
    roundRect(ctx, boxX, boxY, boxW, 34, 10);
    ctx.fill();
    ctx.stroke();
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    if (!r0.emits) {
      ctx.fillStyle = css("--text") || "#e8edf7";
      ctx.font = `600 ${f(12)}px Inter, sans-serif`;
      ctx.fillText(
        `E = ${r0.photonEnergyEv.toFixed(2)} eV  <  Φ = ${r0.workFunctionEv.toFixed(2)} eV`,
        g.w / 2,
        boxY + 13,
      );
      ctx.fillStyle = css("--muted") || "#7c8aa5";
      ctx.font = `500 ${f(10.5)}px Inter, sans-serif`;
      ctx.fillText(
        `faltan ${r0.deficitEv.toFixed(2)} eV · λ₀ = ${r0.thresholdWavelengthNm.toFixed(0)} nm`,
        g.w / 2,
        boxY + 26,
      );
    } else if (r0.blocked) {
      ctx.fillStyle = css("--warn") || "#f59e0b";
      ctx.font = `600 ${f(12)}px Inter, sans-serif`;
      ctx.fillText(`BLOQUEADO: V = ${voltage.toFixed(2)} V < −V₀ = ${(-v0).toFixed(2)} V`, g.w / 2, boxY + 13);
      ctx.fillStyle = css("--muted") || "#7c8aa5";
      ctx.font = `500 ${f(10.5)}px Inter, sans-serif`;
      ctx.fillText(`Kmax = ${r0.kMaxEv.toFixed(2)} eV no cambia con V`, g.w / 2, boxY + 26);
    } else {
      ctx.fillStyle = css("--text") || "#e8edf7";
      ctx.font = `600 ${f(12)}px Inter, sans-serif`;
      ctx.fillText(`Kmax = ${r0.kMaxEv.toFixed(2)} eV · V₀ = ${r0.stoppingPotentialV.toFixed(2)} V`, g.w / 2, boxY + 13);
      ctx.fillStyle = css("--muted") || "#7c8aa5";
      ctx.font = `500 ${f(10.5)}px Inter, sans-serif`;
      ctx.fillText(
        lang === "en"
          ? `v = ${(r0.electronMaxSpeedMS / 1e6).toFixed(2)}×10⁶ m/s · scale ${r0.speedScale.toFixed(2)}×`
          : `v = ${(r0.electronMaxSpeedMS / 1e6).toFixed(2)}×10⁶ m/s · escala ${r0.speedScale.toFixed(2)}×`,
        g.w / 2,
        boxY + 26,
      );
    }
    // leyenda de escala de velocidad (siempre visible: requisito didáctico)
    ctx.textAlign = "left";
    ctx.font = `500 ${f(9.5)}px Inter, sans-serif`;
    ctx.fillStyle = css("--muted") || "#7c8aa5";
    ctx.fillText(
      lab.preferences.speedScale === "compressed"
        ? lang === "en"
          ? `compressed speed scale √(K/1 eV), bounded 0.5–2.2 · legend`
          : `escala de velocidad comprimida √(K/1 eV) acotada 0.5–2.2 · leyenda`
        : lang === "en"
          ? `real logarithmic scale (v in m/s) · legend`
          : `escala real logarítmica (v m/s) · leyenda`,
      12,
      g.h - 10,
    );
    ctx.restore();
  }

  function roundRect(
    ctx: CanvasRenderingContext2D,
    x: number,
    y: number,
    w: number,
    h: number,
    r: number,
  ): void {
    const rr = Math.min(r, w / 2, h / 2);
    ctx.beginPath();
    ctx.moveTo(x + rr, y);
    ctx.arcTo(x + w, y, x + w, y + h, rr);
    ctx.arcTo(x + w, y + h, x, y + h, rr);
    ctx.arcTo(x, y + h, x, y, rr);
    ctx.arcTo(x, y, x + w, y, rr);
    ctx.closePath();
  }

  // --- Tooltips didácticos -----------------------------------------------
  // Cada zona del dibujo explica su magnitud en una frase; la escalera de
  // energías obtiene una lectura en vivo de Φ, E y Kmax.
  type Tip = { title: string; body: string };

  const ladderBox = $derived.by(() => {
    const g = sceneGeometry(W, H);
    return { x0: 14, y0: 54, w: Math.max(168, Math.min(208, g.cathodeX - 64)), h: 120 };
  });

  const overLadder = $derived(
    !!hover && hover.x >= ladderBox.x0 && hover.x <= ladderBox.x0 + ladderBox.w && hover.y >= ladderBox.y0 && hover.y <= ladderBox.y0 + ladderBox.h,
  );

  const tipPos = $derived.by(() => {
    if (!hover) return { left: 0, top: 0 };
    // Sobre la escalera el tooltip va a su derecha: encima taparía justo lo
    // que está explicando.
    if (overLadder) return { left: ladderBox.x0 + ladderBox.w + 14, top: Math.max(10, hover.y - 30) };
    return {
      left: Math.round(Math.min(Math.max(hover.x + 16, 8), Math.max(8, W - 306))),
      top: Math.round(hover.y > 150 ? hover.y - 84 : hover.y + 22),
    };
  });

  const tip = $derived.by((): Tip | null => {
    const r0 = readout;
    if (!hover || !r0) return null;
    const es = lang !== "en";
    const L = ladderBox;
    const g = sceneGeometry(W, H);
    if (hover.x >= L.x0 && hover.x <= L.x0 + L.w && hover.y >= L.y0 && hover.y <= L.y0 + L.h) {
      // Distinguimos las tres franjas del panel: barra (Φ/E), escala numerada y resultado.
      const barY = L.y0 + 46;
      if (hover.y >= barY - 20 && hover.y <= barY + 20) {
        return es
          ? {
              title: "Escalera de energías",
              body: r0.emits
                ? `El fotón aporta E = ${r0.photonEnergyEv.toFixed(2)} eV. Gasta Φ = ${r0.workFunctionEv.toFixed(2)} eV en arrancar el electrón; el resto sale como Kmax = ${r0.kMaxEv.toFixed(2)} eV.`
                : `El fotón aporta solo E = ${r0.photonEnergyEv.toFixed(2)} eV y el material pide Φ = ${r0.workFunctionEv.toFixed(2)} eV: faltan ${r0.deficitEv.toFixed(2)} eV, no hay emisión.`,
            }
          : {
              title: "Energy ladder",
              body: r0.emits
                ? `The photon brings E = ${r0.photonEnergyEv.toFixed(2)} eV. Φ = ${r0.workFunctionEv.toFixed(2)} eV frees the electron; the rest leaves as Kmax = ${r0.kMaxEv.toFixed(2)} eV.`
                : `The photon brings only E = ${r0.photonEnergyEv.toFixed(2)} eV but Φ = ${r0.workFunctionEv.toFixed(2)} eV is needed: ${r0.deficitEv.toFixed(2)} eV short, no emission.`,
            };
      }
      return es
        ? { title: "Escala en eV", body: `De 0 a ${Math.max(7.2, Math.ceil(r0.photonEnergyEv) + 0.5).toFixed(1)} eV. La marca ámbar es Φ y el punto de color, la energía del fotón.` }
        : { title: "Scale in eV", body: `From 0 to ${Math.max(7.2, Math.ceil(r0.photonEnergyEv) + 0.5).toFixed(1)} eV. The amber mark is Φ; the coloured dot is the photon energy.` };
    }
    if (hover.x < g.cathodeX - 24) {
      return es
        ? { title: `Luz incidente · ${wl.toFixed(0)} nm`, body: `Cada fotón lleva E = ${r0.photonEnergyEv.toFixed(2)} eV (E = hc/λ). Intensidad ${lab.controls.intensityPercent.toFixed(0)} %: cambia cuántos fotones, no su energía.` }
        : { title: `Incident light · ${wl.toFixed(0)} nm`, body: `Each photon carries E = ${r0.photonEnergyEv.toFixed(2)} eV (E = hc/λ). Intensity ${lab.controls.intensityPercent.toFixed(0)} %: changes how many photons, not their energy.` };
    }
    if (Math.abs(hover.x - g.cathodeX) < 26) {
      return es
        ? { title: `Cátodo · Φ = ${r0.workFunctionEv.toFixed(2)} eV`, body: `Umbral λ₀ = ${(1240 / Math.max(r0.workFunctionEv, 0.01)).toFixed(0)} nm. Con λ mayor que λ₀ ningún fotón arranca electrones, por intensa que sea la luz.` }
        : { title: `Cathode · Φ = ${r0.workFunctionEv.toFixed(2)} eV`, body: `Threshold λ₀ = ${(1240 / Math.max(r0.workFunctionEv, 0.01)).toFixed(0)} nm. Above λ₀ no photon frees an electron, however bright the light.` };
    }
    if (Math.abs(hover.x - g.anodeX) < 26) {
      return es
        ? { title: `Ánodo · I = ${(r0.photocurrentA * 1e6).toFixed(2)} µA`, body: `Corriente medida con V = ${lab.controls.appliedVoltageV.toFixed(2)} V. Colección ${(r0.collectionFactor * 100).toFixed(0)} %: si V < −V₀ no llega ningún electrón.` }
        : { title: `Anode · I = ${(r0.photocurrentA * 1e6).toFixed(2)} µA`, body: `Current at V = ${lab.controls.appliedVoltageV.toFixed(2)} V. Collection ${(r0.collectionFactor * 100).toFixed(0)} %: if V < −V₀ none arrive.` };
    }
    if (hover.y > g.plateBottom + 8) {
      return es
        ? { title: `Circuito · V = ${lab.controls.appliedVoltageV.toFixed(2)} V`, body: `La batería fija la diferencia cátodo–ánodo. Negativa frena a los electrones con Kmax ≤ e·V₀; a −V₀ la corriente se corta.` }
        : { title: `Circuit · V = ${lab.controls.appliedVoltageV.toFixed(2)} V`, body: `The battery sets the cathode–anode voltage. Reversed it brakes electrons while Kmax ≤ e·V₀; at −V₀ the current stops.` };
    }
    return es
      ? { title: "Electrones emitidos", body: `Kmax = ${r0.kMaxEv.toFixed(2)} eV → v = ${(r0.electronMaxSpeedMS / 1e6).toFixed(2)}×10⁶ m/s. El tamaño del punto es la energía, no la intensidad.` }
      : { title: "Emitted electrons", body: `Kmax = ${r0.kMaxEv.toFixed(2)} eV → v = ${(r0.electronMaxSpeedMS / 1e6).toFixed(2)}×10⁶ m/s. Dot size is energy, not intensity.` };
  });

  onMount(() => {
    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  });

  // --- Navegación por teclado de las zonas explicativas -------------------
  // Las flechas recorren las mismas zonas que el puntero, de modo que la
  // explicación no depende de poder mover el ratón.
  let kbdZone = $state<number | null>(null);
  const zones = $derived.by(() => {
    const g = sceneGeometry(W, H);
    const L = ladderBox;
    return [
      { x: g.sourceX, y: g.midY },
      { x: L.x0 + L.w / 2, y: L.y0 + 46 },
      { x: g.cathodeX, y: g.midY },
      { x: g.anodeX, y: g.midY },
      { x: (g.cathodeX + g.anodeX) / 2, y: g.batteryY },
      { x: (g.cathodeX + g.anodeX) / 2, y: g.midY },
    ];
  });

  function onSceneKey(event: KeyboardEvent): void {
    if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
      event.preventDefault();
      const dir = event.key === "ArrowRight" ? 1 : -1;
      const start = kbdZone ?? (dir > 0 ? -1 : 0);
      kbdZone = (start + dir + zones.length) % zones.length;
      hover = zones[kbdZone];
    } else if (event.key === "Escape") {
      kbdZone = null;
      hover = null;
    }
  }

  const stateWord = $derived(
    lab.emissionState === "emit" ? s.emits : lab.emissionState === "blocked" ? s.blocked : s.noEmit,
  );
  const sceneAria = $derived(
    lang === "en"
      ? `Photoelectric tube scene. ${lab.titleFor(lab.controls.material)}, ${wl.toFixed(0)} nm, ${voltage.toFixed(2)} V. ${stateWord}. Use the left and right arrows to explore the regions; Escape closes the explanation.`
      : `Escena del tubo fotoeléctrico. ${lab.titleFor(lab.controls.material)}, ${wl.toFixed(0)} nm, ${voltage.toFixed(2)} V. ${stateWord}. Usa las flechas izquierda y derecha para recorrer las zonas; Escape cierra la explicación.`,
  );

  // Exponer fps para la barra de estado (vía store ligero).
  $effect(() => {
    (window as unknown as { __photolabFps?: number }).__photolabFps = fps;
  });
</script>

<!-- La escena es un gráfico interactivo: con tabindex y flechas se puede
     recorrer con teclado, aunque su rol accesible siga siendo "img". -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="scene-wrap"
  bind:this={wrap}
  onpointermove={(e) => {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    hover = { x: e.clientX - rect.left, y: e.clientY - rect.top };
  }}
  onpointerleave={() => (hover = null)}
  onkeydown={onSceneKey}
  onblur={() => {
    if (kbdZone !== null) {
      kbdZone = null;
      hover = null;
    }
  }}
  tabindex="0"
  role="img"
  aria-label={sceneAria}
>
  <canvas bind:this={canvas}></canvas>
  <div class="kbd-hint small" aria-hidden="true">
    <span class="kbd">←</span><span class="kbd">→</span> {lang === "en" ? "explore" : "explorar"}
  </div>
  {#if hover && tip}
    <div
      class="readout-float tip"
      style:left="{tipPos.left}px"
      style:top="{tipPos.top}px"
      role="status"
    >
      <span class="tip-title">{tip.title}</span>
      <span class="tip-body">{tip.body}</span>
    </div>
  {/if}
  {#if !lab.animating}
    <div class="paused-badge badge warn">{s.paused}</div>
  {/if}
  {#if !color.visible}
    <div class="uv-badge badge warn invisible-hatch">{color.band.toUpperCase()} · {s.noVisible}</div>
  {/if}
</div>

<style>
  .scene-wrap {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 280px;
    border-radius: var(--r-lg);
    overflow: hidden;
    border: 1px solid var(--line);
    background: var(--canvas-bg-0);
  }
  .scene-wrap:focus-visible {
    outline: 2px solid var(--wl);
    outline-offset: -2px;
  }
  .kbd-hint {
    position: absolute;
    top: 10px;
    right: 12px;
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 2px 8px;
    border-radius: var(--r-full);
    background: color-mix(in srgb, var(--panel-solid) 82%, transparent);
    border: 1px solid var(--line);
    color: var(--muted);
    opacity: 0;
    transition: opacity var(--dur-1) var(--ease);
    pointer-events: none;
  }
  .scene-wrap:focus-visible .kbd-hint {
    opacity: 1;
  }

  canvas {
    display: block;
  }

  .readout-float.tip {
    max-width: 268px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 8px 10px;
    line-height: 1.35;
    box-shadow: 0 10px 26px rgba(0, 0, 0, 0.35);
  }
  .tip-title {
    font-weight: 650;
    font-size: 0.78em;
    color: var(--text);
    font-family: Inter, system-ui, sans-serif;
  }
  .tip-body {
    font-size: 0.74em;
    color: var(--muted);
    font-family: Inter, system-ui, sans-serif;
  }

  .readout-float {
    position: absolute;
    pointer-events: none;
    background: var(--panel-solid);
    border: 1px solid var(--line-strong);
    border-radius: var(--r-sm);
    padding: 4px 8px;
    font-size: 0.74em;
    color: var(--text-2);
    box-shadow: var(--el-2);
  }

  .paused-badge {
    position: absolute;
    top: 14px;
    right: 14px;
  }

  .uv-badge {
    position: absolute;
    bottom: 14px;
    right: 14px;
  }
</style>
