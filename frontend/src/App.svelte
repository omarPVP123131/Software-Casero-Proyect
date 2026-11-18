<script lang="ts">
  // Composición: barra superior, tres paneles redimensionables, zona de
  // pestañas (celda + gráficas), barra de estado, paleta y tutorial.
  import { onMount, untrack } from "svelte";
  import { lab } from "./lib/store.svelte";
  import { t } from "./lib/i18n";
  import { startClock, resetTime, stepFrame } from "./lib/playback";
  import { SCENARIOS } from "./lib/scenarios";
  import { saveSession, openSession, autosave, readAutosave } from "./lib/io";
  import { experimentCsv, isTauri } from "./lib/physics";
  import { isMac } from "./lib/platform";
  import { canvasToPng, downloadBlob } from "./lib/plots";
  import type { ExperimentControls } from "./lib/types";
  import { wavelengthToColor } from "./lib/spectrum";
  import TopBar from "./components/TopBar.svelte";
  import ControlsPanel from "./components/controls/ControlsPanel.svelte";
  import Scene from "./components/Scene.svelte";
  import ReadoutsPanel from "./components/ReadoutsPanel.svelte";
  import PanelGraphs from "./components/graphs/PanelGraphs.svelte";
  import Notebook from "./components/Notebook.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import CommandPalette from "./components/CommandPalette.svelte";
  import Tutorial from "./components/Tutorial.svelte";
  import ShortcutsOverlay from "./components/ShortcutsOverlay.svelte";

  const s = $derived(t(lab.preferences.lang));
  let paletteOpen = $state(false);
  let showNotebook = $state(false);
  let shortcutsOpen = $state(false);
  let dragging: { side: "left" | "right"; startX: number; startWidth: number } | null = null;

  type PanelSide = "controls" | "readouts";
  // A partir de 1180 px los paneles pasan a ser drawers superpuestos: así
  // siguen accesibles en portátiles de 1024–1366 px.
  let narrow = $state(
    typeof window !== "undefined" && window.matchMedia("(max-width: 1180px)").matches,
  );
  let drawer = $state<PanelSide | null>(null);

  function togglePanel(side: PanelSide): void {
    if (narrow) {
      drawer = drawer === side ? null : side;
      return;
    }
    if (side === "controls") {
      lab.setPreferences(
        { controlsPanelOpen: !lab.preferences.controlsPanelOpen },
        "Panel de controles",
      );
    } else {
      lab.setPreferences(
        { readoutsPanelOpen: !lab.preferences.readoutsPanelOpen },
        "Panel de lecturas",
      );
    }
  }

  onMount(() => {
    lab.restoreLocal();
    void lab.init().then(async () => {
      const autosaved = await readAutosave();
      if (autosaved && autosaved.controls) {
        // Arranque: sesión donde la dejaste (sin pisar si falla el parseo).
        try {
          lab.loadSession(autosaved);
          lab.status = "Sesión autoguardada restaurada.";
        } catch {
          lab.status = "Autoguardado ilegible: usando valores por defecto.";
        }
      }
    });
    const stopClock = startClock();
    const unsubscribers = [bindKeyboard()];
    // autoguardado cada 20 s
    const autoTimer = setInterval(() => {
      lab.persistLocal();
      void autosave(lab.snapshot());
    }, 20_000);
    const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    if (motionQuery.matches) {
      lab.preferences = { ...lab.preferences, motion: "reduced" };
      // Accesibilidad: respetar prefers-reduced-motion pausa la animación;
      // el usuario puede reanudarla con el botón ▶ o con Espacio.
      lab.animating = false;
    }
    const narrowQuery = window.matchMedia("(max-width: 1180px)");
    const onNarrow = (event: MediaQueryListEvent): void => {
      narrow = event.matches;
      if (!event.matches) drawer = null;
    };
    narrow = narrowQuery.matches;
    narrowQuery.addEventListener("change", onNarrow);
    return () => {
      stopClock();
      clearInterval(autoTimer);
      unsubscribers.forEach((u) => u());
      narrowQuery.removeEventListener("change", onNarrow);
    };
  });

  // tema + escala de fuente en <html>
  $effect(() => {
    document.documentElement.dataset.theme = lab.preferences.theme;
    document.documentElement.dataset.motion = lab.preferences.motion;
    document.documentElement.style.setProperty("--font-scale", String(lab.preferences.fontScale));
  });

  // La barra de título nativa sigue al tema de la app (Windows/macOS).
  $effect(() => {
    const nativeTheme = lab.preferences.theme === "light" ? "light" : "dark";
    document.documentElement.style.colorScheme = nativeTheme;
    if (!isTauri()) return;
    void import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => getCurrentWindow().setTheme(nativeTheme))
      .catch(() => {});
  });

  // color de acento global según λ (borde activo, foco, slider)
  $effect(() => {
    const rgb = wavelengthToColor(lab.controls.wavelengthNm).rgb;
    document.documentElement.style.setProperty("--wl", `rgb(${rgb[0]} ${rgb[1]} ${rgb[2]})`);
    document.documentElement.style.setProperty("--wl-soft", `rgba(${rgb[0]},${rgb[1]},${rgb[2]},0.14)`);
    document.documentElement.style.setProperty("--wl-strong", `rgba(${rgb[0]},${rgb[1]},${rgb[2]},0.4)`);
  });

  // Escenario activo: se muestra como chip en la barra superior y tiñe un
  // filete de acento (sello visual). Se limpia al tocar cualquier control
  // manual para no mentir sobre el estado.
  let activeScenario = $state<string | null>(null);
  const SCENARIO_ACCENT: Record<string, string> = {
    threshold: "#A78BFA",
    intensity: "#38BDF8",
    stopping: "#F59E0B",
    measure_h: "#34D399",
  };
  const scenarioAccent = $derived(activeScenario ? SCENARIO_ACCENT[activeScenario] : null);

  // El sello del escenario deja de ser cierto en cuanto se toca un control a
  // mano: se compara la firma de los controles con la que dejó el escenario.
  const controlsSignature = $derived(
    JSON.stringify([
      lab.controls.material,
      lab.controls.customPhiEv,
      lab.controls.wavelengthNm,
      lab.controls.intensityPercent,
      lab.controls.appliedVoltageV,
      lab.controls.cathodeAreaCm2,
      lab.controls.quantumEfficiencyPercent,
    ]),
  );
  let scenarioSignature: string | null = null;

  $effect(() => {
    const sig = controlsSignature;
    untrack(() => {
      if (scenarioSignature !== null && sig !== scenarioSignature) {
        activeScenario = null;
        scenarioSignature = null;
      }
    });
  });

  function applyScenario(id: (typeof SCENARIOS)[number]["id"]): void {
    const scenario = SCENARIOS.find((s) => s.id === id);
    if (!scenario) return;
    const next = scenario.apply(lab.controls);
    lab.update((c) => Object.assign(c, next) as ExperimentControls, `Escenario: ${scenario.labelEs}`);
    // La firma se toma después del update: así el efecto anterior no lo borra.
    scenarioSignature = JSON.stringify([
      lab.controls.material,
      lab.controls.customPhiEv,
      lab.controls.wavelengthNm,
      lab.controls.intensityPercent,
      lab.controls.appliedVoltageV,
      lab.controls.cathodeAreaCm2,
      lab.controls.quantumEfficiencyPercent,
    ]);
    if (id === "measure_h") {
      lab.tab = "experiment";
      void lab.autoSweep(6);
    } else if (id === "intensity" || id === "stopping") {
      lab.tab = "iv";
    } else {
      lab.tab = "cell";
    }
    lab.status = scenario.detailEs;
    activeScenario = id;
  }

  async function handleSave(): Promise<void> {
    const path = await saveSession(lab.snapshot());
    lab.status = path ? `Sesión guardada: ${path}` : "Guardado cancelado.";
    lab.pushHistory("Sesión guardada");
  }

  async function handleOpen(): Promise<void> {
    const session = await openSession();
    if (session) {
      lab.loadSession(session);
      lab.status = "Sesión cargada.";
    }
  }

  async function handleExport(kind: "csv" | "png" | "svg" | "pdf"): Promise<void> {
    if (kind === "csv") {
      const csv = await experimentCsv(lab.experiment);
      downloadBlob(new Blob([csv], { type: "text/csv" }), "photolab-experimento.csv");
      lab.status = "CSV exportado.";
    } else if (kind === "png") {
      if (!lab.sceneCanvas) return;
      downloadBlob(await canvasToPng(lab.sceneCanvas), "photolab-escena.png");
      lab.status = "Canvas exportado a PNG.";
    } else if (kind === "svg") {
      // Las gráficas tienen su propio export SVG; aquí exportamos la gráfica visible.
      const canvas = lab.plottingCanvas;
      if (canvas) {
        downloadBlob(await canvasToPng(canvas), "photolab-grafica.png");
        lab.status = "Gráfica exportada (usa PNG/SVG en cada gráfica para vectorial).";
      }
    } else {
      showNotebook = true;
      lab.status = "Cuaderno abierto: exporta a PDF desde ahí.";
    }
  }

  function togglePresentation(): void {
    lab.setPreferences(
      {
        presentationMode: !lab.preferences.presentationMode,
        fontScale: lab.preferences.presentationMode ? 1 : 1.15,
      },
      "Modo presentación",
    );
  }

  function bindKeyboard(): () => void {
    const handler = (event: KeyboardEvent): void => {
      const target = event.target as HTMLElement | null;
      const typing = target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable);
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        paletteOpen = !paletteOpen;
        return;
      }
      if (typing) return;
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "z") {
        event.preventDefault();
        if (event.shiftKey) lab.redo();
        else lab.undo();
        return;
      }
      if (!isMac && (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "y") {
        event.preventDefault();
        lab.redo();
        return;
      }
      switch (event.key) {
        case " ":
          event.preventDefault();
          lab.animating = !lab.animating;
          break;
        case "ArrowRight":
          if (event.shiftKey) stepFrame();
          break;
        case "r":
        case "R":
          resetTime();
          break;
        case "1":
          lab.tab = "cell";
          break;
        case "2":
          lab.tab = "iv";
          break;
        case "3":
          lab.tab = "kmax";
          break;
        case "4":
          lab.tab = "experiment";
          break;
        case "F11":
          event.preventDefault();
          togglePresentation();
          break;
        case "?":
          event.preventDefault();
          shortcutsOpen = !shortcutsOpen;
          break;
        case "Escape":
          if (paletteOpen) {
            paletteOpen = false;
            break;
          }
          if (shortcutsOpen) {
            shortcutsOpen = false;
            break;
          }
          if (drawer) {
            drawer = null;
            break;
          }
          if (lab.tutorial.active) lab.tutorial = { active: false, step: 0 };
          break;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }

  function paletteAction(id: string): void {
    if (id.startsWith("scenario:")) {
      applyScenario(id.split(":")[1] as (typeof SCENARIOS)[number]["id"]);
    } else if (id === "save") void handleSave();
    else if (id === "open") void handleOpen();
    else if (id.startsWith("export:")) void handleExport(id.split(":")[1] as "csv" | "png" | "svg" | "pdf");
    else if (id === "presentation") togglePresentation();
  }

  // arrastre de paneles laterales
  function startDrag(side: "left" | "right", event: PointerEvent): void {
    dragging = {
      side,
      startX: event.clientX,
      startWidth: side === "left" ? lab.preferences.controlsPanelWidth : lab.preferences.readoutsPanelWidth,
    };
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function moveDrag(event: PointerEvent): void {
    if (!dragging) return;
    const dx = event.clientX - dragging.startX;
    const next = dragging.side === "left" ? dragging.startWidth + dx : dragging.startWidth - dx;
    const clamped = Math.min(440, Math.max(240, next));
    if (dragging.side === "left") {
      lab.preferences = { ...lab.preferences, controlsPanelWidth: clamped };
    } else {
      lab.preferences = { ...lab.preferences, readoutsPanelWidth: clamped };
    }
  }

  const presentation = $derived(lab.preferences.presentationMode);
  const showControls = $derived(lab.preferences.controlsPanelOpen && !presentation);
  const showReadouts = $derived(lab.preferences.readoutsPanelOpen && !presentation);
  const gridColumns = $derived(
    narrow
      ? "minmax(0, 1fr)"
      : showControls && showReadouts
        ? "var(--controls-w) 6px minmax(0, 1fr) 6px var(--readouts-w)"
        : showControls
          ? "var(--controls-w) 6px minmax(0, 1fr)"
          : showReadouts
            ? "minmax(0, 1fr) 6px var(--readouts-w)"
            : "minmax(0, 1fr)",
  );

  const tabs = $derived([
    { id: "cell" as const, label: s.tabsCell },
    { id: "iv" as const, label: s.tabsIV },
    { id: "kmax" as const, label: s.tabsKmax },
    { id: "experiment" as const, label: s.tabsExperiment },
  ]);
</script>

<div
  class="app-shell"
  class:presentation
  style:--controls-w="{lab.preferences.controlsPanelWidth}px"
  style:--readouts-w="{lab.preferences.readoutsPanelWidth}px"
>
  <div class="scenario-accent" style:background={scenarioAccent} aria-hidden="true"></div>
  <TopBar
    scenarioId={activeScenario}
    scenarioAccent={scenarioAccent}
    onScenario={applyScenario}
    onSave={handleSave}
    onOpen={handleOpen}
    onExport={handleExport}
    onPalette={() => (paletteOpen = true)}
    onPresentation={togglePresentation}
    onShortcuts={() => (shortcutsOpen = !shortcutsOpen)}
  />

  <div class="workspace" style:grid-template-columns={gridColumns}>
    {#if !narrow && showControls}
      <div class="side left">
        <ControlsPanel />
      </div>
      <div
        class="resize-handle"
        role="separator"
        aria-orientation="vertical"
        aria-label="Redimensionar panel de controles"
        onpointerdown={(e) => startDrag("left", e)}
        onpointermove={moveDrag}
        onpointerup={() => (dragging = null)}
      ></div>
    {/if}

    <main class="center">
      <div class="tabs-row">
        <div class="tabs" role="tablist" aria-label="Vistas">
          {#each tabs as tabDef (tabDef.id)}
            <button
              class="tab"
              class:active={lab.tab === tabDef.id}
              role="tab"
              aria-selected={lab.tab === tabDef.id}
              onclick={() => { lab.tab = tabDef.id; drawer = null; }}
              onkeydown={(e) => {
                if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
                  const index = tabs.findIndex((t) => t.id === lab.tab);
                  const next = (index + (e.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
                  lab.tab = tabs[next].id;
                  drawer = null;
                }
              }}
            >{tabDef.label}</button>
          {/each}
        </div>
        <div class="tab-actions">
          <button
            class="btn ghost icon"
            class:active={narrow ? drawer === "controls" : lab.preferences.controlsPanelOpen}
            onclick={() => togglePanel("controls")}
            title={s.controls}
            aria-label={s.controls}
            aria-pressed={narrow ? drawer === "controls" : lab.preferences.controlsPanelOpen}
          >
            <svg viewBox="0 0 20 20" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true">
              <path d="M3 6h14M3 14h14" />
              <circle cx="8" cy="6" r="2" fill="currentColor" stroke="none" />
              <circle cx="13" cy="14" r="2" fill="currentColor" stroke="none" />
            </svg>
          </button>
          <button
            class="btn ghost icon"
            class:active={narrow ? drawer === "readouts" : lab.preferences.readoutsPanelOpen}
            onclick={() => togglePanel("readouts")}
            title={s.readouts}
            aria-label={s.readouts}
            aria-pressed={narrow ? drawer === "readouts" : lab.preferences.readoutsPanelOpen}
          >
            <svg viewBox="0 0 20 20" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true">
              <path d="M4 16V9M10 16V4M16 16v-5" />
            </svg>
          </button>
          <span class="tab-divider" aria-hidden="true"></span>
          <button class="btn ghost icon" onclick={() => (lab.animating = !lab.animating)} title="Espacio" aria-label="Pausar animación">
            {lab.animating ? "❚❚" : "▶"}
          </button>
          <button
            class="btn ghost"
            class:active={lab.slowMotion}
            onclick={() => (lab.slowMotion = !lab.slowMotion)}
            title="Cámara lenta (×0.25)"
          >0.25×</button>
          <button class="btn ghost icon" onclick={resetTime} title="R" aria-label="Reiniciar tiempo">⟲</button>
        </div>
      </div>

      <div class="stage">
        {#if lab.tab === "cell"}
          <Scene />
          {#if showNotebook}
            <div class="notebook-slot rise">
              <Notebook />
            </div>
          {/if}
        {:else}
          <div class="scroll-stage scroll">
            <PanelGraphs tab={lab.tab} />
          </div>
        {/if}
      </div>
    </main>

    {#if !narrow && showReadouts}
      <div
        class="resize-handle"
        role="separator"
        aria-orientation="vertical"
        aria-label="Redimensionar panel de lecturas"
        onpointerdown={(e) => startDrag("right", e)}
        onpointermove={moveDrag}
        onpointerup={() => (dragging = null)}
      ></div>
      <div class="side right">
        <ReadoutsPanel />
      </div>
    {/if}
  </div>

  {#if narrow && drawer}
    <button class="drawer-scrim" aria-label={s.closePanel} onclick={() => (drawer = null)}></button>
    <div
      class="drawer"
      class:right={drawer === "readouts"}
      role="dialog"
      aria-modal="true"
      aria-label={drawer === "controls" ? s.controls : s.readouts}
    >
      <div class="drawer-head">
        <span>{drawer === "controls" ? s.controls : s.readouts}</span>
        <button class="btn ghost icon" onclick={() => (drawer = null)} aria-label={s.closePanel} title={s.closePanel}>✕</button>
      </div>
      <div class="drawer-body">
        {#if drawer === "controls"}
          <ControlsPanel />
        {:else}
          <ReadoutsPanel />
        {/if}
      </div>
    </div>
  {/if}

  <StatusBar />
  <CommandPalette bind:open={paletteOpen} onAction={paletteAction} />
  <ShortcutsOverlay bind:open={shortcutsOpen} />
  <Tutorial />
</div>

<style>
  .app-shell {
    position: relative;
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100%;
    min-height: 480px;
    overflow: hidden;
  }
  .scenario-accent {
    position: absolute;
    inset: 0 0 auto 0;
    height: 2px;
    pointer-events: none;
    opacity: 0.9;
    transition: background var(--dur-2) var(--ease);
    z-index: 5;
  }
  .workspace {
    display: grid;
    grid-template-columns: var(--controls-w) 6px minmax(0, 1fr) 6px var(--readouts-w);
    min-height: 0;
  }
  .app-shell.presentation .workspace {
    grid-template-columns: minmax(0, 1fr);
  }
  .side {
    min-height: 0;
    min-width: 0;
    /* Red de seguridad: si el contenido excede el alto, se desplaza en vez de
       recortarse contra la barra de estado. */
    overflow-y: auto;
    background: color-mix(in srgb, var(--panel-solid) 55%, transparent);
    border-color: var(--line);
  }
  .side.left {
    border-right: 0;
  }
  .resize-handle {
    cursor: col-resize;
    background: transparent;
    transition: background var(--dur-1) var(--ease);
    position: relative;
  }
  .resize-handle:hover {
    background: var(--wl-soft);
  }
  .resize-handle::after {
    content: "";
    position: absolute;
    left: 2px;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--line);
  }
  .center {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  .tabs-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px 0;
  }
  .tabs {
    display: flex;
    gap: 2px;
    background: var(--panel-2);
    padding: 3px;
    border-radius: var(--r-sm);
    border: 1px solid var(--line);
  }
  .tab {
    border: 0;
    background: transparent;
    color: var(--text-2);
    padding: 5px 12px;
    border-radius: 6px;
    font-size: 0.82em;
    font-weight: 500;
    transition:
      background var(--dur-1) var(--ease),
      color var(--dur-1) var(--ease);
  }
  .tab.active {
    background: var(--wl-soft);
    color: var(--text);
    box-shadow: inset 0 0 0 1px var(--wl-strong);
  }
  .tab-actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .tab-divider {
    width: 1px;
    height: 20px;
    margin: 0 4px;
    background: var(--line);
  }
  .stage {
    flex: 1;
    min-height: 0;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .stage > :global(.scene-wrap) {
    flex: 1;
  }
  .scroll-stage {
    flex: 1;
    min-height: 0;
    border-radius: var(--r-lg);
    border: 1px solid var(--line);
    background: color-mix(in srgb, var(--panel-solid) 40%, transparent);
  }
  .notebook-slot {
    max-height: 40%;
    overflow-y: auto;
  }
  /* Drawers para ventanas estrechas (1024–1180 px): los paneles se abren
     encima del escenario en lugar de robarle ancho. */
  .drawer-scrim {
    position: fixed;
    inset: 0;
    z-index: 40;
    border: 0;
    padding: 0;
    background: rgba(3, 7, 18, 0.55);
    backdrop-filter: blur(1.5px);
    cursor: pointer;
    animation: scrim-in var(--dur-1) var(--ease);
  }
  .drawer {
    position: fixed;
    z-index: 41;
    top: 0;
    bottom: 0;
    left: 0;
    display: flex;
    flex-direction: column;
    width: min(380px, 88vw);
    background: var(--panel-solid);
    border-right: 1px solid var(--line-strong);
    box-shadow: 12px 0 40px rgba(0, 0, 0, 0.45);
    animation: drawer-in var(--dur-2) var(--ease);
  }
  .drawer.right {
    left: auto;
    right: 0;
    border-right: 0;
    border-left: 1px solid var(--line-strong);
    box-shadow: -12px 0 40px rgba(0, 0, 0, 0.45);
  }
  .drawer-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-2);
    padding: 10px 12px;
    border-bottom: 1px solid var(--line);
    font-size: 0.84em;
    font-weight: 600;
    color: var(--text-2);
  }
  .drawer-body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .drawer-body > :global(*) {
    min-height: 0;
  }
  @keyframes drawer-in {
    from {
      transform: translateX(-12px);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
  @keyframes scrim-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .drawer,
    .drawer-scrim {
      animation: none;
    }
  }
  @media (max-width: 1180px) {
    .workspace {
      grid-template-columns: minmax(0, 1fr);
    }
    .side,
    .resize-handle {
      display: none;
    }
  }
</style>
