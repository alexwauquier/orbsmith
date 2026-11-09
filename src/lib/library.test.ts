import { describe, expect, it } from "vitest";
import {
  LAST_RUNS_LIMIT,
  RECENT_LIMIT,
  addRecent,
  emptyLibrary,
  parseLibrary,
  rememberRun,
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
    expect(parseLibrary(raw)).toEqual({ favorites: ["a", "b"], recent: [], lastRuns: {} });
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

describe("rememberRun", () => {
  it("records the duration and the executable", () => {
    expect(rememberRun({}, "a", { minutes: 30, executable: "a.exe" })).toEqual({
      a: { minutes: 30, executable: "a.exe" }
    });
  });

  it("keeps the previous duration when the run had no timer", () => {
    const runs = rememberRun({}, "a", { minutes: 30 });
    expect(rememberRun(runs, "a", {})).toEqual({ a: { minutes: 30 } });
  });

  it("forgets the executable when the default is used again", () => {
    const runs = rememberRun({}, "a", { minutes: 30, executable: "a.exe" });
    expect(rememberRun(runs, "a", { minutes: 15 })).toEqual({ a: { minutes: 15 } });
  });

  it("keeps the most recently started apps", () => {
    let runs = rememberRun({}, "a", { minutes: 1 });
    runs = rememberRun(runs, "b", { minutes: 2 });
    runs = rememberRun(runs, "a", { minutes: 3 });
    runs = rememberRun(runs, "c", { minutes: 4 }, 2);
    expect(Object.keys(runs)).toEqual(["a", "c"]);
  });
});

describe("parseLibrary lastRuns", () => {
  it("keeps valid entries only", () => {
    const raw = JSON.stringify({
      lastRuns: {
        a: { minutes: 30, executable: "a.exe" },
        b: { minutes: -1, executable: 3 },
        c: "nope"
      }
    });
    expect(parseLibrary(raw).lastRuns).toEqual({ a: { minutes: 30, executable: "a.exe" }, b: {} });
  });

  it("caps the number of entries", () => {
    const lastRuns = Object.fromEntries(
      Array.from({ length: LAST_RUNS_LIMIT + 5 }, (_, i) => [`app-${i}`, { minutes: 1 }])
    );
    expect(Object.keys(parseLibrary(JSON.stringify({ lastRuns })).lastRuns)).toHaveLength(
      LAST_RUNS_LIMIT
    );
  });
});
