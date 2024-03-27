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
import { Utils } from "vscode-uri";

import { client } from "./extension";

export default class PreviewHtml {
  public static currentPanel: PreviewHtml | undefined;

  public static readonly viewType = "orgwise-preview-html";

  private readonly _panel: WebviewPanel;
  private _orgUri: Uri;
  private _extensionUri: Uri;

  private _disposables: Disposable[] = [];

  static register(context: ExtensionContext): Disposable {
    return commands.registerTextEditorCommand(
      "orgwise.preview-html-ui",
      (editor) =>
        PreviewHtml.createOrShow(context.extensionUri, editor.document.uri)
    );
  }

  private static createOrShow(extensionUri: Uri, orgUri: Uri) {
    const column = window.activeTextEditor.viewColumn! + 1;

    // If we already have a panel, show it.
    if (PreviewHtml.currentPanel) {
      PreviewHtml.currentPanel._panel.reveal(column);
      PreviewHtml.currentPanel._orgUri = orgUri;
      PreviewHtml.currentPanel.refresh();
      // PreviewHtmlPanel.currentPanel._panel.webview.pos
      return;
    }

    // Otherwise, create a new panel.
    const panel = window.createWebviewPanel(
      PreviewHtml.viewType,
      "Preview of " + Utils.basename(orgUri),
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

    PreviewHtml.currentPanel = new PreviewHtml(panel, orgUri, extensionUri);
  }

  private constructor(panel: WebviewPanel, orgUri: Uri, extensionUri: Uri) {
    this._panel = panel;
    this._orgUri = orgUri;
    this._extensionUri = extensionUri;

    // Set the webview's initial html content
    this._update();

    // Listen for when the panel is disposed
    // This happens when the user closes the panel or when the panel is closed programmatically
    this._panel.onDidDispose(
      () => {
        this.dispose();
      },
      null,
      this._disposables
    );

    workspace.onDidChangeTextDocument((event) => {
      if (event.document.uri.fsPath === this._orgUri.fsPath) {
        this.refresh();
      }
    }, this._disposables);

    workspace.onDidOpenTextDocument((document) => {
      if (document.uri.fsPath === this._orgUri.fsPath) {
        this.refresh();
      }
    }, this._disposables);

    // Update the content based on view changes
    this._panel.onDidChangeViewState(
      (e) => {
        if (this._panel.visible) {
          this.refresh();
        }
      },
      null,
      this._disposables
    );
  }

  private readonly _delay = 300;
  private _throttleTimer: any;
  private _firstUpdate = true;

  public refresh() {
    // Schedule update if none is pending
    if (!this._throttleTimer) {
      if (this._firstUpdate) {
        this._update();
      } else {
        this._throttleTimer = setTimeout(() => this._update(), this._delay);
      }
    }

    this._firstUpdate = false;
  }

  private async _update() {
    clearTimeout(this._throttleTimer);
    this._throttleTimer = undefined;

    if (!client) {
      return;
    }

    try {
      const content: string = await client.sendRequest(
        "workspace/executeCommand",
        {
          command: "orgwise.preview-html",
          arguments: [this._orgUri.with({ scheme: "file" }).toString()],
        }
      );

      this._panel.webview.html = `<!doctype html>
        <html lang="en">
          <head>
            <meta charset="UTF-8" />

            <meta
              name="viewport"
              content="width=device-width, initial-scale=1.0"
            />

            <base href="${this._panel.webview.asWebviewUri(this._orgUri)}" />

            <link
              href="${this._panel.webview.asWebviewUri(
                Uri.joinPath(this._extensionUri, "media", "org-mode.css")
              )}"
              rel="stylesheet"
            />
          </head>
          <body>
            <article>${content}</article>
          </body>
        </html>`;
    } catch {}
  }

  public dispose() {
    PreviewHtml.currentPanel = undefined;

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
