import { getLibrary, setLibrary } from "$lib/api";
import {
  addRecent,
  initialLibrary,
  rememberRun,
  toggleFavorite,
  type LastRun,
  type Library
} from "$lib/library";

/** Where versions up to 1.0.0 kept the library. */
const LEGACY_STORAGE_KEY = "orbsmith.library";

/**
 * Favorites, recently started apps and how each was last started, saved by
 * the backend as `library.json` in the app data dir. Changes made before
 * `load` finishes are ignored, since loading would replace them.
 */
export class LibraryStore {
  favorites = $state<string[]>([]);
  recent = $state<string[]>([]);
  lastRuns = $state<Record<string, LastRun>>({});

  #loaded = false;
  /** Saves run one after another, so an older one never lands last. */
  #saving: Promise<boolean> = Promise.resolve(true);

  /**
   * Reads the saved library. The first time, it moves over what local storage
   * held, and clears it once saved. A corrupt file is reported, and replaced
   * on the next change.
   */
  async load() {
    try {
      const { library, migrate } = initialLibrary(await getLibrary(), readLegacy());
      this.favorites = library.favorites;
      this.recent = library.recent;
      this.lastRuns = library.lastRuns;
      if (migrate && (await this.save())) clearLegacy();
    } finally {
      this.#loaded = true;
    }
  }

  lastRun(id: string): LastRun | undefined {
    return this.lastRuns[id];
  }

  /** Marks `id` as used and remembers how it was started. */
  markStarted(id: string, run: LastRun) {
    if (!this.#loaded) return;
    this.lastRuns = rememberRun(this.lastRuns, id, run);
    this.markUsed(id);
  }

  isFavorite(id: string): boolean {
    return this.favorites.includes(id);
  }

  toggleFavorite(id: string) {
    if (!this.#loaded) return;
    this.favorites = toggleFavorite(this.favorites, id);
    void this.save();
  }

  clearRecent() {
    if (!this.#loaded) return;
    this.recent = [];
    void this.save();
  }

  markUsed(id: string) {
    if (!this.#loaded) return;
    this.recent = addRecent(this.recent, id);
    void this.save();
  }

  /** Resolves to whether the save succeeded. */
  private save(): Promise<boolean> {
    const library: Library = $state.snapshot({
      favorites: this.favorites,
      recent: this.recent,
      lastRuns: this.lastRuns
    });
    this.#saving = this.#saving.then(() =>
      setLibrary(library).then(
        () => true,
        (err: unknown) => {
          // The lists then last for this session only.
          console.warn("failed to save the library:", err);
          return false;
        }
      )
    );
    return this.#saving;
  }
}

function readLegacy(): string | null {
  try {
    return localStorage.getItem(LEGACY_STORAGE_KEY);
  } catch {
    return null;
  }
}

function clearLegacy() {
  try {
    localStorage.removeItem(LEGACY_STORAGE_KEY);
  } catch {
    // Harmless: the saved library takes precedence from now on.
  }
}
