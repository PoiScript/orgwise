import React, { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";

import { DataTableDemo } from "./Table";
import fetch from "./fetch";

export type SearchResult = {
  title: string;
  url: string;
  offset: number;
  level: number;
  priority?: string;
  tags: string[];
  keyword?: string;
  keyword_type?: string;
  deadline?: string;
  scheduled?: string;
  closed?: string;
};

const App: React.FC<{}> = () => {
  const [data, setData] = useState<SearchResult[]>([]);

  const fetchHeadline = () => {
    fetch<SearchResult[]>("search-headline", {})
      .then(setData)
      .catch(console.error);
  };

  useEffect(() => {
    fetchHeadline();
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
