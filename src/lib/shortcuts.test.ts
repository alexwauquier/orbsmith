import { describe, expect, it } from "vitest";
import { moveHighlight, searchShortcut, windowShortcut, type KeyPress } from "$lib/shortcuts";

function press(key: string, modifiers: Partial<KeyPress> = {}): KeyPress {
  return { key, ctrlKey: false, metaKey: false, altKey: false, ...modifiers };
}

describe("windowShortcut", () => {
  it("focuses the search with Ctrl+F, whatever the case", () => {
    expect(windowShortcut(press("f", { ctrlKey: true }), true)).toBe("focus-search");
    expect(windowShortcut(press("F", { ctrlKey: true }), false)).toBe("focus-search");
  });

  it("focuses the search with / unless typing", () => {
    expect(windowShortcut(press("/"), false)).toBe("focus-search");
    expect(windowShortcut(press("/"), true)).toBeNull();
  });

  it("opens the settings with Ctrl+,", () => {
    expect(windowShortcut(press(",", { ctrlKey: true }), true)).toBe("open-settings");
  });

  it("ignores plain keys and Alt combinations", () => {
    expect(windowShortcut(press("f"), false)).toBeNull();
    expect(windowShortcut(press("f", { ctrlKey: true, altKey: true }), false)).toBeNull();
  });
});

describe("searchShortcut", () => {
  it("maps arrows, Enter and Escape", () => {
    expect(searchShortcut(press("ArrowDown"))).toBe("next");
    expect(searchShortcut(press("ArrowUp"))).toBe("previous");
    expect(searchShortcut(press("Enter"))).toBe("start");
    expect(searchShortcut(press("Escape"))).toBe("clear");
  });

  it("leaves other keys and modified keys to the input", () => {
    expect(searchShortcut(press("a"))).toBeNull();
    expect(searchShortcut(press("Enter", { ctrlKey: true }))).toBeNull();
  });
});

describe("moveHighlight", () => {
  it("moves within the rows shown", () => {
    expect(moveHighlight(0, 1, 3)).toBe(1);
    expect(moveHighlight(2, 1, 3)).toBe(2);
    expect(moveHighlight(0, -1, 3)).toBe(0);
  });

  it("stays at 0 without rows, and clamps after the list shrank", () => {
    expect(moveHighlight(4, 1, 0)).toBe(0);
    expect(moveHighlight(9, -1, 3)).toBe(2);
  });
});
