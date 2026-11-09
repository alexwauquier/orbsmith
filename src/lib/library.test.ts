import { describe, expect, it } from "vitest";
import {
  RECENT_LIMIT,
  addRecent,
  emptyLibrary,
  parseLibrary,
  resolveApps,
  toggleFavorite
} from "$lib/library";

describe("addRecent", () => {
  it("moves the app to the front without duplicating it", () => {
    expect(addRecent(["a", "b", "c"], "b")).toEqual(["b", "a", "c"]);
    expect(addRecent([], "a")).toEqual(["a"]);
  });

  it("keeps at most the limit", () => {
    expect(addRecent(["a", "b", "c"], "d", 3)).toEqual(["d", "a", "b"]);
  });
});

describe("toggleFavorite", () => {
  it("adds new favorites first and removes existing ones", () => {
    expect(toggleFavorite(["a"], "b")).toEqual(["b", "a"]);
    expect(toggleFavorite(["b", "a"], "b")).toEqual(["a"]);
  });
});

describe("parseLibrary", () => {
  it("starts empty without stored data or with invalid JSON", () => {
    expect(parseLibrary(null)).toEqual(emptyLibrary());
    expect(parseLibrary("not json")).toEqual(emptyLibrary());
    expect(parseLibrary("42")).toEqual(emptyLibrary());
    expect(parseLibrary("null")).toEqual(emptyLibrary());
  });

  it("keeps only string IDs, without duplicates", () => {
    const raw = JSON.stringify({ favorites: ["a", 1, "a", null, "b"], recent: "a" });
    expect(parseLibrary(raw)).toEqual({ favorites: ["a", "b"], recent: [] });
  });

  it("trims the recent list to the limit", () => {
    const recent = Array.from({ length: RECENT_LIMIT + 3 }, (_, i) => `app-${i}`);
    expect(parseLibrary(JSON.stringify({ recent })).recent).toHaveLength(RECENT_LIMIT);
  });
});

describe("resolveApps", () => {
  it("keeps the order and skips IDs missing from the catalog", () => {
    const byId = new Map([
      ["a", { id: "a" }],
      ["c", { id: "c" }]
    ]);
    expect(resolveApps(["c", "b", "a"], byId)).toEqual([{ id: "c" }, { id: "a" }]);
  });
});
