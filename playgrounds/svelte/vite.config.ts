import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
    plugins: [svelte()],
    // top-level await (wasm init in bridge.ts)
    build: { target: "esnext" },
});
