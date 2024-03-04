// web worker implementation for orgwise lsp server

import {
  BrowserMessageReader,
  BrowserMessageWriter,
  createMessageConnection,
} from "vscode-languageserver-protocol/browser";

import init, { Server } from "../../pkg/orgwise";

declare var self: DedicatedWorkerGlobalScope;

init()
  .then(() => {
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
  })
  .catch(console.error);
