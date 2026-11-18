<script lang="ts">
  // Tutorial guiado paso a paso con resaltado de controles, preguntas de
  // predicción y navegación accesible. Las preguntas se responden ANTES de ver
  // el resultado (botón "ver resultado").
  import { lab } from "../lib/store.svelte";
  import { tutorialSteps } from "../lib/causal";

  const steps = $derived(tutorialSteps(lab.preferences.lang));
  const step = $derived(steps[lab.tutorial.step]);
  let revealed = $state(false);
  let answer: string | null = $state(null);
  const total = $derived(steps.length);

  $effect(() => {
    // resetea la predicción al cambiar de paso
    lab.tutorial.step;
    revealed = false;
    answer = null;
  });

  function next(): void {
    if (lab.tutorial.step < total - 1) {
      lab.tutorial = { active: true, step: lab.tutorial.step + 1 };
    } else {
      lab.tutorial = { active: false, step: 0 };
      lab.pushHistory("Tutorial completado");
    }
  }

  function prev(): void {
    lab.tutorial = { active: true, step: Math.max(0, lab.tutorial.step - 1) };
  }

  function applyScenarioFor(currentStep: number): void {
    if (currentStep === 0) {
      lab.update((c) => {
        c.material = "sodium";
        c.wavelengthNm = 600;
        c.intensityPercent = 60;
        c.appliedVoltageV = 0;
      }, "Tutorial · paso umbral");
    } else if (currentStep === 1) {
      lab.update((c) => { c.wavelengthNm = 400; c.intensityPercent = 20; }, "Tutorial · paso intensidad");
    } else if (currentStep === 2) {
      lab.update((c) => { c.appliedVoltageV = -1.2; }, "Tutorial · paso frenado");
    } else {
      lab.tab = "experiment";
    }
  }

  const answerText = $derived.by(() => {
    if (!step) return "";
    if (lab.preferences.lang === "en") {
      return lab.tutorial.step === 2
        ? "Below −V₀ no electron reaches the anode."
        : "Kmax and V₀ do not change. More electrons per second only.";
    }
    return lab.tutorial.step === 2
      ? "Con V más negativo que −V₀ no llega ningún electrón al ánodo."
      : "No cambia Kmax ni V₀. Solo hay más electrones por segundo.";
  });
</script>

{#if lab.tutorial.active && step}
  <div class="tutorial" role="dialog" aria-label="Tutorial guiado" data-testid="tutorial">
    <div class="step-dots" aria-hidden="true">
      {#each steps as _, i}
        <span class="dot" class:active={i === lab.tutorial.step}></span>
      {/each}
    </div>
    <div class="tut-head">
      <span class="badge ok">Guía {lab.tutorial.step + 1}/{total}</span>
      <button class="btn ghost icon" onclick={() => (lab.tutorial = { active: false, step: 0 })} aria-label="Cerrar tutorial">✕</button>
    </div>
    <h3>{step.title}</h3>
    <p class="small">{step.body}</p>

    {#if step.prediction}
      <div class="prediction card">
        <div class="label">{lab.preferences.lang === "en" ? "Prediction" : "Predicción"}</div>
        <p class="small">{step.prediction}</p>
        <div class="pred-actions">
          <button class="btn" class:active={answer === "yes"} onclick={() => (answer = "yes")}>
            {lab.preferences.lang === "en" ? "Yes" : "Sí"}
          </button>
          <button class="btn" class:active={answer === "no"} onclick={() => (answer = "no")}>
            {lab.preferences.lang === "en" ? "No" : "No"}
          </button>
          <button class="btn ghost small" onclick={() => (revealed = true)} disabled={answer === null}>
            {lab.preferences.lang === "en" ? "Reveal" : "Ver resultado"}
          </button>
        </div>
        {#if revealed}
          <div class="reveal small rise" role="status">{answerText}</div>
        {/if}
      </div>
    {/if}

    <div class="tut-actions">
      <button class="btn ghost" onclick={prev} disabled={lab.tutorial.step === 0}>
        {lab.preferences.lang === "en" ? "Back" : "Anterior"}
      </button>
      <button class="btn" onclick={() => applyScenarioFor(lab.tutorial.step)}>
        {lab.preferences.lang === "en" ? "Set up step" : "Preparar paso"}
      </button>
      <button class="btn primary" onclick={next}>
        {lab.tutorial.step === total - 1 ? (lab.preferences.lang === "en" ? "Finish" : "Terminar") : (lab.preferences.lang === "en" ? "Next" : "Siguiente")}
      </button>
    </div>
  </div>
{/if}

<style>
  .tutorial {
    position: fixed;
    right: 24px;
    bottom: 56px;
    width: min(360px, 90vw);
    background: var(--panel-solid);
    border: 1px solid var(--wl-strong, var(--line-strong));
    border-radius: var(--r-lg);
    padding: var(--sp-4);
    box-shadow: var(--el-3);
    z-index: 80;
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
    backdrop-filter: blur(18px);
  }
  .step-dots {
    display: flex;
    gap: 5px;
  }
  .dot {
    width: 18px;
    height: 4px;
    border-radius: 2px;
    background: var(--line-strong);
    transition: background var(--dur-2) var(--ease);
  }
  .dot.active {
    background: var(--wl);
  }
  .tut-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  h3 {
    margin: 0;
    font-size: 1em;
  }
  p {
    margin: 0;
    color: var(--text-2);
    line-height: 1.5;
  }
  .prediction {
    padding: var(--sp-3);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .pred-actions {
    display: flex;
    gap: 6px;
  }
  .reveal {
    background: var(--wl-soft);
    border-radius: var(--r-sm);
    padding: 8px 10px;
    color: var(--text);
  }
  .tut-actions {
    display: flex;
    gap: 6px;
    justify-content: space-between;
  }
</style>
