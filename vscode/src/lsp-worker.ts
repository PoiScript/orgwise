// web worker implementation for orgwise lsp server

import {
  BrowserMessageReader,
  BrowserMessageWriter,
  createMessageConnection,
} from "vscode-languageserver-protocol/browser";

import { WasmLspServer as Server, initSync } from "../../pkg/orgwise";

// @ts-ignore
import wasm from "../dist/orgwise_bg.wasm?arraybuffer";

declare var self: DedicatedWorkerGlobalScope;

initSync(wasm);

const writer = new BrowserMessageWriter(self);
const reader = new BrowserMessageReader(self);

const connection = createMessageConnection(reader, writer);

const server = new Server({
  sendNotification: (method: string, params: any) => {
    return connection.sendNotification(method, params);
  },
  sendRequest: (method: string, params: any) => {
    return connection.sendRequest(method, params);
  },
});

connection.onRequest((method, params) => {
  return server.onRequest(method, params);
});

connection.onNotification((method, params) => {
  return server.onNotification(method, params);
});

connection.listen();
