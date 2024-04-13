import react from "@vitejs/plugin-react";
import * as esbuild from "esbuild";
import { copyFile } from "node:fs/promises";
import path from "node:path";
import { execSync } from "node:child_process";
import * as vite from "vite";

const GIT_COMMIT = execSync("git rev-parse --short HEAD", {
  encoding: "utf8",
}).trim();
const BUILD_TIME = new Date().toISOString();

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
  define: {
    WEB_EXTENSION: "false",
    GIT_COMMIT: `"${GIT_COMMIT}"`,
    BUILD_TIME: `"${BUILD_TIME}"`,
  },
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
  define: {
    WEB_EXTENSION: "true",
    GIT_COMMIT: `"${GIT_COMMIT}"`,
    BUILD_TIME: `"${BUILD_TIME}"`,
  },
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

await esbuild.build({
  bundle: true,
  entryPoints: ["./vscode/src/lsp-worker.ts"],
  outfile: "./vscode/dist/lsp-worker.js",
  format: "cjs",
  platform: "browser",
  treeShaking: true,
  minify: true,
});

await copyFile("./pkg/orgwise_bg.wasm", "./vscode/dist/orgwise_bg.wasm");
