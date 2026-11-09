import { describe, expect, it } from "vitest";
import type { AppSummary } from "$lib/api";
import { searchApps } from "$lib/search";

function app(name: string, aliases: string[] = []): AppSummary {
  return { id: name, name, iconHash: null, aliases, executable: `${name}.exe` };
}

const names = (apps: AppSummary[]) => apps.map((found) => found.name);

describe("searchApps", () => {
  const apps = [
    app("Minecraft Dungeons"),
    app("Minecraft"),
    app("Legends of Minecraft"),
    app("Steve Simulator", ["minecraft-like"]),
    app("Rocket League")
  ];

  it("finds nothing for a blank query", () => {
    expect(searchApps(apps, "", 10)).toEqual([]);
    expect(searchApps(apps, "   ", 10)).toEqual([]);
  });

  it("ranks exact, then prefix, then substring, then alias matches", () => {
    expect(names(searchApps(apps, "minecraft", 10))).toEqual([
      "Minecraft",
      "Minecraft Dungeons",
      "Legends of Minecraft",
      "Steve Simulator"
    ]);
  });

  it("ignores case and surrounding spaces", () => {
    expect(names(searchApps(apps, "  ROCKET ", 10))).toEqual(["Rocket League"]);
  });

  it("puts shorter names first within a rank, then sorts alphabetically", () => {
    const sameRank = [app("Rocket Rats"), app("Rocket Arena"), app("Rocket League")];
    expect(names(searchApps(sameRank, "rocket", 10))).toEqual([
      "Rocket Rats",
      "Rocket Arena",
      "Rocket League"
    ]);
  });

  it("returns at most `limit` results", () => {
    expect(searchApps(apps, "minecraft", 2)).toHaveLength(2);
  });
});
