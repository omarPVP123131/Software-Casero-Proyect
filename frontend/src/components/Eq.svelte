<script lang="ts">
  import { renderEq } from "../lib/eq";
  let { latex, display = false, block = false }: { latex: string; display?: boolean; block?: boolean } = $props();
  const html = $derived(renderEq(latex, display));
</script>

{#if block}
  <div class="eq-block" aria-label="Ecuación">{@html html}</div>
{:else}
  <span class="eq-inline" aria-label="Ecuación">{@html html}</span>
{/if}

<style>
  .eq-inline :global(.katex) {
    font-size: 1em;
  }
  .eq-block {
    padding: 8px 10px;
    border-radius: var(--r-sm);
    background: var(--panel-2);
    border: 1px solid var(--line);
    overflow-x: auto;
  }
  .eq-block :global(.katex) {
    font-size: 0.95em;
    color: var(--text);
  }
</style>
