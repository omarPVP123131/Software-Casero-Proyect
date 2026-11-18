// Render de ecuaciones con KaTeX (importado en main.ts). Se usa un helper
// en vez de un componente para que funcione igual en tooltips y tarjetas.
import katex from "katex";

export function renderEq(latex: string, display = false): string {
  try {
    return katex.renderToString(latex, {
      displayMode: display,
      throwOnError: false,
      output: "html",
      strict: false,
    });
  } catch {
    return latex;
  }
}
