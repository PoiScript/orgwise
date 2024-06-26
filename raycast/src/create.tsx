import {
  LaunchProps,
  LaunchType,
  launchCommand,
  showToast,
} from "@raycast/api";
import { useAtomValue } from "jotai";
import { mutate } from "swr";

import { backendAtom, orgFileAtom } from "./atom";
import { TaskForm } from "./form";

type CreateResult = {
  line: number;
  url: string;
};

export default function Command(
  props: LaunchProps<{ arguments: Arguments.Create }>
) {
  const orgTodoFile = useAtomValue(orgFileAtom);
  const backend = useAtomValue(backendAtom);

  return (
    <TaskForm
      defaultValue={props.arguments}
      onSubmit={(values) => {
        backend
          .executeCommand("headline-create", {
            url: orgTodoFile.toString(),
            ...values,
          })
          .then((result: CreateResult) => {
            mutate("headline-search");
            showToast({ title: "TODO item created" });
            launchCommand({
              name: "list",
              type: LaunchType.UserInitiated,
              context: { selectedItemId: result.url + "#" + result.line },
            });
          });
      }}
    />
  );
}
