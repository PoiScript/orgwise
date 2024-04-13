import * as vscode from "vscode";

import { LanguageClient as BrowserLanguageClient } from "vscode-languageclient/browser";
import {
  BaseLanguageClient,
  Executable,
  LanguageClientOptions,
  LanguageClient as NodeLanguageClient,
  NodeModule,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

import PreviewHtml from "./preview-html";
import SyntaxTree from "./syntax-tree";
import WebPanel from "./web-panel";

declare const WEB_EXTENSION: boolean;
declare const GIT_COMMIT: string;
declare const BUILD_TIME: string;

export let client: BaseLanguageClient;

export function activate(context: vscode.ExtensionContext) {
  const configuration = vscode.workspace.getConfiguration();

  // Options to control the language client
  const clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{ scheme: "file", language: "org" }],
    initializationOptions: {
      todoKeywords: configuration.get("orgwise.todoKeywords"),
      doneKeywords: configuration.get("orgwise.doneKeywords"),
      wasmUrl: vscode.Uri.joinPath(
        context.extensionUri,
        "./dist/orgwise_bg.wasm"
      ).toString(),
    },
  };

  if (WEB_EXTENSION) {
    const workerUrl = vscode.Uri.joinPath(
      context.extensionUri,
      "./dist/lsp-worker.js"
    );

    client = new BrowserLanguageClient(
      "orgwise",
      "Orgwise",
      clientOptions,
      new Worker(workerUrl.toString())
    );
  } else if (configuration.get("orgwise.useCli")) {
    const run: Executable = {
      command: "/Users/poi/.cargo/bin/orgwise",
      args: ["lsp"],
    };

    const serverOptions: ServerOptions = {
      run,
      debug: run,
    };

    client = new NodeLanguageClient(
      "orgwise",
      "Orgwise",
      serverOptions,
      clientOptions
    );
  } else {
    const serverUrl = vscode.Uri.joinPath(
      context.extensionUri,
      "./dist/lsp-server.js"
    );

    const run: NodeModule = {
      module: serverUrl.fsPath,
      transport: TransportKind.ipc,
    };

    const serverOptions: ServerOptions = {
      run,
      debug: run,
    };

    client = new NodeLanguageClient(
      "orgwise",
      "Orgwise",
      serverOptions,
      clientOptions
    );
  }

  // Start the client. This will also launch the server
  client.start();

  context.subscriptions.push(SyntaxTree.register());
  context.subscriptions.push(PreviewHtml.register(context));
  context.subscriptions.push(WebPanel.register(context));

  vscode.commands.registerCommand("orgwise.show-info-ui", () => {
    vscode.window.showInformationMessage(`orgwise info:`, {
      modal: true,
      detail:
        `Web Extension: ${WEB_EXTENSION}\n` +
        `Use CLI: ${configuration.get("orgwise.useCli")}\n` +
        `Git Commit: ${GIT_COMMIT}\n` +
        `Build Time: ${BUILD_TIME}`,
    });
  });
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
