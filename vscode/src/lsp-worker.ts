// web worker implementation for orgwise lsp server

import {
  BrowserMessageReader,
  BrowserMessageWriter,
  createMessageConnection,
} from "vscode-languageserver-protocol/browser";

import init, { LspBackend } from "../../pkg/orgwise";

declare var self: DedicatedWorkerGlobalScope;

const writer = new BrowserMessageWriter(self);
const reader = new BrowserMessageReader(self);

const connection = createMessageConnection(reader, writer);

let backend: LspBackend;

connection.onRequest("initialize", async (params) => {
  if (!backend) {
    await init((<any>params).initializationOptions.wasmUrl);

    backend = new LspBackend({
      sendNotification: (method: string, params: any) => {
        return connection.sendNotification(method, params);
      },
      sendRequest: (method: string, params: any) => {
        return connection.sendRequest(method, params);
      },
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
