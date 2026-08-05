import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
    plugins: [vue()],
    // top-level await (wasm init in bridge.ts)
    build: { target: "esnext" },
});
