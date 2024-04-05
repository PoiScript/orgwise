declare const acquireVsCodeApi: () => any | undefined;

const createExecuteCommand = () => {
  if (typeof acquireVsCodeApi != "undefined") {
    const vscode = acquireVsCodeApi();
    let reqId = 0;

    return function executeCommand<T, A = any>(
      command: string,
      argument?: A
    ): Promise<T> {
      return new Promise((resolve) => {
        const id = ++reqId;

        window.addEventListener("message", (ev) => {
          if (ev.data.id == id) {
            resolve(ev.data.result);
          }
        });

        vscode.postMessage({
          id,
          command: `orgwise.${command}`,
          arguments: [argument || {}],
        });
      });
    };
  } else {
    return function executeCommand<T, A = any>(
      command: string,
      argument?: A
    ): Promise<T> {
      return window
        .fetch("http://127.0.0.1:4100/api/command", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            command,
            argument: argument || {},
          }),
        })
        .then((res) => res.json());
    };
  }
};

export default createExecuteCommand();
