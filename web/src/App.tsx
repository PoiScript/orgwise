import { useAtomValue, useSetAtom } from "jotai/react";
import React, { Suspense, useEffect } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

import { Tasks } from "./Tasks";
import { ViewMode, loadingAtom, searchAtom, viewModeAtom } from "./atom";
import { CalendarDay } from "./CalendarDay";
import Clocking from "./components/clocking";

export type SearchResult = {
  title: string;
  url: string;
  line: number;
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
  const viewMode = useAtomValue(viewModeAtom);

  return (
    <>
      <div className="flex items-center justify-between px-4 pt-4">
        <SearchInput />

        <LoadingButton />
      </div>

      <Clocking />

      {viewMode === ViewMode.Tasks && <Tasks />}

      {viewMode === ViewMode.CalendarDay && <CalendarDay />}
    </>
  );
};

const SearchInput: React.FC = () => {
  const search = useSetAtom(searchAtom);

  useEffect(() => {
    search();
  }, []);

  return (
    <Input
      placeholder="Filter..."
      // value={(table.getColumn("title")?.getFilterValue() as string) ?? ""}
      // onChange={(event) =>
      //   table.getColumn("title")?.setFilterValue(event.target.value)
      // }
      className="max-w-sm"
    />
  );
};

const LoadingButton: React.FC = () => {
  const search = useSetAtom(searchAtom);
  const loading = useAtomValue(loadingAtom);

  return (
    <Button variant="ghost" disabled={loading} onClick={search}>
      Reload
    </Button>
  );
};

export default App;
