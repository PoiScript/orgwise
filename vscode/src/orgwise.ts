import {
  Disposable,
  ExtensionContext,
  Uri,
  ViewColumn,
  WebviewPanel,
  commands,
  window,
  workspace,
} from "vscode";

import { client } from "./extension";
import { createHtml } from "./webview";

export const register = (context: ExtensionContext) => {
  context.subscriptions.push(
    commands.registerTextEditorCommand("orgwise.show-orgwise", (editor) => {
      OrgwisePanel.createOrShow(context.extensionUri, editor.document.uri);
    }),
  );
};

class OrgwisePanel {
  public static currentPanel: OrgwisePanel | undefined;

  public static readonly viewType = "orgwise";

  private readonly _panel: WebviewPanel;

  private _disposables: Disposable[] = [];

  public static createOrShow(extensionUri: Uri, orgUri: Uri) {
    const column = window.activeTextEditor.viewColumn! + 1;

    // If we already have a panel, show it.
    if (OrgwisePanel.currentPanel) {
      OrgwisePanel.currentPanel._panel.reveal(column);
      return;
    }

    // Otherwise, create a new panel.
    const panel = window.createWebviewPanel(
      OrgwisePanel.viewType,
      "Orgwise",
      column || ViewColumn.One,
      {
        // Enable javascript in the webview
        enableScripts: true,
        // And restrict the webview to only loading content from our extension's `media` directory.
        localResourceRoots: [
          Uri.joinPath(extensionUri, "dist"),
          ...workspace.workspaceFolders.map((folder) => folder.uri),
        ],
      },
    );

    panel.webview.html = createHtml(orgUri, extensionUri, panel.webview);

    OrgwisePanel.currentPanel = new OrgwisePanel(panel);
  }

  private constructor(panel: WebviewPanel) {
    this._panel = panel;

    // Listen for when the panel is disposed
    // This happens when the user closes the panel or when the panel is closed programmatically
    this._panel.onDidDispose(
      () => {
        this.dispose();
      },
      null,
      this._disposables,
    );

    this._panel.webview.onDidReceiveMessage(
      async (message) => {
        const result = await client.sendRequest("workspace/executeCommand", {
          command: message.command,
          arguments: message.arguments,
        });
        await this._panel.webview.postMessage({
          id: message.id,
          result: result,
        });
      },
      undefined,
      this._disposables,
    );
  }

  public dispose() {
    OrgwisePanel.currentPanel = undefined;

    // Clean up our resources
    this._panel.dispose();

    while (this._disposables.length) {
      const x = this._disposables.pop();
      if (x) {
        x.dispose();
      }
    }
  }
}
