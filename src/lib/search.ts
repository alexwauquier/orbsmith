import type { AppSummary } from "$lib/api";

/**
 * Apps whose name or an alias contains `query`, case-insensitively. Names
 * starting with the query rank first, then shorter names, since a short exact
 * hit ("Minecraft") is more likely the one wanted than "Minecraft Dungeons".
 */
export function searchApps(apps: AppSummary[], query: string, limit: number): AppSummary[] {
  const needle = query.trim().toLowerCase();
  if (!needle) return [];

  const hits: { app: AppSummary; rank: number }[] = [];
  for (const app of apps) {
    const name = app.name.toLowerCase();
    let rank: number;
    if (name === needle) rank = 0;
    else if (name.startsWith(needle)) rank = 1;
    else if (name.includes(needle)) rank = 2;
    else if (app.aliases.some((alias) => alias.toLowerCase().includes(needle))) rank = 3;
    else continue;
    hits.push({ app, rank });
  }

  hits.sort(
    (a, b) =>
      a.rank - b.rank ||
      a.app.name.length - b.app.name.length ||
      a.app.name.localeCompare(b.app.name)
  );
  return hits.slice(0, limit).map((hit) => hit.app);
}
