/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {
  /** File - Path to your org todo file */
  "orgTodoFile": string,
  /** TODO keywords - separated by comma */
  "orgTodoKeywords": string,
  /** DONE keywords - separated by comma */
  "orgDoneKeywords": string,
  /** Tags - separated by comma */
  "orgTags": string,
  /** Priorities - separated by comma */
  "orgPriorities": string
}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `list` command */
  export type List = ExtensionPreferences & {}
  /** Preferences accessible in the `create` command */
  export type Create = ExtensionPreferences & {}
}

declare namespace Arguments {
  /** Arguments passed to the `list` command */
  export type List = {}
  /** Arguments passed to the `create` command */
  export type Create = {
  /** your title */
  "title": string
}
}

