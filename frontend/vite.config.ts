import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri espera un servidor fijo en 1420 y un bundle estático en build/.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    host: "0.0.0.0",
    port: 1420,
    strictPort: true,
    fs: { allow: [".."] },
  },
  build: {
    target: "es2022",
    outDir: "build",
    emptyOutDir: true,
    sourcemap: false,
    assetsInlineLimit: 0,
  },
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
