<script lang="ts">
  // Barra de estado: motor OK · modo ideal/ruido · fps · unidades.
  import { lab } from "../lib/store.svelte";
  import { t } from "../lib/i18n";
  import { physicsState } from "../lib/physics";

  const s = $derived(t(lab.preferences.lang));
  let fps = $state(60);

  $effect(() => {
    const id = setInterval(() => {
      fps = (window as unknown as { __photolabFps?: number }).__photolabFps ?? fps;
    }, 700);
    return () => clearInterval(id);
  });

  const units = $derived(
    lab.preferences.lang === "en"
      ? "SI · eV readouts · λ nm"
      : "SI · lecturas en eV · λ en nm",
  );
</script>

<footer class="statusbar small" role="status">
  <span class="engine">
    <span class="pulse-dot" aria-hidden="true"></span>
    {s.engineOk} · {physicsState.backend === "wasm" ? "WASM" : "Tauri IPC"}
  </span>
  <span class="chip mono">{lab.experiment.experimentalMode ? `ruido ±${lab.experiment.noisePercent.toFixed(1)} %` : "ideal"}</span>
  <span class="chip mono">{fps} fps</span>
  <span class="chip mono">{lab.preferences.speedScale === "compressed" ? "v:√(K/1 eV) 0.5–2.2" : "v: log real"}</span>
  <span class="spacer"></span>
  {#if lab.status}
    <span class="status-msg" title={lab.status}>{lab.status}</span>
  {/if}
  <span class="muted">{units}</span>
</footer>

<style>
  .statusbar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 12px;
    border-top: 1px solid var(--line);
    background: var(--panel-solid);
    color: var(--text-2);
    font-size: 0.72em;
    min-height: 30px;
  }
  .engine {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    color: var(--text-2);
  }
  .chip {
    padding: 1px 8px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: var(--panel-2);
    font-size: 0.94em;
  }
  .spacer {
    flex: 1;
  }
  .status-msg {
    max-width: 46ch;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text);
  }
</style>
