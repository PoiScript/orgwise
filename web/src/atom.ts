import { atom } from "jotai";
import fetch from "./command";

export type SearchResult = {
  title: string;
  section?: string;
  url: string;
  line: number;
  level: number;
  priority?: string;
  tags: string[];
  keyword?: { value: string; type: "DONE" | "TODO" };
  planning: { deadline?: string; scheduled?: string; closed?: string };
};

export const enum ViewMode {
  Tasks,
  CalendarDay,
}

export const viewModeAtom = atom<ViewMode>(ViewMode.Tasks);

export const loadingAtom = atom(false);

export const filtersAtom = atom({});

export const itemsAtom = atom<SearchResult[]>([]);

export const tagsAtom = atom((get) => {
  const tags = get(itemsAtom).reduce((acc, item) => {
    for (const tag of item.tags) {
      acc.add(tag);
    }
    return acc;
  }, new Set());
  return [...tags];
});

export const searchAtom = atom(null, async (get, set) => {
  set(loadingAtom, true);
  const items = await fetch<SearchResult[]>(
    "search-headline",
    get(filtersAtom)
  );
  set(itemsAtom, items);
  set(loadingAtom, false);
});

export const commandAtom = atom(
  null,
  async (get, set, command: string, argument: any) => {
    await fetch<SearchResult[]>(command, argument);
    const items = await fetch<SearchResult[]>(
      "search-headline",
      get(filtersAtom)
    );
    set(itemsAtom, items);
  }
);
