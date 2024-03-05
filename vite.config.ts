import react from "@vitejs/plugin-react";
import path from "node:path";
import { defineConfig } from "vite";

export default defineConfig({
  server: {
    open: "./web/index.html",
    port: 4200,
  },
  build: {
    emptyOutDir: false,
    rollupOptions: {
      input: "./web/index.html",
    },
    outDir: "./vscode/dist",
    minify: false,
    manifest: true,
  },
  resolve: {
    alias: {
      "@": path.resolve("./web/src"),
    },
  },
  plugins: [react()],
});
