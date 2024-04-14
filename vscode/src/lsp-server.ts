// Node.js implementation for orgwise lsp server

import { existsSync } from "node:fs";
import { readFile, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import {
  IPCMessageReader,
  IPCMessageWriter,
  createMessageConnection,
} from "vscode-languageserver-protocol/node";
import { URI } from "vscode-uri";

import { LspBackend, initSync } from "../../pkg/orgwise";

const writer = new IPCMessageWriter(process);
const reader = new IPCMessageReader(process);

const connection = createMessageConnection(reader, writer);

let backend: LspBackend;

connection.onRequest("initialize", async (params) => {
  if (!backend) {
    const path = URI.parse((<any>params).initializationOptions.wasmUrl).fsPath;
    const buffer = await readFile(path);
    initSync(buffer);

    backend = new LspBackend({
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
  }

  return backend.onRequest("initialize", params);
});

connection.onRequest((method, params) => {
  return backend.onRequest(method, params);
});

connection.onNotification((method, params) => {
  return backend.onNotification(method, params);
});

connection.listen();
