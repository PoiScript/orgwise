import { useAtomValue, useSetAtom } from "jotai/react";
import React, { useEffect } from "react";
import useSWR, { SWRConfig } from "swr";
import executeCommand from "./command";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

import { CalendarDay } from "./CalendarDay";
import { Tasks } from "./Tasks";
import { ViewMode, filtersAtom, viewModeAtom } from "./atom";
import Clocking from "./components/Clocking";

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
    <SWRConfig
      value={{
        fetcher: executeCommand,
        revalidateOnFocus: false,
      }}
    >
      <div className="flex items-center justify-between px-4 pt-4">
        <SearchInput />

        <LoadingButton />
      </div>

      <Clocking />

      {viewMode === ViewMode.Tasks && <Tasks />}

      {viewMode === ViewMode.CalendarDay && <CalendarDay />}
    </SWRConfig>
  );
};

const SearchInput: React.FC = () => {
  const setFilter = useSetAtom(filtersAtom);

  useEffect(() => {
    setFilter({});
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
  const { isLoading, mutate } = useSWR("headline-search");

  return (
    <Button variant="ghost" disabled={isLoading} onClick={() => mutate()}>
      Reload
    </Button>
  );
};

export default App;
