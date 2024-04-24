import { showHUD } from "@raycast/api";
import { getDefaultStore } from "jotai";

import { backendAtom } from "./atom";

type ClockStatus = {
  start: string;
  title: string;
  url: string;
  line: number;
};

export default async function Command() {
  const backend = getDefaultStore().get(backendAtom);

  const status: { running?: ClockStatus } = await backend.executeCommand(
    "clocking-status",
    {}
  );

  if (!status.running) {
    return await showHUD("No running clock");
  }

  await backend.executeCommand("clocking-stop", {
    url: status.running.url,
    line: status.running.line,
  });

  await showHUD(`Stopped clocking on ${JSON.stringify(status.running.title)}`);
}
