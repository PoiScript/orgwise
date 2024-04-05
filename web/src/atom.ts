import { atom } from "jotai";

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
  clocking: { total_minutes: number; start?: string };
};

export const enum ViewMode {
  Tasks,
  CalendarDay,
}

export const viewModeAtom = atom<ViewMode>(ViewMode.Tasks);

export const filtersAtom = atom({});

export const selectedAtom = atom(null as SearchResult | null);
