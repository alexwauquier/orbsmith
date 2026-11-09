/** How an app was last started, to start it the same way next time. */
export interface LastRun {
  /** The auto-stop duration, when one was set. */
  minutes?: number;
  /** A non-default executable, when one was chosen. */
  executable?: string;
}

/** The user's favorite and recently started apps, as app IDs, newest first. */
export interface Library {
  favorites: string[];
  recent: string[];
  /** Keyed by app ID, oldest first. */
  lastRuns: Record<string, LastRun>;
}

export const RECENT_LIMIT = 8;
export const LAST_RUNS_LIMIT = 100;

export function emptyLibrary(): Library {
  return { favorites: [], recent: [], lastRuns: {} };
}

/**
 * Records how `id` was started. A missing duration keeps the previous one, so
 * a run without a timer does not forget it. Keeps the `limit` most recent.
 */
export function rememberRun(
  lastRuns: Record<string, LastRun>,
  id: string,
  run: LastRun,
  limit = LAST_RUNS_LIMIT
): Record<string, LastRun> {
  const previous = lastRuns[id];
  const next: LastRun = { minutes: run.minutes ?? previous?.minutes, executable: run.executable };
  const entries = Object.entries(lastRuns).filter(([other]) => other !== id);
  entries.push([id, withoutUndefined(next)]);
  return Object.fromEntries(entries.slice(-limit));
}

function withoutUndefined(run: LastRun): LastRun {
  const clean: LastRun = {};
  if (run.minutes !== undefined) clean.minutes = run.minutes;
  if (run.executable !== undefined) clean.executable = run.executable;
  return clean;
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
      recent: [...new Set(stringIds(record.recent))].slice(0, RECENT_LIMIT),
      lastRuns: parseLastRuns(record.lastRuns)
    };
  } catch {
    return emptyLibrary();
  }
}

function parseLastRuns(field: unknown): Record<string, LastRun> {
  if (typeof field !== "object" || field === null || Array.isArray(field)) return {};
  const runs: [string, LastRun][] = [];
  for (const [id, value] of Object.entries(field)) {
    if (typeof value !== "object" || value === null) continue;
    const { minutes, executable } = value as Record<string, unknown>;
    const run: LastRun = {};
    if (typeof minutes === "number" && Number.isInteger(minutes) && minutes > 0) {
      run.minutes = minutes;
    }
    if (typeof executable === "string") run.executable = executable;
    runs.push([id, run]);
  }
  return Object.fromEntries(runs.slice(-LAST_RUNS_LIMIT));
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
