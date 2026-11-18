// Utilidades de gráficas: escalas, ejes con unidades, cursor con lectura,
// zoom/pan por arrastre y export a PNG/SVG. Se usan en Canvas 2D con dpr.

export interface PlotArea {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface PlotView {
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
}

export interface PlotPoint {
  x: number;
  y: number;
}

export interface SeriesSpec {
  points: PlotPoint[];
  color: string;
  width?: number;
  dash?: number[];
  label?: string;
  kind?: "line" | "scatter";
  errorY?: number[];
}

export interface PlotSpec {
  area: PlotArea;
  view: PlotView;
  series: SeriesSpec[];
  xLabel: string;
  yLabel: string;
  xTicks?: number;
  yTicks?: number;
  title?: string;
  zeroLine?: boolean;
  marker?: { x: number; color: string; label: string };
  cursor?: PlotPoint | null;
  theme: {
    text: string;
    muted: string;
    line: string;
    grid: string;
    panel: string;
  };
  fontFamily?: string;
}

export const NICE_STEPS = [1, 2, 2.5, 5, 10];

export function niceTicks(min: number, max: number, target = 5): number[] {
  if (!Number.isFinite(min) || !Number.isFinite(max) || min === max) {
    return [min];
  }
  const span = Math.abs(max - min);
  const rough = span / Math.max(1, target);
  const pow = 10 ** Math.floor(Math.log10(rough));
  let step = pow;
  for (const s of NICE_STEPS) {
    if (pow * s >= rough) {
      step = pow * s;
      break;
    }
  }
  const start = Math.ceil(min / step) * step;
  const ticks: number[] = [];
  for (let v = start; v <= max + step * 1e-6; v += step) {
    ticks.push(Number(v.toFixed(10)));
  }
  return ticks;
}

export function sScale(value: number, min: number, max: number, a: number, b: number): number {
  if (max === min) return (a + b) / 2;
  return a + ((value - min) / (max - min)) * (b - a);
}

export function pathForPolyline(
  ctx: CanvasRenderingContext2D,
  area: PlotArea,
  view: PlotView,
  points: PlotPoint[],
): void {
  let started = false;
  for (const p of points) {
    if (!Number.isFinite(p.x) || !Number.isFinite(p.y)) continue;
    const x = sScale(p.x, view.xMin, view.xMax, area.x, area.x + area.w);
    const y = sScale(p.y, view.yMin, view.yMax, area.y + area.h, area.y);
    if (!started) {
      ctx.moveTo(x, y);
      started = true;
    } else {
      ctx.lineTo(x, y);
    }
  }
}

export function drawPlot(ctx: CanvasRenderingContext2D, spec: PlotSpec): void {
  const { area, view, theme } = spec;
  ctx.save();
  ctx.clearRect(0, 0, area.x * 2 + area.w + 40, area.y * 2 + area.h + 40);
  const font = spec.fontFamily ?? "JetBrains Mono, monospace";
  // marco
  ctx.fillStyle = "transparent";
  ctx.strokeStyle = theme.line;
  ctx.lineWidth = 1;
  ctx.strokeRect(area.x, area.y, area.w, area.h);

  // rejilla + ejes
  const xTicks = niceTicks(view.xMin, view.xMax, spec.xTicks ?? 6);
  const yTicks = niceTicks(view.yMin, view.yMax, spec.yTicks ?? 5);
  ctx.font = `10px ${font}`;
  ctx.fillStyle = theme.muted;
  ctx.textAlign = "center";
  ctx.textBaseline = "top";
  for (const tx of xTicks) {
    const x = sScale(tx, view.xMin, view.xMax, area.x, area.x + area.w);
    ctx.strokeStyle = theme.grid;
    ctx.beginPath();
    ctx.moveTo(x, area.y);
    ctx.lineTo(x, area.y + area.h);
    ctx.stroke();
    ctx.fillStyle = theme.muted;
    ctx.fillText(formatTick(tx), x, area.y + area.h + 8);
  }
  ctx.textAlign = "right";
  ctx.textBaseline = "middle";
  for (const ty of yTicks) {
    const y = sScale(ty, view.yMin, view.yMax, area.y + area.h, area.y);
    ctx.strokeStyle = theme.grid;
    ctx.beginPath();
    ctx.moveTo(area.x, y);
    ctx.lineTo(area.x + area.w, y);
    ctx.stroke();
    ctx.fillStyle = theme.muted;
    ctx.fillText(formatTick(ty), area.x - 8, y);
  }

  if (spec.zeroLine) {
    const y = sScale(0, view.yMin, view.yMax, area.y + area.h, area.y);
    if (y > area.y && y < area.y + area.h) {
      ctx.strokeStyle = theme.line;
      ctx.lineWidth = 1.4;
      ctx.beginPath();
      ctx.moveTo(area.x, y);
      ctx.lineTo(area.x + area.w, y);
      ctx.stroke();
    }
  }

  // series
  for (const series of spec.series) {
    if (series.kind === "scatter") {
      ctx.fillStyle = series.color;
      for (let i = 0; i < series.points.length; i++) {
        const p = series.points[i];
        const x = sScale(p.x, view.xMin, view.xMax, area.x, area.x + area.w);
        const y = sScale(p.y, view.yMin, view.yMax, area.y + area.h, area.y);
        if (x < area.x - 10 || x > area.x + area.w + 10) continue;
        const err = series.errorY?.[i];
        if (err && err > 0) {
          const y1 = sScale(p.y - err, view.yMin, view.yMax, area.y + area.h, area.y);
          const y2 = sScale(p.y + err, view.yMin, view.yMax, area.y + area.h, area.y);
          ctx.strokeStyle = series.color;
          ctx.globalAlpha = 0.55;
          ctx.beginPath();
          ctx.moveTo(x, y1);
          ctx.lineTo(x, y2);
          ctx.moveTo(x - 4, y1);
          ctx.lineTo(x + 4, y1);
          ctx.moveTo(x - 4, y2);
          ctx.lineTo(x + 4, y2);
          ctx.stroke();
          ctx.globalAlpha = 1;
        }
        ctx.beginPath();
        ctx.arc(x, y, 3.6, 0, Math.PI * 2);
        ctx.fill();
      }
      continue;
    }
    ctx.strokeStyle = series.color;
    ctx.lineWidth = series.width ?? 2;
    ctx.setLineDash(series.dash ?? []);
    ctx.beginPath();
    pathForPolyline(ctx, area, view, series.points);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  // marcador vertical (p. ej. −V₀)
  if (spec.marker) {
    const x = sScale(spec.marker.x, view.xMin, view.xMax, area.x, area.x + area.w);
    if (x >= area.x - 1 && x <= area.x + area.w + 1) {
      const grad = ctx.createLinearGradient(x, area.y, x, area.y + area.h);
      grad.addColorStop(0, spec.marker.color);
      grad.addColorStop(1, "rgba(0,0,0,0)");
      ctx.strokeStyle = spec.marker.color;
      ctx.setLineDash([5, 5]);
      ctx.lineWidth = 1.3;
      ctx.beginPath();
      ctx.moveTo(x, area.y);
      ctx.lineTo(x, area.y + area.h);
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.fillStyle = spec.marker.color;
      ctx.font = `600 10px ${font}`;
      ctx.textAlign = "left";
      ctx.fillText(spec.marker.label, x + 6, area.y + 12);
    }
  }

  // cursor con lectura
  if (spec.cursor) {
    const x = sScale(spec.cursor.x, view.xMin, view.xMax, area.x, area.x + area.w);
    const y = sScale(spec.cursor.y, view.yMin, view.yMax, area.y + area.h, area.y);
    if (x >= area.x && x <= area.x + area.w) {
      ctx.strokeStyle = theme.text;
      ctx.globalAlpha = 0.35;
      ctx.beginPath();
      ctx.moveTo(x, area.y);
      ctx.lineTo(x, area.y + area.h);
      ctx.stroke();
      ctx.globalAlpha = 1;
      ctx.fillStyle = theme.text;
      ctx.beginPath();
      ctx.arc(x, Math.min(area.y + area.h, Math.max(area.y, y)), 4, 0, Math.PI * 2);
      ctx.fill();
      const label = `(${formatTick(spec.cursor.x, 3)}, ${formatTick(spec.cursor.y, 3)})`;
      ctx.font = `600 10px ${font}`;
      const tw = ctx.measureText(label).width + 12;
      const bx = Math.min(area.x + area.w - tw, x + 10);
      ctx.fillStyle = theme.panel;
      ctx.strokeStyle = theme.line;
      ctx.beginPath();
      ctx.roundRect(bx, area.y + 6, tw, 18, 4);
      ctx.fill();
      ctx.stroke();
      ctx.fillStyle = theme.text;
      ctx.textAlign = "left";
      ctx.textBaseline = "middle";
      ctx.fillText(label, bx + 6, area.y + 16);
    }
  }

  // títulos de ejes SIEMPRE con unidades
  ctx.fillStyle = theme.text;
  ctx.font = `600 11px Inter, sans-serif`;
  ctx.textAlign = "center";
  ctx.textBaseline = "bottom";
  ctx.fillText(spec.xLabel, area.x + area.w / 2, area.y + area.h + 34);
  ctx.save();
  ctx.translate(area.x - 46, area.y + area.h / 2);
  ctx.rotate(-Math.PI / 2);
  ctx.textBaseline = "top";
  ctx.fillText(spec.yLabel, 0, 0);
  ctx.restore();
  ctx.restore();
}

export function formatTick(value: number, digits = 2): string {
  if (!Number.isFinite(value)) return "—";
  if (value === 0) return "0";
  const abs = Math.abs(value);
  if (abs >= 1e5 || abs < 1e-2) {
    const exp = Math.floor(Math.log10(abs));
    const mant = value / 10 ** exp;
    return `${mant.toFixed(1)}e${exp}`;
  }
  return value.toFixed(digits);
}

/** Exporta el canvas (incluye dpr) a PNG en un Blob. */
export function canvasToPng(canvas: HTMLCanvasElement): Promise<Blob> {
  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) resolve(blob);
      else reject(new Error("toBlob falló"));
    }, "image/png");
  });
}

/** Serializa una gráfica a SVG (independiente del canvas: vectorial). */
export function plotToSvg(spec: PlotSpec, width: number, height: number): string {
  const { area, view, theme } = spec;
  const xTicks = niceTicks(view.xMin, view.xMax, spec.xTicks ?? 6);
  const yTicks = niceTicks(view.yMin, view.yMax, spec.yTicks ?? 5);
  const sx = (v: number) => sScale(v, view.xMin, view.xMax, area.x, area.x + area.w);
  const sy = (v: number) => sScale(v, view.yMin, view.yMax, area.y + area.h, area.y);
  const parts: string[] = [];
  parts.push(
    `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" font-family="JetBrains Mono, monospace">`,
  );
  parts.push(`<rect width="${width}" height="${height}" fill="${theme.panel}"/>`);
  parts.push(
    `<rect x="${area.x}" y="${area.y}" width="${area.w}" height="${area.h}" fill="none" stroke="${theme.line}"/>`,
  );
  for (const tx of xTicks) {
    parts.push(
      `<line x1="${sx(tx)}" y1="${area.y}" x2="${sx(tx)}" y2="${area.y + area.h}" stroke="${theme.grid}"/>`,
    );
    parts.push(
      `<text x="${sx(tx)}" y="${area.y + area.h + 16}" fill="${theme.muted}" font-size="10" text-anchor="middle">${formatTick(tx)}</text>`,
    );
  }
  for (const ty of yTicks) {
    parts.push(
      `<line x1="${area.x}" y1="${sy(ty)}" x2="${area.x + area.w}" y2="${sy(ty)}" stroke="${theme.grid}"/>`,
    );
    parts.push(
      `<text x="${area.x - 8}" y="${sy(ty) + 3}" fill="${theme.muted}" font-size="10" text-anchor="end">${formatTick(ty)}</text>`,
    );
  }
  for (const series of spec.series) {
    if (series.kind === "scatter") {
      series.points.forEach((p, i) => {
        const err = series.errorY?.[i] ?? 0;
        if (err > 0) {
          parts.push(
            `<line x1="${sx(p.x)}" y1="${sy(p.y - err)}" x2="${sx(p.x)}" y2="${sy(p.y + err)}" stroke="${series.color}" opacity=".55"/>`,
          );
        }
        parts.push(`<circle cx="${sx(p.x)}" cy="${sy(p.y)}" r="3.6" fill="${series.color}"/>`);
      });
    } else {
      const d = series.points
        .filter((p) => Number.isFinite(p.x) && Number.isFinite(p.y))
        .map((p, i) => `${i === 0 ? "M" : "L"}${sx(p.x).toFixed(2)},${sy(p.y).toFixed(2)}`)
        .join(" ");
      parts.push(
        `<path d="${d}" fill="none" stroke="${series.color}" stroke-width="${series.width ?? 2}"${series.dash ? ` stroke-dasharray="${series.dash.join(",")}"` : ""}/>`,
      );
    }
  }
  if (spec.marker) {
    parts.push(
      `<line x1="${sx(spec.marker.x)}" y1="${area.y}" x2="${sx(spec.marker.x)}" y2="${area.y + area.h}" stroke="${spec.marker.color}" stroke-dasharray="5,5"/>`,
    );
    parts.push(
      `<text x="${sx(spec.marker.x) + 6}" y="${area.y + 14}" fill="${spec.marker.color}" font-size="10">${spec.marker.label}</text>`,
    );
  }
  parts.push(
    `<text x="${area.x + area.w / 2}" y="${area.y + area.h + 36}" fill="${theme.text}" font-size="11" font-weight="600" text-anchor="middle">${escapeXml(spec.xLabel)}</text>`,
  );
  parts.push(
    `<text transform="translate(${area.x - 46},${area.y + area.h / 2}) rotate(-90)" fill="${theme.text}" font-size="11" font-weight="600" text-anchor="middle">${escapeXml(spec.yLabel)}</text>`,
  );
  parts.push("</svg>");
  return parts.join("\n");
}

function escapeXml(text: string): string {
  return text.replace(/[<>&"]/g, (c) =>
    c === "<" ? "&lt;" : c === ">" ? "&gt;" : c === "&" ? "&amp;" : "&quot;",
  );
}

export function downloadBlob(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 2000);
}
