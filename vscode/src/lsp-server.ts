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

import { Server, initSync } from "../../pkg/orgwise";

const buffer = readFileSync(resolve(__dirname, "./orgwise_bg.wasm"));
initSync(buffer);

const writer = new IPCMessageWriter(process);
const reader = new IPCMessageReader(process);

const connection = createMessageConnection(reader, writer);

const server = new Server({
  sendNotification: (method: string, params: any) => {
    return connection.sendNotification(method, params);
  },
  sendRequest: (method: string, params: any) => {
    return connection.sendRequest(method, params);
  },

  homeDir: () => homedir(),

  readToString: async (url: string) => {
    const uri = URI.parse(url);
    const path = uri.fsPath;
    if (existsSync(path)) {
      return readFile(path, { encoding: "utf-8" });
    } else {
      return "";
    }
  },

  write: async (url: string, content: string) => {
    const uri = URI.parse(url);
    const path = uri.fsPath;
    return writeFile(path, content);
  },
});

connection.onRequest((method, params) => {
  return server.onRequest(method, params);
});

connection.onNotification((method, params) => {
  return server.onNotification(method, params);
});

connection.listen();
