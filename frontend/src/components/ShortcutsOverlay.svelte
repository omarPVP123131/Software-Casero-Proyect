<script lang="ts">
  // Overlay de atajos (?). Se cierra con ? , Esc o clic en el fondo.
  // Está pensado como chuleta: agrupado por tarea, no por tecla.
  import { lab } from "../lib/store.svelte";
  import { t } from "../lib/i18n";

  let { open = $bindable(false) }: { open?: boolean } = $props();
  let sheetEl = $state<HTMLDivElement | null>(null);

  // Llevar el foco al panel al abrirlo (lectores de pantalla y teclado).
  $effect(() => {
    if (open) sheetEl?.focus();
  });

  const s = $derived(t(lab.preferences.lang));
  const es = $derived(lab.preferences.lang !== "en");

  const groups = $derived([
    {
      title: es ? "Reproducción" : "Playback",
      rows: [
        ["Espacio", es ? "Pausa / reanuda la animación" : "Pause / resume the animation"],
        ["R", es ? "Reinicia el tiempo" : "Reset time"],
        ["Shift + →", es ? "Avanza un fotograma" : "Step one frame"],
      ],
    },
    {
      title: es ? "Vistas" : "Views",
      rows: [
        ["1 … 4", es ? "Celda · I–V · Kmax(λ) · Experimento V₀–f" : "Cell · I–V · Kmax(λ) · V₀–f experiment"],
        ["F11", es ? "Modo presentación" : "Presentation mode"],
      ],
    },
    {
      title: es ? "Edición" : "Editing",
      rows: [
        [es ? "Ctrl/⌘ + Z" : "Ctrl/⌘ + Z", es ? "Deshacer" : "Undo"],
        [es ? "Ctrl/⌘ + Shift + Z" : "Ctrl/⌘ + Shift + Z", es ? "Rehacer" : "Redo"],
        ["Ctrl/⌘ + K", es ? "Paleta de comandos" : "Command palette"],
      ],
    },
    {
      title: es ? "Panel y ayuda" : "Panel & help",
      rows: [
        ["?", es ? "Esta lista" : "This list"],
        ["Esc", es ? "Cierra diálogos" : "Close dialogs"],
      ],
    },
  ]);

  function onKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") open = false;
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if open}
  <div class="backdrop" role="presentation" onclick={() => (open = false)}>
    <div
      class="sheet"
      bind:this={sheetEl}
      role="dialog"
      aria-modal="true"
      aria-labelledby="shortcuts-title"
      tabindex="-1"
    >
      <header>
        <h2 id="shortcuts-title">{es ? "Atajos de teclado" : "Keyboard shortcuts"}</h2>
        <button class="btn ghost icon" onclick={() => (open = false)} aria-label={es ? "Cerrar" : "Close"}>✕</button>
      </header>
      <div class="grid">
        {#each groups as group (group.title)}
          <section>
            <h3>{group.title}</h3>
            {#each group.rows as row (row[0])}
              <div class="row">
                <kbd>{row[0]}</kbd>
                <span>{row[1]}</span>
              </div>
            {/each}
          </section>
        {/each}
      </div>
      <footer class="small muted">
        {es ? "La app no captura atajos mientras escribes en un campo." : "Shortcuts are ignored while typing in a field."}
      </footer>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 60;
    background: rgba(4, 8, 16, 0.62);
    backdrop-filter: blur(3px);
    display: grid;
    place-items: center;
    padding: 24px;
  }
  .sheet {
    width: min(720px, 100%);
    max-height: min(86vh, 620px);
    overflow-y: auto;
    background: var(--panel-solid);
    border: 1px solid var(--line-strong);
    border-radius: var(--r-lg);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.45);
    padding: 18px 20px 14px;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }
  h2 {
    margin: 0;
    font-size: 1.02em;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  h3 {
    margin: 0 0 6px;
    font-size: 0.76em;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 16px 22px;
  }
  .row {
    display: grid;
    grid-template-columns: minmax(84px, auto) 1fr;
    gap: 10px;
    align-items: baseline;
    padding: 3px 0;
    font-size: 0.84em;
    border-bottom: 1px dashed color-mix(in srgb, var(--line) 60%, transparent);
  }
  .row:last-child {
    border-bottom: 0;
  }
  kbd {
    font-family: "JetBrains Mono", ui-monospace, monospace;
    font-size: 0.92em;
    padding: 2px 6px;
    border-radius: 5px;
    background: var(--panel-2);
    border: 1px solid var(--line);
    border-bottom-width: 2px;
    white-space: nowrap;
    justify-self: start;
  }
  footer {
    margin-top: 14px;
    font-size: 0.78em;
  }
</style>
