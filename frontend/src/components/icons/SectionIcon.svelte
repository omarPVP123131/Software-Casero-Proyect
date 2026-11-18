<script lang="ts">
  // Iconos de sección como SVG inline (no glifos de fuente): se ven idénticos
  // en Linux, Windows y macOS, donde los símbolos Unicode caían a fuentes
  // distintas (◈ ≋ ⎓ ⏦ ⚗) y cambiaban de tamaño y trazo.
  // Trazo 1.7 px, caja 24×24, hereda currentColor.
  type Name = "material" | "light" | "voltage" | "current" | "realism";

  let { name, size = 15 }: { name: Name; size?: number } = $props();
</script>

<svg
  class="sec-icon"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.7"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  {#if name === "material"}
    <!-- red cristalina / retícula del cátodo -->
    <path d="M12 2.8 20.2 7.4v9.2L12 21.2 3.8 16.6V7.4z" />
    <path d="M12 7.6 16.4 10v4.2L12 16.5 7.6 14.2V10z" />
  {:else if name === "light"}
    <!-- onda luminosa (se colorea con la longitud de onda activa) -->
    <path d="M2 12c2.2-4.6 4.4-6.9 6.6-6.9S13 7.4 15.2 12s4.4 6.9 6.6 6.9" />
    <path d="M2 16.8c1.6-3.2 3.2-4.8 4.8-4.8s3.2 1.6 4.8 4.8" />
  {:else if name === "voltage"}
    <!-- pila: carcasa y electrodos -->
    <rect x="2.6" y="7.4" width="15.6" height="9.2" rx="2.2" />
    <path d="M21.4 10.6v2.8" />
    <path d="M6.2 10.4v3.2M9.6 10.4v3.2" />
  {:else if name === "current"}
    <!-- amperímetro / oscilación de corriente -->
    <circle cx="12" cy="12" r="9.2" />
    <path d="M7.6 14.4c1.1-3.6 2.1-5.4 3.1-5.4s2 1.8 3.1 5.4" />
    <path d="M15.6 12.6h3" />
  {:else}
    <!-- matraz: modelos de realismo -->
    <path d="M9.2 3h5.6v4.1l4.1 9.4A2.4 2.4 0 0 1 16.7 20H7.3a2.4 2.4 0 0 1-2.2-3.5l4.1-9.4z" />
    <path d="M7.4 14.6h9.2" />
    <path d="M9 3h6" />
  {/if}
</svg>

<style>
  .sec-icon {
    flex: 0 0 auto;
    display: block;
    opacity: 0.95;
  }
</style>
