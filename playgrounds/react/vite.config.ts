import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
    plugins: [react()],
    // top-level await (wasm init in ahoi.ts)
    build: { target: "esnext" },
});
