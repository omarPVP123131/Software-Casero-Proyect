<script lang="ts">
  // Barra superior: logo · escenario · sesión · atajos · tema.
  import { lab } from "../lib/store.svelte";
  import { t } from "../lib/i18n";
  import { paletteShortcut, redoShortcut, undoShortcut } from "../lib/platform";
  import { wavelengthToColor } from "../lib/spectrum";

  let {
    scenarioId = null,
    scenarioAccent = null,
    onScenario,
    onSave,
    onOpen,
    onExport,
    onPalette,
    onPresentation,
    onShortcuts,
  }: {
    /** Escenario activo (o null) y su color de sello. */
    scenarioId?: string | null;
    scenarioAccent?: string | null;
    onScenario: (id: "threshold" | "intensity" | "stopping" | "measure_h") => void;
    onSave: () => void;
    onOpen: () => void;
    onExport: (kind: "csv" | "png" | "svg" | "pdf") => void;
    onPalette: () => void;
    onPresentation: () => void;
    onShortcuts: () => void;
  } = $props();

  const s = $derived(t(lab.preferences.lang));
  const accent = $derived(wavelengthToColor(lab.controls.wavelengthNm).css);
  let exportOpen = $state(false);
  let scenarioOpen = $state(false);
  let barEl = $state<HTMLElement | null>(null);

  function closeMenus(event: PointerEvent): void {
    if (barEl && !barEl.contains(event.target as Node)) {
      scenarioOpen = false;
      exportOpen = false;
    }
  }

  function closeOnEscape(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      scenarioOpen = false;
      exportOpen = false;
    }
  }

  const scenarios = $derived([
    { id: "threshold" as const, label: s.scenarioThreshold },
    { id: "intensity" as const, label: s.scenarioIntensity },
    { id: "stopping" as const, label: s.scenarioStopping },
    { id: "measure_h" as const, label: s.scenarioMeasureH },
  ]);
</script>

<svelte:window onpointerdown={closeMenus} onkeydown={closeOnEscape} />

<header class="topbar" bind:this={barEl} style:--wl={accent}>
  <div class="brand">
    <svg viewBox="0 0 32 32" width="26" height="26" aria-hidden="true">
      <rect width="32" height="32" rx="8" fill="var(--panel-2)" stroke="var(--line-strong)" />
      <path d="M5 21 L27 11" stroke="var(--wl)" stroke-width="2.2" stroke-linecap="round" />
      <circle cx="15" cy="18" r="1.5" fill="#67E8F9" />
      <circle cx="20" cy="15.5" r="1.5" fill="#67E8F9" />
      <circle cx="24" cy="13.5" r="1.5" fill="#67E8F9" />
    </svg>
    <div class="brand-text">
      <span class="brand-name">{s.appName}</span>
      <span class="brand-tag small muted">{s.appTagline}</span>
    </div>
  </div>

  <div class="sep"></div>

  <div class="menu-wrap">
    <button class="btn ghost" onclick={() => { scenarioOpen = !scenarioOpen; exportOpen = false; }} aria-expanded={scenarioOpen} aria-haspopup="menu">
      ⚗ {s.scenario}
      <span class="caret">▾</span>
    </button>
    {#if scenarioId}
      <span
        class="scenario-chip mono small"
        style:--chip={scenarioAccent}
        title={scenarios.find((x) => x.id === scenarioId)?.label}
      >
        <i aria-hidden="true"></i>{scenarios.find((x) => x.id === scenarioId)?.label}
      </span>
    {/if}
    {#if scenarioOpen}
      <div class="menu panel rise" role="menu">
        {#each scenarios as scenario (scenario.id)}
          <button
            role="menuitem"
            class="menu-item"
            onclick={() => {
              onScenario(scenario.id);
              scenarioOpen = false;
            }}>{scenario.label}</button>
        {/each}
      </div>
    {/if}
  </div>

  <button class="btn ghost" onclick={onPalette} title={`${s.commandHint} (${paletteShortcut})`} aria-label={s.commandPalette}>
    {paletteShortcut} <span class="small muted">{s.commandPalette}</span>
  </button>

  <div class="sep"></div>

  <div class="menu-wrap">
    <button class="btn ghost" onclick={() => { exportOpen = !exportOpen; scenarioOpen = false; }} aria-expanded={exportOpen} aria-haspopup="menu">
      {s.session}
      <span class="caret">▾</span>
    </button>
    {#if exportOpen}
      <div class="menu panel rise" role="menu">
        <button role="menuitem" class="menu-item" onclick={() => { onSave(); exportOpen = false; }}>{s.sessionSave}</button>
        <button role="menuitem" class="menu-item" onclick={() => { onOpen(); exportOpen = false; }}>{s.sessionOpen}</button>
        <div class="menu-sep"></div>
        <button role="menuitem" class="menu-item" onclick={() => { onExport("csv"); exportOpen = false; }}>{s.exportCsv}</button>
        <button role="menuitem" class="menu-item" onclick={() => { onExport("png"); exportOpen = false; }}>{s.exportPng}</button>
        <button role="menuitem" class="menu-item" onclick={() => { onExport("svg"); exportOpen = false; }}>{s.exportSvg}</button>
        <button role="menuitem" class="menu-item" onclick={() => { onExport("pdf"); exportOpen = false; }}>{s.exportPdf}</button>
      </div>
    {/if}
  </div>

  <button class="btn ghost icon" onclick={onPresentation} title="F11" aria-label={s.presentation}>
    <svg viewBox="0 0 20 20" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" aria-hidden="true">
      <path d="M3 7V3h4M13 3h4v4M17 13v4h-4M7 17H3v-4" />
    </svg>
  </button>

  <div class="spacer"></div>

  <div class="undo-redo">
    <button class="btn ghost icon" onclick={() => lab.undo()} title={undoShortcut} aria-label={s.undo}>↶</button>
    <button class="btn ghost icon" onclick={() => lab.redo()} title={redoShortcut} aria-label={s.redo}>↷</button>
  </div>

  <div class="theme-switch" role="group" aria-label={s.theme}>
    {#each [["dark", "◐"], ["light", "☀"], ["contrast", "◑"]] as [theme, icon] (theme)}
      <button
        class="btn ghost icon"
        class:active={lab.preferences.theme === theme}
        onclick={() => lab.setPreferences({ theme: theme as "dark" | "light" | "contrast" }, "Tema " + theme)}
        aria-label={theme}
        aria-pressed={lab.preferences.theme === theme}
      >{icon}</button>
    {/each}
  </div>

  <button
    class="btn ghost icon"
    onclick={onShortcuts}
    title="Atajos (?)"
    aria-label="Ver atajos de teclado"
  >?</button>

  <button
    class="btn ghost lang"
    onclick={() => lab.setPreferences({ lang: lab.preferences.lang === "es" ? "en" : "es" }, "Idioma")}
    aria-label="Cambiar idioma"
  >{lab.preferences.lang.toUpperCase()}</button>

  <div class="font-scale">
    <button class="btn ghost icon" onclick={() => lab.setPreferences({ fontScale: Math.max(0.85, lab.preferences.fontScale - 0.1) }, "Tamaño de fuente")} aria-label="Reducir texto">A−</button>
    <button class="btn ghost icon" onclick={() => lab.setPreferences({ fontScale: Math.min(1.35, lab.preferences.fontScale + 0.1) }, "Tamaño de fuente")} aria-label="Aumentar texto">A+</button>
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--line);
    background: color-mix(in srgb, var(--panel-solid) 72%, transparent);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    position: relative;
    z-index: 20;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .brand-text {
    display: flex;
    flex-direction: column;
    line-height: 1.15;
  }
  .brand-name {
    font-weight: 700;
    letter-spacing: 0.02em;
  }
  .brand-tag {
    font-size: 0.66em;
  }
  .sep {
    width: 1px;
    height: 22px;
    background: var(--line);
    margin: 0 4px;
  }
  .spacer {
    flex: 1;
  }
  .caret {
    font-size: 0.7em;
    opacity: 0.7;
  }
  .menu-wrap {
    position: relative;
  }
  .menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    min-width: 240px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    z-index: 60;
    box-shadow: var(--el-3);
  }
  .menu-item {
    text-align: left;
    padding: 8px 10px;
    border: 0;
    background: transparent;
    border-radius: var(--r-sm);
    color: var(--text-2);
    font-size: 0.86em;
  }
  .menu-item:hover {
    background: var(--wl-soft);
    color: var(--text);
  }
  .menu-sep {
    height: 1px;
    background: var(--line);
    margin: 4px 2px;
  }
  .undo-redo,
  .theme-switch,
  .font-scale {
    display: flex;
    gap: 0;
  }
  .btn.active {
    color: var(--wl);
  }
  .scenario-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 9px;
    border-radius: 999px;
    border: 1px solid color-mix(in srgb, var(--chip, #A78BFA) 45%, transparent);
    background: color-mix(in srgb, var(--chip, #A78BFA) 12%, transparent);
    color: var(--text);
    white-space: nowrap;
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .scenario-chip i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--chip, #A78BFA);
    flex: none;
  }
  .lang {
    font-weight: 600;
    font-size: 0.78em;
    min-width: 34px;
  }
</style>
