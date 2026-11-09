import type { AppSummary } from "$lib/api";

/**
 * Lowercases `text` and drops what people don't type: accents, ™ ® ©, and
 * punctuation, which becomes a space. "Pokémon™: Let's Go!" reads
 * "pokemon let s go".
 */
export function normalize(text: string): string {
  return text
    .replace(/[™®©]/g, "")
    .normalize("NFKD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim();
}

interface Keys {
  name: string;
  /** The name without spaces, so "halflife" finds "Half-Life". */
  compact: string;
  aliases: string[];
}

// Apps are replaced wholesale, never mutated, so their keys can be cached.
const keyCache = new WeakMap<AppSummary, Keys>();

function keys(app: AppSummary): Keys {
  let cached = keyCache.get(app);
  if (!cached) {
    const name = normalize(app.name);
    cached = { name, compact: name.replaceAll(" ", ""), aliases: app.aliases.map(normalize) };
    keyCache.set(app, cached);
  }
  return cached;
}

/**
 * Apps whose name or an alias contains `query`, ignoring case, accents and
 * punctuation. Names starting with the query rank first, then shorter names,
 * since a short exact hit ("Minecraft") is more likely the one wanted than
 * "Minecraft Dungeons".
 */
export function searchApps(apps: AppSummary[], query: string, limit: number): AppSummary[] {
  const needle = normalize(query);
  if (!needle) return [];
  const compactNeedle = needle.replaceAll(" ", "");

  const hits: { app: AppSummary; rank: number }[] = [];
  for (const app of apps) {
    const { name, compact, aliases } = keys(app);
    let rank: number;
    if (name === needle) rank = 0;
    else if (name.startsWith(needle)) rank = 1;
    else if (name.includes(needle) || compact.includes(compactNeedle)) rank = 2;
    else if (aliases.some((alias) => alias.includes(needle))) rank = 3;
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
