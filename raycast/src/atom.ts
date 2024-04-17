import { environment, getPreferenceValues } from "@raycast/api";
import { atom } from "jotai";
import { existsSync, readFileSync } from "node:fs";
import { readFile, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import { URI } from "vscode-uri";

import { Backend, initSync } from "../../pkg/orgwise";

const preferencesAtom = atom<Preferences>(getPreferenceValues);

export const orgFileAtom = atom((get) =>
  URI.file(get(preferencesAtom).orgTodoFile)
);

export const orgTodoKeywordsAtom = atom((get) =>
  get(preferencesAtom)
    .orgTodoKeywords.split(",")
    .map((x) => x.trim().toUpperCase())
);

export const orgDoneKeywordsAtom = atom((get) =>
  get(preferencesAtom)
    .orgDoneKeywords.split(",")
    .map((x) => x.trim().toUpperCase())
);

export const orgTagsAtom = atom((get) =>
  get(preferencesAtom)
    .orgTags.split(",")
    .map((x) => x.trim())
);

export const orgPrioritiesAtom = atom((get) =>
  get(preferencesAtom)
    .orgPriorities.split(",")
    .map((x) => x.trim().slice(0, 1).toUpperCase())
);

export const backendAtom = atom<Backend>((get) => {
  const buffer = readFileSync(`${environment.assetsPath}/orgwise_bg.wasm`);

  initSync(buffer);

  const backend = new Backend({
    homeDir: () => URI.file(homedir()).toString() + "/",

    readToString: async (url: string) => {
      const path = URI.parse(url).fsPath;
      if (existsSync(path)) {
        return readFile(path, { encoding: "utf-8" });
      } else {
        return "";
      }
    },

    write: (url: string, content: string) =>
      writeFile(URI.parse(url).fsPath, content),
  });

  backend.setOptions({
    todoKeywords: get(orgTodoKeywordsAtom),
    doneKeywords: get(orgDoneKeywordsAtom),
  });

  const url = get(orgFileAtom);

  backend.addOrgFile(url.toString(), readFileSync(url.fsPath, "utf-8"));

  return backend;
});
