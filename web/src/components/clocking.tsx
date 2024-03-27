import fetch from "@/command";
import { Clock, PlayCircle, PauseCircle } from "lucide-react";
import React, { useEffect, useState } from "react";
import { parse } from "date-fns";
import { stat } from "fs";

type ClockStatus = {
  offset: number;
  start: string;
  title: string;
};

const Clocking: React.FC = () => {
  const [status, setStatus] = useState<{
    running?: ClockStatus;
    last?: ClockStatus;
  }>({});

  useEffect(() => {
    fetch("clocking-status", {}).then(setStatus);
  }, []);

  return (
    <div className="flex fixed right-[50%] translate-x-[50%] bottom-4 px-4 py-2 rounded-full items-center bg-white gap-4 shadow-md">
      <Clock size={20} />

      <div className="text-xl font-medium">
        {status.running
          ? parse(
              status.running.start,
              "yyyy-MM-dd'T'HH:mm:ss",
              new Date()
            ).toDateString()
          : "00:00:00"}
      </div>

      <PlayCircle size={20} />

      <PauseCircle size={20} />
    </div>
  );
};

export default Clocking;
