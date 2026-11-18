// Reloj de simulación independiente del framerate.
//
// Regla dura (spec): la animación integra con dt real, acotado para que una
// pestaña en segundo plano no salte. Pausa, cámara lenta y "step" siguen la
// misma ruta. El estado de partículas se deriva de `time` de forma
// determinista (ver Scene.svelte), así que este reloj es la única entrada.
import { lab } from "./store.svelte";

let raf = 0;
let last = 0;

function frame(now: number): void {
  const dt = last === 0 ? 0 : Math.min(0.05, (now - last) / 1000);
  last = now;
  if (lab.animating) {
    const rate = lab.slowMotion ? 0.25 : 1;
    lab.animTime += dt * rate * lab.timeScale;
  }
  raf = requestAnimationFrame(frame);
}

export function startClock(): () => void {
  if (typeof requestAnimationFrame === "undefined") return () => {};
  cancelAnimationFrame(raf);
  last = 0;
  raf = requestAnimationFrame(frame);
  return () => cancelAnimationFrame(raf);
}

export function stepFrame(): void {
  lab.animTime += 1 / 30;
}

export function resetTime(): void {
  lab.animTime = 0;
}
