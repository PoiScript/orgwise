import react from "@vitejs/plugin-react";
import path from "node:path";
import { defineConfig } from "vite";

export default defineConfig({
  build: {
    emptyOutDir: false,
    rollupOptions: { input: "./src/main.tsx" },
    outDir: "dist",
    minify: true,
    assetsDir: "",
    manifest: ".vite.manifest.json",
  },
  resolve: {
    alias: { "@": path.resolve("./src") },
  },
  plugins: [react()],
});
