import { fileURLToPath } from "node:url";

import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const demo = (file: string) => fileURLToPath(new URL(`./src/demo/${file}`, import.meta.url));

// https://v2.tauri.app/start/frontend/vite/
export default defineConfig(({ mode }) => ({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, watch: { ignored: ["**/src-tauri/**", "**/crates/**", "**/target/**"] } },
  build: { target: "es2021", sourcemap: false },
  // `vite build --mode demo`: the same UI against sample data, for the
  // website (website/, quickdesk.click). Tauri's API is swapped for src/demo.
  ...(mode === "demo" && {
    base: "./",
    resolve: {
      alias: {
        "@tauri-apps/api/core": demo("core.ts"),
        "@tauri-apps/api/event": demo("event.ts"),
        "@tauri-apps/api/webviewWindow": demo("webviewWindow.ts"),
      },
    },
    build: { target: "es2021", sourcemap: false, outDir: "dist-demo", emptyOutDir: true },
  }),
}));
