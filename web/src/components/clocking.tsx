import executeCommand from "@/command";
import { formatMinutes } from "@/lib/utils";
import * as Toast from "@radix-ui/react-toast";
import { differenceInMinutes, isValid, parse } from "date-fns";
import { Clock, PauseCircle } from "lucide-react";
import React, { useEffect, useState } from "react";
import useSWR, { mutate } from "swr";

type ClockStatus = {
  start: string;
  title: string;
  url: string;
  line: number;
};

const Clocking: React.FC = () => {
  const {
    data: status,
    isLoading,
    error,
  } = useSWR<{ running: ClockStatus }>("clocking-status");

  if (isLoading || error) return;

  return (
    <Toast.Provider>
      <Toast.Root
        open={!!status?.running}
        className="group pointer-events-auto relative flex w-full items-center justify-between space-x-2 overflow-hidden rounded-full border py-2 px-4 shadow-lg transition-all
        data-[state=open]:animate-in
        data-[state=closed]:animate-out
        data-[state=closed]:fade-out-80
        data-[state=closed]:slide-out-to-bottom-full
        data-[state=open]:slide-in-from-bottom-full border bg-background text-foreground"
      >
        <Clock size={18} />

        {status.running && (
          <div className="text-xl font-medium">
            <CountDown
              start={parse(
                status.running.start,
                "yyyy-MM-dd'T'HH:mm:ss",
                new Date()
              )}
            />
          </div>
        )}

        <PauseCircle
          onClick={() =>
            executeCommand("clocking-stop", {
              url: status.running.url!,
              line: status.running.line!,
            }).then(() => {
              mutate("clocking-status");
              mutate("headline-search");
            })
          }
          size={20}
        />
      </Toast.Root>

      <Toast.Viewport className="fixed right-[50%] translate-x-[50%] z-[30] p-4 bottom-0" />
    </Toast.Provider>
  );
};

export const CountDown: React.FC<{ start: Date }> = ({ start }) => {
  const [minutes, setMinutes] = useState(0);

  useEffect(() => {
    if (!isValid(start)) {
      return;
    }

    const fn = () => setMinutes(differenceInMinutes(new Date(), start));

    fn();
    const timer = setInterval(fn, 60);

    return () => {
      clearInterval(timer);
    };
  }, [start]);

  return <>{formatMinutes(minutes)}</>;
};

export default Clocking;
