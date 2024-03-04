import { Uri, type Webview } from "vscode";
import manifest from "../dist/.vite/manifest.json";

export const createHtml = (base: Uri, extendUri: Uri, webview: Webview) => {
  const script = webview.asWebviewUri(
    Uri.joinPath(extendUri, "dist", manifest["web/index.html"].file),
  );

  const styles = (manifest["web/index.html"].css || []).map((css) =>
    webview.asWebviewUri(Uri.joinPath(extendUri, "dist", css)),
  );

  const baseUrl = webview.asWebviewUri(base);

  return `<!doctype html>
    <html lang="en">
      <head>
        <meta charset="UTF-8" />

        <meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <base href="${baseUrl}" />
      </head>
      <body>
        <div id="root"></div>

        <script type="module" src="${script}"></script>

        ${styles.map((style) => `<link href="${style}" rel="stylesheet" />`)}
      </body>
    </html>`;
};
