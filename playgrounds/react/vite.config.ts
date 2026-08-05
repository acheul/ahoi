import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
    plugins: [react()],
    // top-level await (wasm init in bridge.ts)
    build: { target: "esnext" },
});
