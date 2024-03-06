declare const acquireVsCodeApi: () => any;

const createFetch = () => {
  if (typeof acquireVsCodeApi != "undefined") {
    const vscode = acquireVsCodeApi();
    let fetchId = 0;

    return function fetch<T, A = any>(
      command: string,
      argument: A,
    ): Promise<T> {
      return new Promise((resolve) => {
        const id = ++fetchId;

        window.addEventListener("message", (ev) => {
          if (ev.data.id == id) {
            resolve(ev.data.result);
          }
        });

        vscode.postMessage({
          id,
          command: `orgwise.${command}`,
          arguments: [argument],
        });
      });
    };
  } else {
    return function fetch<T, A = any>(
      command: string,
      argument: A,
    ): Promise<T> {
      return window
        .fetch("http://127.0.0.1:3000/api/" + command, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(argument),
        })
        .then((res) => res.json());
    };
  }
};

export default createFetch();
