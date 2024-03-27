// Node.js implementation for orgwise lsp server

import { readFileSync, existsSync } from "node:fs";
import { readFile, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import { resolve } from "node:path";
import {
  IPCMessageReader,
  IPCMessageWriter,
  createMessageConnection,
} from "vscode-languageserver-protocol/node";
import { URI } from "vscode-uri";

import { WasmLspServer as Server, initSync } from "../../pkg/orgwise";

const buffer = readFileSync(resolve(__dirname, "./orgwise_bg.wasm"));
initSync(buffer);

const writer = new IPCMessageWriter(process);
const reader = new IPCMessageReader(process);

const connection = createMessageConnection(reader, writer);

const server = new Server({
  sendNotification: (method: string, params: any) =>
    connection.sendNotification(method, params),

  sendRequest: (method: string, params: any) =>
    connection.sendRequest(method, params),

  homeDir: () => URI.file(homedir()).toString() + "/",

  readToString: async (url: string) => {
    const path = URI.parse(url).fsPath;
    if (existsSync(path)) {
      return readFile(path, { encoding: "utf-8" });
    } else {
      return "";
    }
  },

  write: (url: string, content: string) =>
    writeFile(URI.parse(url).fsPath, content),
});

connection.onRequest((method, params) => {
  return server.onRequest(method, params);
});

connection.onNotification((method, params) => {
  return server.onNotification(method, params);
});

connection.listen();
