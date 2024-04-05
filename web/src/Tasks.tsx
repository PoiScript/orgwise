import clsx from "clsx";
import { useAtom, useSetAtom } from "jotai/react";
import {
  CheckCircle2,
  Circle,
  CircleDashed,
  Clock,
  Copy,
  Play,
  Tag,
  Trash2,
} from "lucide-react";
import React from "react";
import useSWR, { mutate } from "swr";

import { DropdownMenuDemo } from "@/components/headline-dialog";
import { Badge } from "@/components/ui/badge";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Separator } from "@/components/ui/separator";
import { SearchResult, selectedAtom } from "./atom";
import executeCommand from "./command";

export const Tasks: React.FC = () => {
  const { data, isLoading, error } = useSWR<SearchResult[]>("headline-search");

  const [selected, setSelected] = useAtom(selectedAtom);

  if (isLoading || error) return;

  return (
    <div className="w-full">
      <Dialog open={!!selected} onOpenChange={() => setSelected(null)}>
        {data.map((item) => (
          <div className="" key={item.url + "#" + item.line}>
            <ListItem item={item} />
            <Separator />
          </div>
        ))}

        <DialogContent
          className={clsx("min-w-full sm:min-w-[600px] p-0 gap-0")}
        >
          <DropdownMenuDemo
            item={selected!}
            onClose={() => setSelected(null)}
          />
        </DialogContent>
      </Dialog>
    </div>
  );
};

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { parse } from "date-fns";
import { CountDown } from "./components/Clocking";
import { formatMinutes } from "./lib/utils";

const ListItem: React.FC<{ item: SearchResult }> = ({ item }) => {
  const select = useSetAtom(selectedAtom);

  return (
    <ContextMenu>
      <ContextMenuTrigger>
        <div
          tabIndex={0}
          className="p-4 flex gap-2 cursor-pointer items-center hover:bg-accent focus:outline-0 focus:bg-accent"
          onClick={() => select(item)}
        >
          {item.keyword ? (
            item.keyword.type === "TODO" ? (
              <Circle size={20} />
            ) : (
              <CheckCircle2 size={20} />
            )
          ) : (
            <CircleDashed size={20} />
          )}

          {item.priority && (
            <Badge variant="secondary"> #{item.priority} </Badge>
          )}

          <div>{item.title}</div>

          {Array.isArray(item.tags) && (
            <>
              {item.tags.map((tag) => (
                <div className="rounded border px-1.5 py-1 text-xs font-semibold text-sm items-center leading-none inline-flex gap-1">
                  <Tag size={12} />
                  {tag}
                </div>
              ))}
            </>
          )}

          {item.clocking.total_minutes > 0 && (
            <div className="rounded border px-1.5 py-1 text-xs font-semibold text-sm items-center leading-none inline-flex gap-1">
              <Clock size={12} />
              {formatMinutes(item.clocking.total_minutes)}
            </div>
          )}

          {item.clocking.start && (
            <div className="rounded border px-1.5 py-1 text-xs font-semibold text-sm items-center leading-none inline-flex gap-1">
              <Play size={12} />
              <CountDown
                start={parse(
                  item.clocking.start,
                  "yyyy-MM-dd'T'HH:mm:ss",
                  new Date()
                )}
              />
            </div>
          )}
        </div>
      </ContextMenuTrigger>

      <ContextMenuContent>
        <ContextMenuItem
          onClick={() =>
            executeCommand("headline-duplicate", {
              url: item.url,
              line: item.line,
            }).then(() => {
              mutate("headline-search");
            })
          }
        >
          <Copy className="h-4 w-4 mr-1" />
          Duplicate
        </ContextMenuItem>

        {item.clocking.start ? (
          <ContextMenuItem
            onClick={() =>
              executeCommand("clocking-stop", {
                url: item.url,
                line: item.line,
              }).then(() => {
                mutate("headline-search");
                mutate("clocking-status");
              })
            }
          >
            <Play className="h-4 w-4 mr-1" />
            Clock Out
          </ContextMenuItem>
        ) : (
          <ContextMenuItem
            onClick={() =>
              executeCommand("clocking-start", {
                url: item.url,
                line: item.line,
              }).then(() => {
                mutate("headline-search");
                mutate("clocking-status");
              })
            }
          >
            <Play className="h-4 w-4 mr-1" />
            Clock In
          </ContextMenuItem>
        )}

        <ContextMenuItem
          className="text-red-500"
          onClick={() =>
            executeCommand("headline-remove", {
              url: item.url,
              line: item.line,
            }).then(() => {
              mutate("headline-search");
            })
          }
        >
          <Trash2 className="h-4 w-4 mr-1" />
          Remove
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};
