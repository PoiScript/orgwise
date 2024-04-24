import {
  Action,
  ActionPanel,
  Color,
  Icon,
  LaunchProps,
  List,
  Toast,
  showToast,
  useNavigation,
} from "@raycast/api";
import {
  differenceInMinutes,
  format,
  formatDistanceToNowStrict,
  parse,
  getHours,
  getMinutes,
  lightFormat,
  formatDuration,
} from "date-fns";
import { atom, useAtomValue, useSetAtom } from "jotai";
import { useHydrateAtoms } from "jotai/utils";
import React, { useEffect, useState } from "react";
import useSWR, { SWRConfig, mutate } from "swr";

import type { SearchResult } from "../../web/src/atom";
import { backendAtom, orgPrioritiesAtom, orgTagsAtom } from "./atom";
import { TaskForm } from "./form";

const showDetailAtom = atom<"no" | "metadata-only" | "all">("no");

export default function Command(props: LaunchProps) {
  useHydrateAtoms([
    [
      showDetailAtom,
      props.launchContext?.selectedItemId ? "no" : "metadata-only",
    ],
  ]);

  const backend = useAtomValue(backendAtom);

  return (
    <SWRConfig
      value={{
        fetcher: (command, argument = {}) =>
          backend.executeCommand(command, argument),
      }}
    >
      <TaskList selectedItemId={props.launchContext?.selectedItemId} />
    </SWRConfig>
  );
}

export function formatMinutes(minutes: number) {
  const hh = (minutes / 60) | 0;
  const mm = minutes % 60;
  return `${hh.toString().padStart(2, "0")}:${mm.toString().padStart(2, "0")}`;
}

const TaskList: React.FC<{ selectedItemId?: string }> = ({
  selectedItemId,
}) => {
  const { data = [], isLoading } = useSWR<SearchResult[]>("headline-search");

  const isShowingDetail = useAtomValue(showDetailAtom);

  return (
    <List
      selectedItemId={isLoading ? undefined : selectedItemId}
      isShowingDetail={isShowingDetail !== "no"}
      isLoading={isLoading}
      searchBarAccessory={<GroupBy />}
    >
      {data.length === 0 ? (
        <List.EmptyView
          icon={{ source: "https://placekitten.com/500/500" }}
          title="Type something to get started"
        />
      ) : (
        data.map((item) => <TaskItem key={item.url + item.line} item={item} />)
      )}
    </List>
  );
};

const GroupBy: React.FC = () => {
  return (
    <List.Dropdown tooltip="Group by" storeValue={true}>
      <List.Dropdown.Item title="Default" value="default" />
    </List.Dropdown>
  );
};

const TaskItem: React.FC<{ item: SearchResult }> = ({ item }) => {
  const [offset, setOffset] = useState<number | null>(() => {
    if (!item.clocking.start) {
      return null;
    } else {
      return differenceInMinutes(new Date(), item.clocking.start);
    }
  });

  const accessories: any[] = item.tags.map((tag) => ({ tag }));

  if (item.priority) {
    accessories.push({
      tag: { value: "#" + item.priority, color: Color.Magenta },
    });
  }

  if (item.clocking.total_minutes) {
    accessories.push({
      icon: Icon.Stopwatch,
      tag: {
        value: formatDuration(
          { minutes: item.clocking.total_minutes },
          { zero: false, format: ["days", "hours", "minutes"] }
        ),
        color: Color.Yellow,
      },
    });
  }

  if (typeof offset === "number") {
    accessories.push({
      icon: Icon.Play,
      tag: { value: formatMinutes(offset), color: Color.Orange },
    });
  }

  useEffect(() => {
    if (!item.clocking.start) {
      setOffset(null);
      return;
    }

    const start = parse(
      item.clocking.start,
      "yyyy-MM-dd'T'HH:mm:ss",
      new Date()
    );

    const fn = () => setOffset(differenceInMinutes(new Date(), start));

    fn();
    const timer = setInterval(fn, 60);

    return () => clearInterval(timer);
  }, [item.clocking.start]);

  const isDone = item.keyword?.type === "DONE";

  const isShowingDetail = useAtomValue(showDetailAtom);

  return (
    <List.Item
      id={item.url + "#" + item.line}
      title={item.title}
      icon={
        isDone
          ? {
              source: Icon.CheckCircle,
              tooltip: "DONE",
              tintColor: Color.Green,
            }
          : {
              source: Icon.Circle,
              tooltip: "TODO",
              tintColor: Color.PrimaryText,
            }
      }
      accessories={isShowingDetail ? [] : accessories}
      keywords={[
        item.title,
        ...item.tags,
        item.priority || "",
        item.keyword?.value || "",
      ]}
      detail={isShowingDetail && <TaskItemDetail item={item} offset={offset} />}
      actions={<TaskActionPanel item={item} />}
    />
  );
};

const TaskItemDetail: React.FC<{
  item: SearchResult;
  offset: number | null;
}> = ({ item, offset }) => {
  const showDetail = useAtomValue(showDetailAtom);

  return (
    <List.Item.Detail
      markdown={
        showDetail === "all"
          ? `# ${item.title}${item.section ? "\n```\n" + item.section + "\n```\n" : ""}`
          : undefined
      }
      metadata={
        <List.Item.Detail.Metadata>
          <List.Item.Detail.Metadata.Label title="Title" text={item.title} />

          <List.Item.Detail.Metadata.Label
            title="Section"
            text={item.section}
          />

          <List.Item.Detail.Metadata.Separator />

          {item.keyword ? (
            <List.Item.Detail.Metadata.TagList title="Status">
              <List.Item.Detail.Metadata.TagList.Item
                text={item.keyword.value}
                color={
                  item.keyword.type === "DONE" ? Color.Green : Color.PrimaryText
                }
              />
            </List.Item.Detail.Metadata.TagList>
          ) : (
            <List.Item.Detail.Metadata.Label
              title="Status"
              text={{ value: "(no set)", color: Color.SecondaryText }}
            />
          )}

          {item.priority ? (
            <List.Item.Detail.Metadata.TagList title="Priority">
              <List.Item.Detail.Metadata.TagList.Item
                icon={Icon.Flag}
                text={item.priority}
                color={Color.Magenta}
              />
            </List.Item.Detail.Metadata.TagList>
          ) : (
            <List.Item.Detail.Metadata.Label
              title="Priority"
              text={{ value: "(no set)", color: Color.SecondaryText }}
            />
          )}

          {item.tags.length > 0 ? (
            <List.Item.Detail.Metadata.TagList title="Tags">
              {item.tags.map((tag) => (
                <List.Item.Detail.Metadata.TagList.Item key={tag} text={tag} />
              ))}
            </List.Item.Detail.Metadata.TagList>
          ) : (
            <List.Item.Detail.Metadata.Label
              title="Tags"
              text={{ value: "(no set)", color: Color.SecondaryText }}
            />
          )}

          <List.Item.Detail.Metadata.Separator />

          <List.Item.Detail.Metadata.Label
            title="Scheduled"
            text={
              item.planning.scheduled
                ? {
                    value: formatDetailDate(item.planning.scheduled),
                    color: Color.PrimaryText,
                  }
                : { value: "(no set)", color: Color.SecondaryText }
            }
          />

          <List.Item.Detail.Metadata.Label
            title="Deadline"
            text={
              item.planning.deadline
                ? {
                    value: formatDetailDate(item.planning.deadline),
                    color: Color.PrimaryText,
                  }
                : { value: "(no set)", color: Color.SecondaryText }
            }
          />

          <List.Item.Detail.Metadata.Label
            title="Closed"
            text={
              item.planning.closed
                ? {
                    value: formatDetailDate(item.planning.closed),
                    color: Color.PrimaryText,
                  }
                : { value: "(no set)", color: Color.SecondaryText }
            }
          />

          <List.Item.Detail.Metadata.Separator />

          {item.clocking.total_minutes > 0 ? (
            <List.Item.Detail.Metadata.TagList title="Sum">
              <List.Item.Detail.Metadata.TagList.Item
                icon={Icon.Stopwatch}
                text={item.clocking.total_minutes + ` minutes`}
                color={Color.Yellow}
              />
            </List.Item.Detail.Metadata.TagList>
          ) : (
            <List.Item.Detail.Metadata.Label
              title="Sum"
              text={{ value: "(no start)", color: Color.SecondaryText }}
            />
          )}

          {typeof offset === "number" ? (
            <List.Item.Detail.Metadata.TagList title="Clock">
              <List.Item.Detail.Metadata.TagList.Item
                icon={Icon.Play}
                text={formatMinutes(offset)}
                color={Color.Orange}
              />
            </List.Item.Detail.Metadata.TagList>
          ) : (
            <List.Item.Detail.Metadata.Label
              title="Clock"
              text={{ value: "(no start)", color: Color.SecondaryText }}
            />
          )}

          <List.Item.Detail.Metadata.Separator />

          <List.Item.Detail.Metadata.Label
            title="ID"
            text={item.url + ":" + item.line}
          />
        </List.Item.Detail.Metadata>
      }
    />
  );
};

const TaskActionPanel: React.FC<{ item: SearchResult }> = ({ item }) => {
  const isDone = item.keyword?.type === "DONE";
  const isRunning = !!item.clocking.start;

  const setShowingDetail = useSetAtom(showDetailAtom);
  const backend = useAtomValue(backendAtom);
  const { pop } = useNavigation();
  const priorities = useAtomValue(orgPrioritiesAtom);
  const tags = useAtomValue(orgTagsAtom);

  return (
    <ActionPanel title={item.title}>
      <Action
        icon={Icon.AppWindowSidebarLeft}
        title="Toggle detail"
        onAction={() => {
          setShowingDetail((x) => {
            const arr = ["no", "metadata-only", "all"] as const;
            return arr[(arr.indexOf(x) + 1) % 3];
          });
        }}
      />

      <Action
        icon={
          isDone
            ? { source: Icon.CircleProgress, tintColor: Color.Green }
            : Icon.CheckCircle
        }
        title={isDone ? "Change to TODO" : "Change to DONE"}
        shortcut={{ modifiers: ["cmd"], key: "." }}
        onAction={() => {
          backend
            .executeCommand("headline-update", {
              url: item.url,
              line: item.line,
              keyword: isDone ? "TODO" : "DONE",
            })
            .then(() => mutate("headline-search"))
            .then(() => showToast({ title: "Item status changed" }));
        }}
      />

      <ActionPanel.Submenu
        icon={Icon.Flag}
        title="Change priority"
        shortcut={{ modifiers: ["cmd"], key: "y" }}
      >
        {priorities.map((p) => (
          <Action
            key={p}
            title={"#" + p}
            onAction={() => {
              backend
                .executeCommand("headline-update", {
                  url: item.url,
                  line: item.line,
                  priority: p,
                })
                .then(() => mutate("headline-search"))
                .then(() => showToast({ title: "Item priority changed" }));
            }}
          />
        ))}
      </ActionPanel.Submenu>

      <ActionPanel.Submenu
        icon={Icon.Tag}
        title="Change tags"
        shortcut={{ modifiers: ["cmd"], key: "t" }}
      >
        {tags.map((tag) => {
          const includes = item.tags.includes(tag);
          return (
            <Action
              key={tag}
              icon={includes ? Icon.CircleFilled : Icon.Circle}
              title={tag}
              onAction={() => {
                backend
                  .executeCommand("headline-update", {
                    url: item.url,
                    line: item.line,
                    tags: includes
                      ? item.tags.filter((x) => x != tag)
                      : [...item.tags, tag],
                  })
                  .then(() => mutate("headline-search"))
                  .then(() => showToast({ title: "Item tags changed" }));
              }}
            />
          );
        })}
      </ActionPanel.Submenu>

      <Action.PickDate
        icon={Icon.Calendar}
        title="Set schedule date"
        shortcut={{ modifiers: ["cmd"], key: "s" }}
        onChange={(date) => {
          backend
            .executeCommand("headline-update", {
              url: item.url,
              line: item.line,
              scheduled: date
                ? lightFormat(date, "yyyy-MM-dd'T'HH:mm:ss")
                : null,
            })
            .then(() => mutate("headline-search"))
            .then(() => showToast({ title: "Item schedule date changed" }));
        }}
      />

      <Action
        icon={isRunning ? Icon.Pause : Icon.Stopwatch}
        title={isRunning ? "Clock out" : "Clock in"}
        shortcut={{ modifiers: ["cmd"], key: "l" }}
        onAction={() => {
          backend
            .executeCommand(isRunning ? "clocking-stop" : "clocking-start", {
              url: item.url,
              line: item.line,
            })
            .then(() => mutate("headline-search"))
            .then(() =>
              showToast({
                title: isRunning ? "Item clock in" : "Item clock out",
              })
            );
        }}
      />

      <Action.Push
        title="Edit"
        icon={Icon.Pencil}
        shortcut={{ modifiers: ["cmd"], key: "e" }}
        target={
          <TaskForm
            defaultValue={item}
            onSubmit={(values) => {
              backend
                .executeCommand("headline-update", {
                  url: item.url,
                  line: item.line,
                  ...values,
                })
                .then(() => {
                  mutate("headline-search");
                  showToast({ title: "TODO item updated" });
                  pop();
                });
            }}
          />
        }
      />

      <Action
        title="Duplicate"
        icon={Icon.CopyClipboard}
        shortcut={{ modifiers: ["cmd"], key: "d" }}
        onAction={() => {
          backend
            .executeCommand("headline-duplicate", {
              url: item.url,
              line: item.line,
            })
            .then(() => mutate("headline-search"))
            .then(() =>
              showToast({
                title: "Item duplicated",
                style: Toast.Style.Success,
              })
            );
        }}
      />

      <Action
        title="Remove"
        style={Action.Style.Destructive}
        icon={Icon.Trash}
        shortcut={{ modifiers: ["cmd"], key: "x" }}
        onAction={() => {
          backend
            .executeCommand("headline-remove", {
              url: item.url,
              line: item.line,
            })
            .then(() => mutate("headline-search"))
            .then(() =>
              showToast({
                title: "Item removed",
                style: Toast.Style.Success,
              })
            );
        }}
      />
    </ActionPanel>
  );
};

const formatDetailDate = (s: string): string => {
  const d = parse(s, "yyyy-MM-dd'T'HH:mm:ss", new Date());

  return (
    (getHours(d) === 0 && getMinutes(d)
      ? format(d, "yyyy eee LLL dd kk':'mm")
      : format(d, "yyyy eee LLL dd")) +
    " (" +
    formatDistanceToNowStrict(d, { addSuffix: true }) +
    ")"
  );
};
