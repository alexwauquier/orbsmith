/** The user's favorite and recently started apps, as app IDs, newest first. */
export interface Library {
  favorites: string[];
  recent: string[];
}

export const RECENT_LIMIT = 8;

export function emptyLibrary(): Library {
  return { favorites: [], recent: [] };
}

/** Moves `id` to the front of the recent list, keeping at most `limit` IDs. */
export function addRecent(recent: string[], id: string, limit = RECENT_LIMIT): string[] {
  return [id, ...recent.filter((other) => other !== id)].slice(0, limit);
}

/** Adds `id` to the front of the favorites, or removes it if already there. */
export function toggleFavorite(favorites: string[], id: string): string[] {
  return favorites.includes(id) ? favorites.filter((other) => other !== id) : [id, ...favorites];
}

/** Reads a stored library, ignoring anything malformed rather than failing. */
export function parseLibrary(raw: string | null): Library {
  if (!raw) return emptyLibrary();
  try {
    const value: unknown = JSON.parse(raw);
    if (typeof value !== "object" || value === null) return emptyLibrary();
    const record = value as Record<string, unknown>;
    return {
      favorites: [...new Set(stringIds(record.favorites))],
      recent: [...new Set(stringIds(record.recent))].slice(0, RECENT_LIMIT)
    };
  } catch {
    return emptyLibrary();
  }
}

function stringIds(field: unknown): string[] {
  return Array.isArray(field) ? field.filter((id): id is string => typeof id === "string") : [];
}

/** Resolves IDs to apps, in order, skipping those no longer in the catalog. */
export function resolveApps<T extends { id: string }>(ids: string[], byId: Map<string, T>): T[] {
  return ids.flatMap((id) => {
    const app = byId.get(id);
    return app ? [app] : [];
  });
}
