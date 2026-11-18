<script lang="ts">
  // Slider de λ con el espectro visible pintado como fondo y la marca del
  // umbral λ₀. El riel se tiñe con el color real de la luz; UV/IR añaden
  // patrón rayado + etiqueta "no visible".
  import { lab } from "../../lib/store.svelte";
  import { spectrumGradient, wavelengthToColor } from "../../lib/spectrum";
  import { thzToNm } from "../../lib/format";

  let {
    value = $bindable(lab.controls.wavelengthNm),
    min = 180,
    max = 900,
  }: { value?: number; min?: number; max?: number } = $props();

  const color = $derived(wavelengthToColor(value));
  const thresholdNm = $derived(lab.readout?.thresholdWavelengthNm ?? null);
  const pct = $derived(((value - min) / (max - min)) * 100);
  const thresholdPct = $derived(
    thresholdNm != null && thresholdNm >= min && thresholdNm <= max
      ? ((thresholdNm - min) / (max - min)) * 100
      : null,
  );
  let mode = $state<"nm" | "thz" | "ev">("nm");
  let inputValue = $state(String(Math.round(value)));

  $effect(() => {
    inputValue = mode === "nm" ? value.toFixed(1)
      : mode === "thz" ? (299792.458 / value).toFixed(1)
      : (1239.841984 / value).toFixed(3);
  });

  function commit(raw: string): void {
    const numeric = Number(raw.replace(",", "."));
    if (!Number.isFinite(numeric) || numeric <= 0) return;
    let nm = numeric;
    if (mode === "thz") nm = thzToNm(numeric);
    if (mode === "ev") nm = 1239.841984 / numeric;
    lab.update((c) => {
      c.wavelengthNm = Math.min(max, Math.max(min, nm));
    }, `λ → ${nm.toFixed(1)} nm`);
  }

  function nudge(delta: number, shift: boolean): void {
    const step = shift ? 10 : 1;
    lab.update((c) => {
      c.wavelengthNm = Math.min(max, Math.max(min, c.wavelengthNm + delta * step));
    }, `λ → ${value + delta * step} nm`);
  }

  const quickButtons: [string, number][] = [
    ["UV", 254],
    ["Violeta", 405],
    ["Verde", 532],
    ["Rojo", 650],
    ["IR", 850],
  ];

  function setQuick(nm: number, label: string): void {
    lab.update((c) => {
      c.wavelengthNm = nm;
    }, `λ → ${label} (${nm} nm)`);
  }
</script>

<div class="spectrum-control">
  <div class="top-row">
    <span class="label" style:color={color.css}>λ · {color.visible ? "" : "no visible"}</span>
    <span class="mono value" style:color={color.css}>{value.toFixed(1)} nm</span>
  </div>

  <div class="track-wrap" class:invisible={!color.visible}>
    <div class="track" style:background={spectrumGradient()}></div>
    {#if thresholdPct != null}
      <div
        class="threshold-mark"
        style:left="{thresholdPct}%"
        title="λ₀ = {thresholdNm?.toFixed(0)} nm (umbral del material)"
      >
        <span class="threshold-label mono">λ₀</span>
      </div>
    {/if}
    <div class="thumb" style:left="{pct}%" style:background={color.css} aria-hidden="true"></div>
    <input
      type="range"
      {min}
      {max}
      step="0.5"
      value={value}
      aria-label="Longitud de onda (nm)"
      aria-valuetext="{value.toFixed(0)} nanómetros"
      oninput={(e) => {
        const nm = Number((e.currentTarget as HTMLInputElement).value);
        lab.update((c) => {
          c.wavelengthNm = nm;
        }, `λ → ${nm.toFixed(0)} nm`);
      }}
      onkeydown={(e) => {
        if (e.key === "ArrowRight" || e.key === "ArrowUp") {
          e.preventDefault();
          nudge(1, e.shiftKey);
        } else if (e.key === "ArrowLeft" || e.key === "ArrowDown") {
          e.preventDefault();
          nudge(-1, e.shiftKey);
        }
      }}
      ondblclick={() => setQuick(400, "restablecer")}
    />
  </div>

  <div class="numeric-row">
    <div class="modes">
      {#each [["nm", "nm"], ["thz", "THz"], ["ev", "eV"]] as const as [m, label]}
        <button
          class="btn ghost tiny"
          class:active={mode === m}
          onclick={() => (mode = m)}
          aria-pressed={mode === m}
        >{label}</button>
      {/each}
    </div>
    <input
      class="mono num"
      type="number"
      bind:value={inputValue}
      onchange={() => commit(inputValue)}
      onkeydown={(e) => e.key === "Enter" && commit(inputValue)}
      aria-label="Valor de longitud de onda en {mode}"
    />
    <span class="conversion small muted mono">
      {(299792.458 / value).toFixed(0)} THz ↔ {(1239.841984 / value).toFixed(2)} eV
    </span>
  </div>

  <div class="quick">
    {#each quickButtons as [label, nm]}
      <button class="chip-btn" onclick={() => setQuick(nm, label)}>
        <i style:background={wavelengthToColor(nm).css} class:hatch={!wavelengthToColor(nm).visible}></i>
        {label}
      </button>
    {/each}
  </div>
</div>

<style>
  .spectrum-control {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .top-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .top-row .label {
    letter-spacing: 0.05em;
  }
  .value {
    font-weight: 600;
    font-size: 0.95em;
    transition: color var(--dur-1) var(--ease);
  }
  .track-wrap {
    position: relative;
    height: 30px;
  }
  .track {
    position: absolute;
    inset: 8px 0;
    border-radius: 999px;
    opacity: 0.85;
    filter: saturate(1.1);
  }
  .track-wrap.invisible .track {
    background-image: repeating-linear-gradient(
      135deg,
      rgba(255, 255, 255, 0.18) 0 2px,
      transparent 2px 8px
    ) !important;
    background-blend-mode: overlay;
  }
  input[type="range"] {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    background: transparent;
    -webkit-appearance: none;
    appearance: none;
    cursor: ew-resize;
  }
  input[type="range"]::-webkit-slider-runnable-track {
    height: 100%;
    background: transparent;
  }
  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 0;
    height: 0;
  }
  input[type="range"]::-moz-range-thumb {
    width: 0;
    height: 0;
    border: 0;
  }
  .thumb {
    position: absolute;
    top: 2px;
    bottom: 2px;
    width: 4px;
    margin-left: -2px;
    border-radius: 3px;
    box-shadow: 0 0 12px currentColor, 0 0 2px #000;
    pointer-events: none;
    transition: left 60ms linear;
  }
  .threshold-mark {
    position: absolute;
    top: -2px;
    bottom: -2px;
    width: 0;
    border-left: 2px dashed var(--warn);
    pointer-events: none;
  }
  .threshold-label {
    position: absolute;
    top: -14px;
    left: -9px;
    font-size: 9px;
    color: var(--warn);
  }
  .numeric-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .modes {
    display: inline-flex;
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    overflow: hidden;
  }
  .tiny {
    font-size: 0.7em;
    padding: 2px 6px;
    border-radius: 0;
  }
  .num {
    width: 90px;
    padding: 4px 6px;
    font-size: 0.85em;
  }
  .conversion {
    margin-left: auto;
    font-size: 0.68em;
    opacity: 0.8;
  }
  .quick {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }
  .chip-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 8px;
    font-size: 0.72em;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: var(--panel-2);
    color: var(--text-2);
    transition: border-color var(--dur-1) var(--ease);
  }
  .chip-btn:hover {
    border-color: var(--wl);
  }
  .chip-btn i {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .chip-btn i.hatch {
    border-radius: 2px;
  }
</style>
