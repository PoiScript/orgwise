import React, { useEffect, useState } from "react";
import { DataTableDemo } from "./Table";
import { Button } from "./components/ui/button";

export type SearchResult = {
  title: string;
  url: string;
  offset: number;
  level: number;
  priority?: string;
  tags: string[];
  keyword?: string;
  deadline?: string;
  scheduled?: string;
  closed?: string;
};

declare const acquireVsCodeApi: () => any;

let fetchId = 0;
let vscode: any = null;

const fetch = <T, A = any>(command: string, argument: A): Promise<T> => {
  vscode ||= acquireVsCodeApi();

  return new Promise((resolve) => {
    const id = ++fetchId;

    window.addEventListener("message", (ev) => {
      if (ev.data.id == id) {
        resolve(ev.data.result);
      }
    });

    vscode.postMessage({ id, command, arguments: [argument] });
  });
};

const App: React.FC<{}> = () => {
  const [data, setData] = useState<SearchResult[]>([
    {
      closed: null,
      deadline: null,
      keyword: "TODO",
      level: 1,
      offset: 0,
      priority: null,
      scheduled: null,
      tags: [],
      title: "*abc* /a/",
      url: "",
    },
  ]);

  const fetchHeadline = () => {
    fetch<SearchResult[]>("orgwise.search-headline", {})
      .then(setData)
      .catch(console.error);
  };

  useEffect(() => {
    // fetchHeadline();
  }, []);

  return (
    <>
      <Button variant="ghost" onClick={() => fetchHeadline()}>
        Reload
      </Button>

      <DataTableDemo data={data} />
    </>
  );
};

export default App;
