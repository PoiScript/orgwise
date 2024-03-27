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

import manifest from "../dist/.vite/manifest.json";

export default class WebPanel {
  public static currentPanel: WebPanel | undefined;

  public static readonly viewType = "orgwise-web-panel";

  private readonly _panel: WebviewPanel;

  private _disposables: Disposable[] = [];

  public static register(context: ExtensionContext): Disposable {
    return commands.registerTextEditorCommand(
      "orgwise.web-panel-ui",
      (editor) =>
        WebPanel.createOrShow(context.extensionUri, editor.document.uri)
    );
  }

  static createOrShow(extensionUri: Uri, orgUri: Uri) {
    const column = window.activeTextEditor.viewColumn! + 1;

    // If we already have a panel, show it.
    if (WebPanel.currentPanel) {
      WebPanel.currentPanel._panel.reveal(column);
      return;
    }

    // Otherwise, create a new panel.
    const panel = window.createWebviewPanel(
      WebPanel.viewType,
      "Orgwise Web",
      column || ViewColumn.One,
      {
        // Enable javascript in the webview
        enableScripts: true,
        // And restrict the webview to only loading content from our extension's `media` directory.
        localResourceRoots: [
          Uri.joinPath(extensionUri, "dist"),
          ...workspace.workspaceFolders.map((folder) => folder.uri),
        ],
      }
    );

    panel.webview.html = `<!doctype html>
      <html lang="en">
        <head>
          <meta charset="UTF-8" />

          <meta
            name="viewport"
            content="width=device-width, initial-scale=1.0"
          />

          <base href="${panel.webview.asWebviewUri(orgUri)}" />
        </head>
        <body>
          <div id="root"></div>

          <script
            type="module"
            src="${panel.webview.asWebviewUri(
              Uri.joinPath(
                extensionUri,
                "dist",
                manifest["web/src/main.tsx"].file
              )
            )}"
          ></script>

          ${(manifest["web/src/main.tsx"].css || [])
            .map(
              (css) =>
                `<link
                  href="${panel.webview.asWebviewUri(
                    Uri.joinPath(extensionUri, "dist", css)
                  )}"
                  rel="stylesheet"
                />`
            )
            .join("")}
        </body>
      </html>`;

    WebPanel.currentPanel = new WebPanel(panel);
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
      this._disposables
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
      this._disposables
    );
  }

  public dispose() {
    WebPanel.currentPanel = undefined;

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
