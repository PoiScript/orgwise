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
import { createHtml } from "./webview";

export const register = (context: ExtensionContext) => {
  context.subscriptions.push(
    commands.registerTextEditorCommand("orgwise.preview-html", (editor) => {
      PreviewHtmlPanel.createOrShow(context.extensionUri, editor.document.uri);
    }),
  );
};

class PreviewHtmlPanel {
  public static currentPanel: PreviewHtmlPanel | undefined;

  public static readonly viewType = "orgwisePreviewHtml";

  private readonly _panel: WebviewPanel;
  private _orgUri: Uri;

  private _disposables: Disposable[] = [];

  public static createOrShow(extensionUri: Uri, orgUri: Uri) {
    const column = window.activeTextEditor.viewColumn! + 1;

    // If we already have a panel, show it.
    if (PreviewHtmlPanel.currentPanel) {
      PreviewHtmlPanel.currentPanel._panel.reveal(column);
      PreviewHtmlPanel.currentPanel._orgUri = orgUri;
      PreviewHtmlPanel.currentPanel.refresh();
      // PreviewHtmlPanel.currentPanel._panel.webview.pos
      return;
    }

    // Otherwise, create a new panel.
    const panel = window.createWebviewPanel(
      PreviewHtmlPanel.viewType,
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
      },
    );

    panel.webview.html = createHtml(orgUri, extensionUri, panel.webview);

    PreviewHtmlPanel.currentPanel = new PreviewHtmlPanel(panel, orgUri);
  }

  private constructor(panel: WebviewPanel, orgUri: Uri) {
    this._panel = panel;
    this._orgUri = orgUri;

    // Set the webview's initial html content
    this._update();

    // Listen for when the panel is disposed
    // This happens when the user closes the panel or when the panel is closed programmatically
    this._panel.onDidDispose(
      () => {
        this.dispose();
      },
      null,
      this._disposables,
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
      this._disposables,
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
        },
      );
      this._panel.webview.postMessage({
        type: "preview-html",
        content: content,
      });
    } catch {}
  }

  public dispose() {
    PreviewHtmlPanel.currentPanel = undefined;

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
