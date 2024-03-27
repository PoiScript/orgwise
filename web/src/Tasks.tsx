import { useAtom, useAtomValue, useSetAtom } from "jotai/react";
import { CheckCircle2, Circle, CircleDashed, Tag } from "lucide-react";
import React from "react";

import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Separator } from "@/components/ui/separator";
import { SearchResult, itemsAtom } from "./atom";
import { DropdownMenuDemo } from "./components/headline-dialog";
import { atom } from "jotai";
import { Badge } from "./components/ui/badge";
import clsx from "clsx";

const selectedAtom = atom(null as SearchResult | null);

export const Tasks: React.FC = () => {
  const data = useAtomValue(itemsAtom);
  const [selected, setSelected] = useAtom(selectedAtom);

  return (
    <div className="w-full">
      <Dialog open={!!selected} onOpenChange={() => setSelected(null)}>
        {data.map((item) => (
          <div className="" key={item.url + item.line}>
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

const ListItem: React.FC<{ item: SearchResult }> = ({ item }) => {
  const select = useSetAtom(selectedAtom);

  return (
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

      {item.priority && <Badge variant="secondary"> #{item.priority} </Badge>}

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
    </div>
  );
};
