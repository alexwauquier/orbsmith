import {
  addRecent,
  parseLibrary,
  rememberRun,
  toggleFavorite,
  type LastRun,
  type Library
} from "$lib/library";

const STORAGE_KEY = "orbsmith.library";

/**
 * Favorites, recently started apps and how each was last started, kept in the
 * webview's local storage, which lives in the app data dir.
 */
export class LibraryStore {
  favorites = $state<string[]>([]);
  recent = $state<string[]>([]);
  lastRuns = $state<Record<string, LastRun>>({});

  constructor() {
    const stored = parseLibrary(read());
    this.favorites = stored.favorites;
    this.recent = stored.recent;
    this.lastRuns = stored.lastRuns;
  }

  lastRun(id: string): LastRun | undefined {
    return this.lastRuns[id];
  }

  /** Marks `id` as used and remembers how it was started. */
  markStarted(id: string, run: LastRun) {
    this.lastRuns = rememberRun(this.lastRuns, id, run);
    this.markUsed(id);
  }

  isFavorite(id: string): boolean {
    return this.favorites.includes(id);
  }

  toggleFavorite(id: string) {
    this.favorites = toggleFavorite(this.favorites, id);
    this.save();
  }

  markUsed(id: string) {
    this.recent = addRecent(this.recent, id);
    this.save();
  }

  private save() {
    const library: Library = {
      favorites: this.favorites,
      recent: this.recent,
      lastRuns: this.lastRuns
    };
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(library));
    } catch {
      // Storage can be unavailable; the lists then last for this session only.
    }
  }
}

function read(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}
