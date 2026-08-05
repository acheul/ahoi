import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
    plugins: [solid()],
    // top-level await (wasm init in bridge.ts)
    build: { target: "esnext" },
});
