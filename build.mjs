import react from "@vitejs/plugin-react";
import * as esbuild from "esbuild";
import { copyFile } from "node:fs/promises";
import path from "node:path";
import * as vite from "vite";
import arraybuffer from "vite-plugin-arraybuffer";

await vite.build({
  configFile: false,
  build: {
    emptyOutDir: false,
    rollupOptions: {
      input: "./web/src/main.tsx",
    },
    outDir: "./vscode/dist",
    minify: true,
    manifest: true,
  },
  resolve: {
    alias: {
      "@": path.resolve("./web/src"),
    },
  },
  plugins: [react()],
});

await esbuild.build({
  bundle: true,
  entryPoints: ["./vscode/src/extension.ts"],
  external: ["vscode"],
  outfile: "./vscode/dist/node.js",
  format: "cjs",
  platform: "node",
  treeShaking: true,
  minify: true,
  define: { WEB_EXTENSION: "false" },
});

await esbuild.build({
  bundle: true,
  entryPoints: ["./vscode/src/extension.ts"],
  external: ["vscode"],
  outfile: "./vscode/dist/browser.js",
  format: "cjs",
  platform: "browser",
  treeShaking: true,
  minify: true,
  define: { WEB_EXTENSION: "true" },
});

await esbuild.build({
  bundle: true,
  entryPoints: ["./vscode/src/lsp-server.ts"],
  external: ["node:*"],
  outfile: "./vscode/dist/lsp-server.js",
  format: "cjs",
  platform: "node",
  treeShaking: true,
  minify: true,
});

await copyFile("./pkg/orgwise_bg.wasm", "./vscode/dist/orgwise_bg.wasm");

await vite.build({
  configFile: false,
  build: {
    emptyOutDir: false,
    lib: {
      name: "orgwise",
      entry: "./vscode/src/lsp-worker.ts",
      formats: ["iife"],
      fileName: () => "lsp-worker.js",
    },
    outDir: "./vscode/dist",
    minify: true,
    manifest: false,
  },
  plugins: [arraybuffer()],
});
