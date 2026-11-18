<script lang="ts">
  // Panel izquierdo: tarjetas colapsables de Material, Luz, Voltaje,
  // Modelo de corriente y Realismo. Cada control tiene scrub sobre la
  // etiqueta, flechas (Shift ×10), doble clic para restablecer y undo global.
  import { lab } from "../../lib/store.svelte";
  import { t } from "../../lib/i18n";
  import { wavelengthToColor } from "../../lib/spectrum";
  import SpectrumSlider from "./SpectrumSlider.svelte";
  import { flux } from "../../lib/format";
  import SectionIcon from "../icons/SectionIcon.svelte";
  import type { MaterialChoice } from "../../lib/types";

  const s = $derived(t(lab.preferences.lang));
  const readout = $derived(lab.readout);
  const materials = $derived(lab.catalog);
  let open = $state<Record<string, boolean>>({
    material: true,
    light: true,
    voltage: true,
    current: false,
    realism: false,
  });

  function toggle(key: string): void {
    open = { ...open, [key]: !open[key] };
  }

  // --- Nivel de detalle -------------------------------------------------
  // En pantallas bajas (1280×800 y similares) Material + Luz + Voltaje no
  // caben desplegados. "Compacto" deja Luz y Voltaje como una línea de
  // resumen con su slider en línea: todo el panel cabe sin scroll.
  type Detail = "auto" | "compacto" | "completo";
  // "auto" elige según el alto disponible y se readapta al redimensionar.
  // Una elección manual manda hasta que se vuelve a "Auto".
  let detailChoice = $state<Detail>("auto");
  let viewportH = $state(typeof window === "undefined" ? 900 : window.innerHeight);

  const compact = $derived(
    detailChoice === "auto" ? viewportH < 900 : detailChoice === "compacto",
  );

  function setDetail(next: Detail): void {
    detailChoice = next;
    if (next === "auto") {
      lab.status = "Detalle automático: compacto en ventanas bajas, completo si hay espacio.";
    } else if (next === "compacto") {
      lab.status = "Panel compacto: Luz y Voltaje en una línea.";
    } else {
      lab.status = "Panel completo: todas las secciones desplegadas.";
    }
  }

  // --- scrub horizontal sobre la etiqueta numérica ---
  let scrubbing: { startX: number; startValue: number; apply: (v: number) => void } | null = null;

  function startScrub(event: PointerEvent, current: number, apply: (v: number) => void): void {
    scrubbing = { startX: event.clientX, startValue: current, apply };
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function moveScrub(event: PointerEvent): void {
    if (!scrubbing) return;
    const delta = (event.clientX - scrubbing.startX) * 0.4;
    scrubbing.apply(scrubbing.startValue + delta);
  }

  function endScrub(): void {
    scrubbing = null;
  }

  function setIntensity(value: number): void {
    lab.update((c) => {
      c.intensityPercent = Math.min(100, Math.max(0, Math.round(value)));
    }, `Intensidad → ${Math.round(value)}%`);
  }

  function setWavelength(value: number): void {
    const wl = Math.min(900, Math.max(180, Math.round(value)));
    lab.update((c) => {
      c.wavelengthNm = wl;
    }, `λ → ${wl} nm`);
  }

  function setVoltage(value: number): void {
    lab.update((c) => {
      c.appliedVoltageV = Math.min(5, Math.max(-5, Number(value.toFixed(2))));
    }, `Voltaje → ${value.toFixed(2)} V`);
  }

  function chooseMaterial(id: MaterialChoice): void {
    lab.update((c) => {
      c.material = id;
    }, `Material → ${lab.titleFor(id)}`);
  }

  function chooseComparison(id: MaterialChoice): void {
    lab.update((c) => {
      c.comparisonMaterial = id;
    }, `Comparación → ${lab.titleFor(id)}`);
  }

  const qePresets: [string, number, string][] = [
    ["Demo 1%", 1, "QE 1 % ilustrativa, sin fuente"],
    ["Bialcali 25%", 25, "K₂CsSb, pico ≈ 25 % @ ~400 nm (Hamamatsu PMT Handbook)"],
    ["Multialcali 20%", 20, "Na₂KSb:Cs, pico ≈ 20 % (Hamamatsu PMT Handbook)"],
  ];

  const v0 = $derived(readout?.stoppingPotentialV ?? 0);
  const blockingZonePct = $derived(Math.min(100, (v0 / 5) * 100));
</script>

<svelte:window onresize={() => (viewportH = window.innerHeight)} />

<aside class="controls-panel" aria-label="Controles del experimento">
  <div class="detail-row" role="group" aria-label="Nivel de detalle">
    <span class="small muted" title={detailChoice === "auto" ? "Automático según el alto de la ventana" : "Elegido a mano"}>
      Detalle{detailChoice === "auto" ? " · auto" : ""}
    </span>
    <div class="seg">
      <button class="seg-btn" class:active={detailChoice === "auto"} aria-pressed={detailChoice === "auto"} onclick={() => setDetail("auto")}>
        Auto
      </button>
      <button class="seg-btn" class:active={detailChoice === "compacto"} aria-pressed={detailChoice === "compacto"} onclick={() => setDetail("compacto")}>
        Compacto
      </button>
      <button class="seg-btn" class:active={detailChoice === "completo"} aria-pressed={detailChoice === "completo"} onclick={() => setDetail("completo")}>
        Completo
      </button>
    </div>
  </div>
  <!-- MATERIAL -->
  <section class="card section">
    <button class="section-head" onclick={() => toggle("material")} aria-expanded={open.material}>
      <span class="section-icon"><SectionIcon name="material" /></span>
      <span>{s.material}</span>
      <span class="section-hint small muted">{lab.titleFor(lab.controls.material)}</span>
      <span class="chev" class:rot={open.material}>›</span>
    </button>
    {#if open.material}
      <div class="section-body rise">
        <div class="material-grid" role="radiogroup" aria-label="Cátodo">
          {#each materials as material (material.id)}
            {@const selected = lab.controls.material === material.id}
            <button
              class="material-card"
              class:selected
              role="radio"
              aria-checked={selected}
              onclick={() => chooseMaterial(material.id as MaterialChoice)}
              title={material.source}
            >
              <span class="symbol">{material.symbol}</span>
              <span class="m-name small" title={material.nameEs}>{lab.preferences.lang === "en" ? material.nameEn : material.nameEs}</span>
              <span class="mono phi">Φ {material.workFunctionEv.toFixed(2)} eV</span>
              <span class="mono th small">λ₀ {material.thresholdWavelengthNm.toFixed(0)} nm</span>
              <span class="mini-spectrum">
                <span
                  class="threshold-dot"
                  style:left="{((material.thresholdWavelengthNm - 180) / 720) * 100}%"
                ></span>
              </span>
            </button>
          {/each}
        </div>
        <div class="custom-row">
          <button
            class="chip-btn"
            class:active={lab.controls.material === "custom"}
            onclick={() => chooseMaterial("custom")}
          >Φ personalizada</button>
          {#if lab.controls.material === "custom"}
            <input
              type="range"
              min="1"
              max="6.5"
              step="0.05"
              value={lab.controls.customPhiEv}
              oninput={(e) => {
                const v = Number((e.currentTarget as HTMLInputElement).value);
                lab.update((c) => {
                  c.customPhiEv = v;
                }, `Φ personalizada → ${v.toFixed(2)} eV`);
              }}
              aria-label="Función de trabajo personalizada en eV"
            />
            <span class="mono small">{(readout?.workFunctionEv ?? 0).toFixed(2)} eV</span>
          {/if}
        </div>
        <div class="compare-row">
          <span class="small muted">Comparar con</span>
          <select
            value={lab.controls.comparisonMaterial}
            onchange={(e) => chooseComparison((e.currentTarget as HTMLSelectElement).value as MaterialChoice)}
            aria-label="Material de comparación"
          >
            {#each materials as material (material.id)}
              <option value={material.id}>{material.symbol} · {material.nameEs}</option>
            {/each}
          </select>
        </div>
      </div>
    {/if}
  </section>

  <!-- LUZ -->
  <section class="card section">
    <button class="section-head" onclick={() => toggle("light")} aria-expanded={open.light}>
      <span class="section-icon" style:color={wavelengthToColor(lab.controls.wavelengthNm).css}><SectionIcon name="light" /></span>
      <span>{s.light}</span>
      <span class="section-hint small muted mono">{lab.controls.wavelengthNm.toFixed(0)} nm</span>
      <span class="chev" class:rot={open.light}>›</span>
    </button>
    {#if compact && open.light}
      <div class="section-body compact-row">
        <span class="mini-cap mono">λ</span>
        <label class="mini">
          <span class="sr-only">Longitud de onda (nm)</span>
          <input
            type="range"
            min="180"
            max="900"
            step="1"
            value={lab.controls.wavelengthNm}
            oninput={(e) => setWavelength(Number((e.currentTarget as HTMLInputElement).value))}
            aria-label="Longitud de onda (nm)"
            style:accent-color={wavelengthToColor(lab.controls.wavelengthNm).css}
          />
        </label>
        <span class="mini-cap mono">%</span>
        <label class="mini">
          <span class="sr-only">Intensidad (%)</span>
          <input
            type="range"
            min="0"
            max="100"
            step="1"
            value={lab.controls.intensityPercent}
            oninput={(e) => setIntensity(Number((e.currentTarget as HTMLInputElement).value))}
            aria-label="Intensidad de la luz (%)"
          />
        </label>
      </div>
    {:else if open.light}
      <div class="section-body rise">
        <SpectrumSlider />

        <div class="slider-row">
          <div class="slider-label">
            <span class="small muted">{s.intensity}</span>
            <span
              class="mono scrub"
              onpointerdown={(e) => startScrub(e, lab.controls.intensityPercent, setIntensity)}
              onpointermove={moveScrub}
              onpointerup={endScrub}
              ondblclick={() => setIntensity(55)}
              role="slider"
              tabindex="0"
              aria-label="Intensidad, arrastra para ajustar"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={lab.controls.intensityPercent}
              onkeydown={(e) => {
                if (e.key === "ArrowRight") setIntensity(lab.controls.intensityPercent + (e.shiftKey ? 10 : 1));
                if (e.key === "ArrowLeft") setIntensity(lab.controls.intensityPercent - (e.shiftKey ? 10 : 1));
              }}
            >{lab.controls.intensityPercent.toFixed(0)} %</span>
          </div>
          <input
            type="range"
            min="0"
            max="100"
            step="1"
            value={lab.controls.intensityPercent}
            aria-label="Intensidad de la luz (%)"
            oninput={(e) => setIntensity(Number((e.currentTarget as HTMLInputElement).value))}
          />
          <div class="sub muted mono small">
            flujo ≈ {flux(readout?.photonFluxPerM2S)} · {(lab.controls.intensityPercent / 10).toFixed(1)} mW/cm²
          </div>
        </div>
      </div>
    {/if}
  </section>

  <!-- VOLTAJE -->
  <section class="card section">
    <button class="section-head" onclick={() => toggle("voltage")} aria-expanded={open.voltage}>
      <span class="section-icon"><SectionIcon name="voltage" /></span>
      <span>{s.voltage}</span>
      <span class="section-hint small muted mono">{lab.controls.appliedVoltageV > 0 ? "+" : ""}{lab.controls.appliedVoltageV.toFixed(2)} V</span>
      <span class="chev" class:rot={open.voltage}>›</span>
    </button>
    {#if compact && open.voltage}
      <div class="section-body compact-row">
        <span class="mini-cap mono">V</span>
        <label class="mini">
          <span class="sr-only">Voltaje aplicado (V)</span>
          <input
            type="range"
            min="-5"
            max="5"
            step="0.01"
            value={lab.controls.appliedVoltageV}
            oninput={(e) => setVoltage(Number((e.currentTarget as HTMLInputElement).value))}
            aria-label="Voltaje aplicado (V)"
          />
        </label>
        <span class="small muted">{readout?.blocked ? "bloqueado" : `rampa ${((readout?.collectionFactor ?? 0) * 100).toFixed(0)} %`}</span>
      </div>
    {:else if open.voltage}
      <div class="section-body rise">
        <div class="slider-row">
          <div class="slider-label">
            <span class="small muted">V aplicado (−5 … +5 V)</span>
            <span
              class="mono scrub"
              onpointerdown={(e) => startScrub(e, lab.controls.appliedVoltageV, setVoltage)}
              onpointermove={moveScrub}
              onpointerup={endScrub}
              ondblclick={() => setVoltage(0)}
              role="slider"
              tabindex="0"
              aria-label="Voltaje aplicado, arrastra para ajustar"
              aria-valuemin={-5}
              aria-valuemax={5}
              aria-valuenow={lab.controls.appliedVoltageV}
              onkeydown={(e) => {
                if (e.key === "ArrowRight") setVoltage(lab.controls.appliedVoltageV + (e.shiftKey ? 0.5 : 0.05));
                if (e.key === "ArrowLeft") setVoltage(lab.controls.appliedVoltageV - (e.shiftKey ? 0.5 : 0.05));
              }}
            >{lab.controls.appliedVoltageV > 0 ? "+" : ""}{lab.controls.appliedVoltageV.toFixed(2)} V</span>
          </div>
          <div class="voltage-track">
            <div class="blocking-zone" style:width="{blockingZonePct / 2}%" title="zona de bloqueo: V < −V₀"></div>
            <div class="zero-mark"></div>
            {#if v0 > 0}
              <div
                class="v0-mark"
                style:left="{(50 - (Math.min(v0, 5) / 5) * 50).toFixed(2)}%"
                title="−V₀ = {(-v0).toFixed(2)} V"
              ></div>
            {/if}
            <input
              type="range"
              min="-5"
              max="5"
              step="0.01"
              value={lab.controls.appliedVoltageV}
              aria-label="Voltaje aplicado (V)"
              oninput={(e) => setVoltage(Number((e.currentTarget as HTMLInputElement).value))}
            />
          </div>
          <div class="sub muted small">
            {#if readout?.blocked}
              <span style:color="var(--warn)">Bloqueado: V &lt; −V₀</span>
            {:else if v0 > 0}
              −V₀ = {(-v0).toFixed(2)} V · rampa {((readout?.collectionFactor ?? 0) * 100).toFixed(0)} %
            {:else}
              sin emisión: no hay rampa de frenado
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </section>

  <!-- MODELO DE CORRIENTE -->
  <section class="card section">
    <button class="section-head" onclick={() => toggle("current")} aria-expanded={open.current}>
      <span class="section-icon"><SectionIcon name="current" /></span>
      <span>{s.currentModel}</span>
      <span class="section-hint small muted mono">QE {lab.controls.quantumEfficiencyPercent.toFixed(0)}%</span>
      <span class="chev" class:rot={open.current}>›</span>
    </button>
    {#if open.current}
      <div class="section-body rise">
        <div class="slider-row">
          <div class="slider-label">
            <span class="small muted">Área del cátodo</span>
            <span class="mono">{lab.controls.cathodeAreaCm2.toFixed(2)} cm²</span>
          </div>
          <input
            type="range"
            min="0.05"
            max="5"
            step="0.05"
            value={lab.controls.cathodeAreaCm2}
            aria-label="Área del cátodo (cm²)"
            oninput={(e) => {
              const v = Number((e.currentTarget as HTMLInputElement).value);
              lab.update((c) => { c.cathodeAreaCm2 = v; }, `Área → ${v.toFixed(2)} cm²`);
            }}
          />
        </div>
        <div class="slider-row">
          <div class="slider-label">
            <span class="small muted">Eficiencia cuántica</span>
            <span class="mono">{lab.controls.quantumEfficiencyPercent.toFixed(0)} %</span>
          </div>
          <input
            type="range"
            min="0"
            max="50"
            step="0.5"
            value={lab.controls.quantumEfficiencyPercent}
            aria-label="Eficiencia cuántica (%)"
            oninput={(e) => {
              const v = Number((e.currentTarget as HTMLInputElement).value);
              lab.update((c) => { c.quantumEfficiencyPercent = v; }, `QE → ${v.toFixed(1)}%`);
            }}
          />
        </div>
        <div class="presets">
          {#each qePresets as [label, qe, source]}
            <span class="tooltip">
              <button
                class="chip-btn"
                onclick={() => lab.update((c) => { c.quantumEfficiencyPercent = qe; }, `QE preset ${label}`)}
              >{label}</button>
              <span class="tip">{source}</span>
            </span>
          {/each}
        </div>
        <p class="xsmall muted">I = e·Φ·A·QE·g(V). Área y QE son calibrables; el resto sale del motor.</p>
      </div>
    {/if}
  </section>

  <!-- REALISMO -->
  <section class="card section">
    <button class="section-head" onclick={() => toggle("realism")} aria-expanded={open.realism}>
      <span class="section-icon"><SectionIcon name="realism" /></span>
      <span>{s.realism}</span>
      <span class="section-hint small muted">{lab.experiment.experimentalMode ? "experimental" : "ideal"}</span>
      <span class="chev" class:rot={open.realism}>›</span>
    </button>
    {#if open.realism}
      <div class="section-body rise">
        <label class="switch-row">
          <span class="small">Ruido experimental</span>
          <input
            type="checkbox"
            checked={lab.experiment.experimentalMode}
            onchange={() => lab.experimentalModeToggle()}
          />
        </label>
        <div class="slider-row">
          <div class="slider-label">
            <span class="small muted">Magnitud del ruido</span>
            <span class="mono">±{lab.experiment.noisePercent.toFixed(1)} %</span>
          </div>
          <input
            type="range"
            min="0"
            max="10"
            step="0.5"
            value={lab.experiment.noisePercent}
            aria-label="Magnitud del ruido (%)"
            oninput={(e) => {
              lab.experiment = {
                ...lab.experiment,
                noisePercent: Number((e.currentTarget as HTMLInputElement).value),
              };
            }}
          />
        </div>
        <div class="slider-row">
          <div class="slider-label">
            <span class="small muted">Escala de velocidad</span>
            <span class="mono small">{lab.preferences.speedScale === "compressed" ? "√(K/1 eV)" : "log v real"}</span>
          </div>
          <select
            value={lab.preferences.speedScale}
            onchange={(e) => {
              const value = (e.currentTarget as HTMLSelectElement).value as "compressed" | "real_log";
              lab.setPreferences({ speedScale: value }, "Escala de velocidad");
            }}
            aria-label="Escala de velocidad de los electrones"
          >
            <option value="compressed">Comprimida 0.5–2.2</option>
            <option value="real_log">Real logarítmica</option>
          </select>
          <p class="xsmall muted">
            {lab.preferences.speedScale === "compressed"
              ? "√(Kmax/1 eV) acotada a 0.5–2.2: comprime diferencias grandes (Na vs Pt)."
              : "v real en m/s, con leyenda; los electrones suelen ser ~10⁶ m/s."}
          </p>
        </div>
      </div>
    {/if}
  </section>
</aside>

<style>
  .controls-panel {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    padding: var(--sp-3);
    overflow-y: auto;
    height: 100%;
  }
  .detail-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    padding: 0 2px;
    flex: 0 0 auto;
  }
  .seg {
    display: inline-flex;
    background: var(--panel-2);
    border: 1px solid var(--line);
    border-radius: 999px;
    padding: 2px;
  }
  .seg-btn {
    /* Un <button> sin clase .btn hereda el fondo de botón del navegador;
       lo neutralizamos explícitamente. */
    all: unset;
    box-sizing: border-box;
    cursor: pointer;
    color: var(--muted);
    font: inherit;
    font-size: 0.76em;
    font-weight: 600;
    padding: 3px 9px;
    border-radius: 999px;
    cursor: pointer;
  }
  /* Mismo patrón que las pestañas: el token --wl es válido en este
     contexto. El fallback anterior (--accent) pintaba el botón INACTIVO. */
  .seg-btn:focus-visible {
    outline: 2px solid var(--wl);
    outline-offset: 1px;
  }
  .seg-btn.active {
    background: var(--wl);
    color: var(--on-accent, #05121b);
  }
  .compact-row {
    display: flex;
    /* .section-body declara flex-direction: column con más especificidad
       (body + .rise); en esta fila debe ganar el eje horizontal. */
    flex-direction: row !important;
    flex-wrap: nowrap;
    align-items: center;
    gap: 7px;
    padding: 0 12px 11px;
    justify-content: flex-start;
  }
  .mini-cap {
    flex: 0 0 auto;
    width: 14px;
    font-size: 0.78em;
    color: var(--muted);
    text-align: right;
  }
  .compact-row input[type="range"] {
    accent-color: var(--wl);
    min-width: 0;
  }
  .compact-row .mini {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    align-items: center;
  }
  .compact-row input[type="range"] {
    width: 100%;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
  .section {
    padding: 0;
    overflow: hidden;
    /* Sin esto la columna flex aplasta las tarjetas y su contenido se corta
       (Material medía 330 px de contenido en 257 px de caja). */
    flex: 0 0 auto;
  }
  .section-head {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 12px;
    background: transparent;
    border: 0;
    color: var(--text);
    font-weight: 600;
    font-size: 0.86em;
    text-align: left;
  }
  .section-head:hover {
    background: var(--panel-2);
  }
  .section-icon {
    font-size: 1.1em;
    color: var(--wl);
  }
  .section-hint {
    margin-left: auto;
    font-weight: 500;
  }
  .chev {
    transition: transform var(--dur-2) var(--ease);
    color: var(--muted);
    transform: rotate(90deg);
  }
  .chev.rot {
    transform: rotate(270deg);
  }
  .section-body {
    padding: 0 12px 12px;
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  .material-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px;
  }
  .material-card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: 8px;
    border-radius: var(--r-sm);
    border: 1px solid var(--line);
    background: var(--panel-2);
    color: var(--text-2);
    transition:
      border-color var(--dur-1) var(--ease),
      background var(--dur-1) var(--ease);
  }
  .material-card:hover {
    border-color: var(--wl-strong);
  }
  .material-card.selected {
    border-color: var(--wl);
    background: var(--wl-soft);
    color: var(--text);
    box-shadow: inset 0 0 0 1px var(--wl);
  }
  .symbol {
    font-weight: 700;
    font-size: 1.05em;
    color: var(--text);
  }
  .m-name {
    color: var(--muted);
    font-size: 0.75em;
    /* "Platino"/"Platinum" caben en tarjetas de ~90 px sin empujar el layout */
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .phi,
  .th {
    font-size: 0.72em;
  }
  .mini-spectrum {
    position: relative;
    display: block;
    height: 4px;
    width: 100%;
    margin-top: 4px;
    border-radius: 2px;
    background: linear-gradient(
      90deg,
      #4c1d95,
      #6d28d9,
      #3b82f6,
      #22c55e,
      #eab308,
      #ef4444,
      #7f1d1d
    );
    opacity: 0.7;
  }
  .threshold-dot {
    position: absolute;
    top: -2px;
    width: 2px;
    height: 8px;
    background: #fff;
    box-shadow: 0 0 4px rgba(255, 255, 255, 0.8);
    border-radius: 1px;
  }
  .custom-row,
  .compare-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .custom-row input[type="range"] {
    flex: 1;
  }
  .chip-btn {
    padding: 3px 9px;
    font-size: 0.74em;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: var(--panel-2);
    color: var(--text-2);
  }
  .chip-btn:hover,
  .chip-btn.active {
    border-color: var(--wl);
    color: var(--text);
  }
  .compare-row select {
    font-size: 0.8em;
    flex: 1;
    padding: 4px 6px;
  }
  .slider-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .slider-label {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .scrub {
    cursor: ew-resize;
    font-size: 0.86em;
    color: var(--text);
    user-select: none;
    padding: 0 2px;
    border-radius: 3px;
  }
  .scrub:focus-visible {
    outline: 2px solid var(--wl);
  }
  .sub {
    font-size: 0.7em;
  }
  input[type="range"] {
    width: 100%;
    accent-color: var(--wl);
  }
  .voltage-track {
    position: relative;
    height: 26px;
  }
  .voltage-track input[type="range"] {
    position: absolute;
    inset: 0;
    margin: 0;
    z-index: 2;
    background: transparent;
  }
  .blocking-zone {
    position: absolute;
    left: 0;
    top: 8px;
    bottom: 8px;
    border-radius: 6px;
    background: color-mix(in srgb, var(--warn) 22%, transparent);
    z-index: 0;
    pointer-events: none;
  }
  .zero-mark {
    position: absolute;
    left: 50%;
    top: 4px;
    bottom: 4px;
    width: 1px;
    background: var(--line-strong);
    z-index: 0;
  }
  .v0-mark {
    position: absolute;
    top: 2px;
    bottom: 2px;
    width: 2px;
    background: var(--warn);
    z-index: 1;
    pointer-events: none;
  }
  .presets {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }
  .switch-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
</style>
