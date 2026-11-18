import { test, expect } from "@playwright/test";

test.describe("PhotoLab UI", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
    await expect(page.getByTestId("emission-state")).toHaveText(/EMITE/, { timeout: 20_000 });
  });

  test("la escena pinta píxeles no vacíos (canvas activo)", async ({ page }) => {
    const canvas = page.locator(".scene-wrap canvas");
    await expect(canvas).toBeVisible();
    // El loop pinta en el frame siguiente al montaje: esperamos a que el
    // canvas deje de estar en blanco antes de muestrear (nada de sleeps fijos).
    const litPixels = () =>
      canvas.evaluate((el: HTMLCanvasElement) => {
        const ctx = el.getContext("2d")!;
        const { data } = ctx.getImageData(0, 0, el.width, el.height);
        let lit = 0;
        for (let i = 3; i < data.length; i += 40) if (data[i] > 0) lit++;
        return lit;
      });
    await expect.poll(litPixels, { timeout: 8000 }).toBeGreaterThan(100);
    const nonEmpty = await litPixels();
    expect(nonEmpty).toBeGreaterThan(100);
  });

  test("snapshot del canvas: sodio 400 nm (regresión visual)", async ({ page }) => {
    await page.waitForTimeout(600);
    await expect(page.locator(".scene-wrap")).toHaveScreenshot("scene-sodium-400nm.png", {
      maxDiffPixelRatio: 0.12,
      animations: "disabled",
    });
  });

  test("cambiar a rojo sin emisión y a bloqueo por frenado", async ({ page }) => {
    await page.getByRole("button", { name: /Rojo/i }).click();
    await expect(page.getByTestId("emission-state")).toHaveText(/NO EMITE/);
    await page.getByRole("button", { name: /Verde/i }).click();
    await expect(page.getByTestId("emission-state")).toHaveText(/EMITE/);
    // Baja el voltaje con el slider de frenado
    await page.evaluate(() => {
      /* usa el atajo del escenario "Potencial de frenado" vía paleta */
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "k", ctrlKey: true }));
    });
    await page.getByRole("option", { name: /Potencial de frenado/ }).click();
    await expect(page.getByTestId("emission-state")).toHaveText(/BLOQUEADO/);
  });

  test("paleta de comandos + barrido + ajuste de h", async ({ page }) => {
    await page.keyboard.press("Control+k");
    await page.getByRole("option", { name: /Medir h/ }).click();
    await expect(page.getByRole("tab", { name: "Experimento V₀–f" })).toHaveAttribute("aria-selected", "true");
    await expect(page.getByText("h medida")).toBeVisible({ timeout: 15_000 });
    await expect(page.getByText("error vs teórico")).toBeVisible();
  });

  test("tutorial guiado con pregunta de predicción", async ({ page }) => {
    await page.keyboard.press("Control+k");
    await page.getByRole("option", { name: /tutorial/i }).click();
    await expect(page.getByTestId("tutorial")).toBeVisible();
    await page.getByRole("button", { name: "Sí" }).click();
    await page.getByRole("button", { name: /Ver resultado/ }).click();
    await expect(page.getByText(/No cambia Kmax/)).toBeVisible();
  });

  test("el informe V₀–f se genera como HTML autocontenido", async ({ page }) => {
    // En navegador openReport abre una ventana con el informe; aquí se
    // intercepta la llamada para validar contenido sin depender del popup.
    await page.addInitScript(() => {
      const w = window as unknown as { __reportHtml?: string };
      window.open = (() => {
        const doc = {
          write: (html: string) => {
            w.__reportHtml = (w.__reportHtml ?? "") + html;
          },
          close: () => undefined,
        };
        return { document: doc } as unknown as Window;
      }) as typeof window.open;
    });
    await page.reload();
    await expect(page.getByTestId("emission-state")).toHaveText(/EMITE/);
    await page.getByRole("tab", { name: /Experimento/ }).click();
    await page.getByRole("button", { name: /Barrido automático|Barrido/ }).first().click();
    await expect.poll(async () => page.locator("tbody tr").count()).toBeGreaterThan(1);
    await page.getByRole("button", { name: "PDF" }).first().click();
    const html = await expect
      .poll(async () => page.evaluate(() => (window as unknown as { __reportHtml?: string }).__reportHtml ?? ""))
      .toContain("PhotoLab · Experimento");
    expect(await page.evaluate(() => (window as unknown as { __reportHtml?: string }).__reportHtml)).toContain("h medida");
  });

  test("el layout no recorta ni desborda a 1280×800", async ({ page }) => {
    // Regresión real: las secciones de controles se aplastaban por flex-shrink
    // (Material: 330 px de contenido en 257 px) y las lecturas desbordaban.
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.waitForTimeout(400);
    const report = await page.evaluate(() => {
      const clippedSections = [...document.querySelectorAll(".controls-panel > .section")]
        .filter((s) => s.scrollHeight > Math.round(s.getBoundingClientRect().height) + 2).length;
      const read = document.querySelector(".readouts")!;
      const escaping = [...document.querySelectorAll(".metric-value")].filter((v) => {
        const card = v.closest(".metric")!.getBoundingClientRect();
        return v.getBoundingClientRect().right > card.right + 1;
      }).length;
      return { clippedSections, readoutsOverflow: read.scrollHeight - read.clientHeight, escaping };
    });
    expect(report.clippedSections).toBe(0);
    expect(report.escaping).toBe(0);
    expect(report.readoutsOverflow).toBeLessThanOrEqual(2);
  });

  test("tema alto contraste aplica tokens", async ({ page }) => {
    await page.getByRole("button", { name: "contrast" }).click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "contrast");
    // El token --bg-0 del tema de alto contraste es negro puro.
    const bg = await page.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue("--bg-0").trim(),
    );
    // El navegador puede normalizar #000000 → #000 según el motor.
    expect(["#000", "#000000"]).toContain(bg.toLowerCase());
  });

  test("nivel de detalle: compacto cabe sin scroll a 1280×800", async ({ page }) => {
    // La altura es la variable crítica: 800 px es el caso apretado.
    await page.setViewportSize({ width: 1280, height: 800 });
    await expect(page.getByTestId("emission-state")).toHaveText(/EMITE/);
    const alto = () =>
      page.evaluate(() => {
        // El que scrollea es el propio panel (el .side solo lo contiene).
        const panel = document.querySelector(".controls-panel") as HTMLElement;
        const total = [...document.querySelectorAll(".controls-panel > *")].reduce(
          (a, el) => a + (el as HTMLElement).getBoundingClientRect().height,
          0,
        );
        return {
          total: Math.round(total),
          caja: panel.clientHeight,
          scroll: panel.scrollHeight > panel.clientHeight + 2,
        };
      });

    // A 800 px de alto el modo automático ya elige compacto: eso es lo que
    // evita el scroll en la ventana real (~770 px).
    await expect(page.getByRole("button", { name: "Compacto" })).toHaveAttribute("aria-pressed", "true");
    await expect(page.locator(".detail-row span").first()).toContainText("auto");

    // Forzado a completo, el panel desborda: es el problema que motiva el modo.
    await page.getByRole("button", { name: "Completo" }).click();
    const completo = await alto();
    expect(completo.total).toBeGreaterThan(completo.caja);
    expect(completo.scroll).toBe(true);

    await page.getByRole("button", { name: "Compacto" }).click();
    await expect(page.getByRole("button", { name: "Compacto" })).toHaveAttribute("aria-pressed", "true");
    const compacto = await alto();
    expect(compacto.total).toBeLessThan(compacto.caja);
    expect(compacto.scroll).toBe(false);

    // la fila compacta es horizontal y el slider sigue siendo usable
    const fila = page.locator(".compact-row").first();
    expect(await fila.evaluate((el) => getComputedStyle(el).flexDirection)).toBe("row");
    await fila.locator('input[type="range"]').first().fill("520");
    await expect(page.locator(".controls-panel")).toContainText("520 nm");
  });

  test("overlay de atajos se abre con ? y se cierra con Esc", async ({ page }) => {
    await expect(page.getByTestId("emission-state")).toHaveText(/EMITE/);
    await page.keyboard.press("?");
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();
    await expect(page.locator("#shortcuts-title")).toContainText(/Atajos/i);
    expect(await page.locator(".sheet .row").count()).toBeGreaterThanOrEqual(8);
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();
    // el botón de la barra abre lo mismo
    await page.getByRole("button", { name: "Ver atajos de teclado" }).click();
    await expect(dialog).toBeVisible();
  });

  test("los tooltips del diagrama explican cada magnitud", async ({ page }) => {
    await expect(page.getByTestId("emission-state")).toHaveText(/EMITE/);
    const box = (await page.locator(".scene-wrap").boundingBox())!;
    const tip = page.locator(".readout-float.tip .tip-title");

    await page.mouse.move(box.x + 120, box.y + 105); // escalera de energías
    await expect(tip).toContainText(/Escalera de energías/i);

    await page.mouse.move(box.x + 60, box.y + 300); // haz de luz
    await expect(tip).toContainText(/Luz incidente/i);

    // el cuerpo del tooltip trae datos vivos, no texto fijo
    await page.mouse.move(box.x + 120, box.y + 105);
    const cuerpo = await page.locator(".readout-float.tip .tip-body").innerText();
    expect(cuerpo).toMatch(/Kmax = \d\.\d\d eV/);
  });

  test("los iconos del panel son SVG, no glifos de fuente", async ({ page }) => {
    await expect(page.getByTestId("emission-state")).toHaveText(/EMITE/);
    expect(await page.locator(".controls-panel .section-icon svg").count()).toBe(5);
    const texto = await page.locator(".controls-panel").innerText();
    expect(/[◈≋⎓⏦⚗]/.test(texto)).toBe(false);
  });
});
