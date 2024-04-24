import { useAtomValue, useSetAtom } from "jotai/react";
import React, { useEffect } from "react";
import useSWR, { SWRConfig } from "swr";
import executeCommand from "./command";

import Clocking from "@/components/Clocking";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

import { CalendarDay } from "./CalendarDay";
import { Tasks } from "./Tasks";
import { ViewMode, filtersAtom, viewModeAtom } from "./atom";

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

export const SearchInput: React.FC = () => {
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

export const LoadingButton: React.FC = () => {
  const { isLoading, mutate } = useSWR("headline-search");

  return (
    <Button variant="ghost" disabled={isLoading} onClick={() => mutate()}>
      Reload
    </Button>
  );
};

export default App;
