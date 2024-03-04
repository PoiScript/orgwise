import react from "@vitejs/plugin-react";
import * as esbuild from "esbuild";
import { copyFile } from "node:fs/promises";
import path from "node:path";
import * as vite from "vite";

await vite.build({
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

await esbuild.build({
  bundle: true,
  entryPoints: ["./vscode/src/extension.ts"],
  external: ["vscode"],
  outfile: "./vscode/dist/node.js",
  format: "cjs",
  platform: "node",
  treeShaking: true,
  define: { WEB_EXTENSION: "false" },
});

await esbuild.build({
  bundle: true,
  entryPoints: ["./vscode/src/extension.ts"],
  external: ["vscode"],
  outfile: "./vscode/dist/browser.js",
  format: "esm",
  platform: "browser",
  treeShaking: true,
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
});

await vite.build({
  build: {
    emptyOutDir: false,
    lib: {
      name: "orgwise",
      entry: "./vscode/src/lsp-worker.ts",
      formats: ["umd"],
      fileName: () => "lsp-worker.js",
    },
    outDir: "./vscode/dist",
    minify: false,
  },
});

await copyFile("./pkg/orgwise_bg.wasm", "./vscode/dist/orgwise_bg.wasm");
