/** The parts of a `KeyboardEvent` the shortcuts look at. */
export interface KeyPress {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
}

export type WindowShortcut = "focus-search" | "open-settings";

/**
 * A shortcut available anywhere in the window: Ctrl+F or `/` focuses the
 * search, Ctrl+, opens the settings. `/` is left alone while typing.
 */
export function windowShortcut(press: KeyPress, typing: boolean): WindowShortcut | null {
  const ctrl = (press.ctrlKey || press.metaKey) && !press.altKey;
  if (ctrl && press.key.toLowerCase() === "f") return "focus-search";
  if (ctrl && press.key === ",") return "open-settings";
  if (!ctrl && !press.altKey && press.key === "/" && !typing) return "focus-search";
  return null;
}

export type SearchShortcut = "next" | "previous" | "start" | "clear";

/** Keys handled in the search box: arrows pick a game, Enter starts it, Escape clears. */
export function searchShortcut(press: KeyPress): SearchShortcut | null {
  if (press.ctrlKey || press.metaKey || press.altKey) return null;
  switch (press.key) {
    case "ArrowDown":
      return "next";
    case "ArrowUp":
      return "previous";
    case "Enter":
      return "start";
    case "Escape":
      return "clear";
    default:
      return null;
  }
}

/** Moves the highlighted row by `step`, staying within the `count` rows shown. */
export function moveHighlight(index: number, step: number, count: number): number {
  if (count === 0) return 0;
  return Math.min(count - 1, Math.max(0, index + step));
}
