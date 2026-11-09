import { describe, expect, it } from "vitest";
import type { AppSummary } from "$lib/api";
import { normalize, searchApps } from "$lib/search";

function app(name: string, aliases: string[] = []): AppSummary {
  const executable = `${name}.exe`;
  return { id: name, name, iconHash: null, aliases, executable, executables: [executable] };
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

  it("ignores punctuation, accents and trademark signs", () => {
    const games = [
      app("Plants vs. Zombies™: Replanted"),
      app("Pokémon® Legends"),
      app("Half-Life 2")
    ];
    expect(names(searchApps(games, "plants vs zombies", 10))).toEqual([
      "Plants vs. Zombies™: Replanted"
    ]);
    expect(names(searchApps(games, "pokemon", 10))).toEqual(["Pokémon® Legends"]);
    expect(names(searchApps(games, "halflife", 10))).toEqual(["Half-Life 2"]);
    expect(names(searchApps(games, "half life 2", 10))).toEqual(["Half-Life 2"]);
  });

  it("finds nothing for a query of punctuation only", () => {
    expect(searchApps(apps, "?!", 10)).toEqual([]);
  });
});

describe("normalize", () => {
  it("keeps letters and digits of any script", () => {
    expect(normalize("Pokémon™: Let's Go!")).toBe("pokemon let s go");
    expect(normalize("  原神  ")).toBe("原神");
    expect(normalize("NieR:Automata™")).toBe("nier automata");
  });
});
