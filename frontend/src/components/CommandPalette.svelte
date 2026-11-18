<script lang="ts">
  // Paleta de comandos (Ctrl/Cmd+K): acciones, materiales y ajustes.
  import { lab } from "../lib/store.svelte";
  import type { MaterialChoice } from "../lib/types";

  let { open = $bindable(false), onAction }: { open?: boolean; onAction: (id: string) => void } = $props();
  let query = $state("");
  let active = $state(0);

  interface Command {
    id: string;
    label: string;
    hint?: string;
    group: string;
    run: () => void;
  }

  const commands = $derived.by<Command[]>(() => {
    const list: Command[] = [
      { id: "scenario-threshold", label: "Escenario: Umbral del sodio", group: "Escenarios", run: () => onAction("scenario:threshold") },
      { id: "scenario-intensity", label: "Escenario: Intensidad vs energía", group: "Escenarios", run: () => onAction("scenario:intensity") },
      { id: "scenario-stopping", label: "Escenario: Potencial de frenado", group: "Escenarios", run: () => onAction("scenario:stopping") },
      { id: "scenario-measure", label: "Escenario: Medir h", group: "Escenarios", run: () => onAction("scenario:measure_h") },
      { id: "tab-cell", label: "Ir a la celda", group: "Vistas", run: () => (lab.tab = "cell") },
      { id: "tab-iv", label: "Ir a la gráfica I–V", group: "Vistas", run: () => (lab.tab = "iv") },
      { id: "tab-kmax", label: "Ir a Kmax(λ)", group: "Vistas", run: () => (lab.tab = "kmax") },
      { id: "tab-experiment", label: "Ir al experimento V₀–f", group: "Vistas", run: () => (lab.tab = "experiment") },
      { id: "sweep", label: "Barrido automático V₀–f", group: "Acciones", run: () => { lab.tab = "experiment"; void lab.autoSweep(6); } },
      { id: "save", label: "Guardar sesión", group: "Acciones", run: () => onAction("save") },
      { id: "open", label: "Abrir sesión", group: "Acciones", run: () => onAction("open") },
      { id: "export-csv", label: "Exportar experimento CSV", group: "Acciones", run: () => onAction("export:csv") },
      { id: "export-png", label: "Exportar canvas PNG", group: "Acciones", run: () => onAction("export:png") },
      { id: "tutorial", label: "Iniciar tutorial guiado", group: "Ayuda", run: () => (lab.tutorial = { active: true, step: 0 }) },
      { id: "presentation", label: "Modo presentación", group: "Vistas", run: () => onAction("presentation") },
      { id: "theme-dark", label: "Tema oscuro", group: "Ajustes", run: () => lab.setPreferences({ theme: "dark" }, "Tema") },
      { id: "theme-light", label: "Tema claro", group: "Ajustes", run: () => lab.setPreferences({ theme: "light" }, "Tema") },
      { id: "theme-contrast", label: "Alto contraste", group: "Ajustes", run: () => lab.setPreferences({ theme: "contrast" }, "Tema") },
      { id: "speed-real", label: "Escala de velocidad real logarítmica", group: "Ajustes", run: () => lab.setPreferences({ speedScale: "real_log" }, "Escala de velocidad") },
      { id: "speed-compressed", label: "Escala de velocidad comprimida", group: "Ajustes", run: () => lab.setPreferences({ speedScale: "compressed" }, "Escala de velocidad") },
    ];
    for (const m of lab.catalog) {
      list.push({
        id: `material-${m.id}`,
        label: `Material: ${m.nameEs} (${m.symbol}) · Φ ${m.workFunctionEv.toFixed(2)} eV`,
        group: "Materiales",
        run: () => lab.update((c) => { c.material = m.id as MaterialChoice; }, `Material → ${m.nameEs}`),
      });
    }
    return list;
  });

  const filtered = $derived(
    query.trim() === ""
      ? commands
      : commands.filter((c) => c.label.toLowerCase().includes(query.toLowerCase())),
  );

  $effect(() => {
    if (open) {
      query = "";
      active = 0;
    }
  });

  function run(command: Command | undefined): void {
    if (!command) return;
    command.run();
    open = false;
  }

  function onKey(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      open = false;
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      active = Math.min(filtered.length - 1, active + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      active = Math.max(0, active - 1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      run(filtered[active]);
    }
  }
</script>

{#if open}
  <div class="backdrop" role="presentation" onclick={() => (open = false)} onkeydown={(e) => e.key === "Escape" && (open = false)}>
    <div
      class="palette panel"
      role="dialog"
      aria-modal="true"
      aria-label="Paleta de comandos"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
    >
      <input
        class="search"
        type="text"
        placeholder="Buscar acción, material o ajuste…"
        bind:value={query}
        onkeydown={onKey}
        aria-label="Buscar comando"
      />
      <div class="results" role="listbox" aria-label="Resultados">
        {#each filtered as command, i (command.id)}
          <button
            class="result"
            class:active={i === active}
            role="option"
            aria-selected={i === active}
            onmouseenter={() => (active = i)}
            onclick={() => run(command)}
          >
            <span class="group small muted">{command.group}</span>
            <span class="label">{command.label}</span>
          </button>
        {:else}
          <div class="empty small muted">Sin resultados</div>
        {/each}
      </div>
      <div class="footer small muted">
        <span><span class="kbd">↑</span><span class="kbd">↓</span> navegar</span>
        <span><span class="kbd">↵</span> ejecutar</span>
        <span><span class="kbd">esc</span> cerrar</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(3, 6, 12, 0.55);
    backdrop-filter: blur(6px);
    z-index: 100;
    display: flex;
    justify-content: center;
    padding-top: 12vh;
  }
  .palette {
    width: min(560px, 92vw);
    max-height: 62vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: var(--el-3);
  }
  .search {
    border: 0;
    border-bottom: 1px solid var(--line);
    border-radius: 0;
    background: transparent;
    padding: 14px 16px;
    font-size: 0.98em;
  }
  .search:focus {
    border-color: var(--wl);
  }
  .results {
    overflow-y: auto;
    padding: 6px;
  }
  .result {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    text-align: left;
    border: 0;
    background: transparent;
    color: var(--text-2);
    border-radius: var(--r-sm);
    padding: 8px 10px;
    font-size: 0.88em;
  }
  .result.active {
    background: var(--wl-soft);
    color: var(--text);
  }
  .group {
    min-width: 84px;
    font-size: 0.72em;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .result .label {
    flex: 1;
  }
  .empty {
    padding: 16px;
    text-align: center;
  }
  .footer {
    display: flex;
    gap: 16px;
    padding: 8px 14px;
    border-top: 1px solid var(--line);
  }
</style>
