import { addRecent, parseLibrary, toggleFavorite, type Library } from "$lib/library";

const STORAGE_KEY = "orbsmith.library";

/**
 * Favorites and recently started apps, kept in the webview's local storage,
 * which lives in the app data dir.
 */
export class LibraryStore {
  favorites = $state<string[]>([]);
  recent = $state<string[]>([]);

  constructor() {
    const stored = parseLibrary(read());
    this.favorites = stored.favorites;
    this.recent = stored.recent;
  }

  isFavorite(id: string): boolean {
    return this.favorites.includes(id);
  }

  toggleFavorite(id: string) {
    this.favorites = toggleFavorite(this.favorites, id);
    this.save();
  }

  markStarted(id: string) {
    this.recent = addRecent(this.recent, id);
    this.save();
  }

  private save() {
    const library: Library = { favorites: this.favorites, recent: this.recent };
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
