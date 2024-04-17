import * as esbuild from "esbuild";
import { execSync } from "node:child_process";

const GIT_COMMIT = execSync("git rev-parse --short HEAD", {
  encoding: "utf8",
}).trim();
const BUILD_TIME = new Date().toISOString();

await esbuild.build({
  bundle: true,
  entryPoints: ["./src/extension.ts"],
  external: ["vscode"],
  outfile: "./dist/node.js",
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
  entryPoints: ["./src/extension.ts"],
  external: ["vscode"],
  outfile: "./dist/browser.js",
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
  entryPoints: ["./src/lsp-server.ts"],
  external: ["node:*"],
  outfile: "./dist/lsp-server.js",
  format: "cjs",
  platform: "node",
  treeShaking: true,
  minify: true,
});

await esbuild.build({
  bundle: true,
  entryPoints: ["./src/lsp-worker.ts"],
  outfile: "./dist/lsp-worker.js",
  format: "cjs",
  platform: "browser",
  treeShaking: true,
  minify: true,
});
