import { invoke } from "@tauri-apps/api/core";
import type { Library } from "$lib/library";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** An app that can be simulated on the current platform. */
export interface AppSummary {
  id: string;
  name: string;
  iconHash: string | null;
  aliases: string[];
  /** Executable path the dummy runs as by default, e.g. `game/bin/game.exe`. */
  executable: string;
  /** Every executable that can be simulated, the default first. */
  executables: string[];
}

export interface AppList {
  apps: AppSummary[];
  /** Milliseconds since the Unix epoch. */
  fetchedAt: number;
}

export interface Simulation {
  appId: string;
  appName: string;
  executable: string;
  pid: number;
  /** Milliseconds since the Unix epoch. */
  startedAt: number;
  /** Milliseconds since the Unix epoch, or `null` to run until stopped. */
  stopAt: number | null;
}

/** Mirrors `orbsmith_core::Settings`. */
export interface Settings {
  /** Start every queued game at once instead of one after another. */
  parallelQueue: boolean;
  /** Look for a new release at startup. */
  checkForUpdates: boolean;
  /** Wait between two games of the queue when they run one after another. */
  queueGapSecs: number;
  /** Queued games run a little longer than their duration, and waits vary. */
  varyQueueTimes: boolean;
}

/** Longest wait between queued games (`orbsmith_core::queue::MAX_GAP`, 24 hours). */
export const MAX_GAP_MINUTES = 24 * 60;

/** Mirrors `UpdateInfo` in `src-tauri/src/update.rs`. */
export interface UpdateInfo {
  version: string;
  currentVersion: string;
  /** The release notes, as Markdown. */
  notes: string | null;
}

/** Mirrors `orbsmith_core::queue::QueueItemInfo`. */
export interface QueueItem {
  appId: string;
  appName: string;
  /** How long the game runs. */
  durationSecs: number;
  /** Time left after a pause interrupted the game, used when it resumes. */
  leftSecs: number | null;
}

/** Mirrors `orbsmith_core::QueueState`. */
export interface QueueState {
  /** Whether the queue is being worked through. */
  running: boolean;
  /** Paused with items left; `startQueue` resumes. */
  paused: boolean;
  /** Items whose game is running. */
  active: QueueItem[];
  /** Items waiting their turn, in order. */
  queued: QueueItem[];
  /** While waiting between two games: when the next one starts, in milliseconds since the Unix epoch. */
  nextStartAt: number | null;
}

/** Uses the cached list unless `refresh` is set or there is no cache yet. */
export function getApps(refresh = false): Promise<AppList> {
  return invoke("get_apps", { refresh });
}

export function listSimulations(): Promise<Simulation[]> {
  return invoke("list_simulations");
}

/**
 * With `durationSecs`, the simulation stops automatically after that long.
 * `executable` is one of the app's `executables`; the default one otherwise.
 */
export function startSimulation(
  appId: string,
  durationSecs?: number,
  executable?: string
): Promise<Simulation> {
  return invoke("start_simulation", {
    appId,
    durationSecs: durationSecs ?? null,
    executable: executable ?? null
  });
}

export function stopSimulation(appId: string): Promise<void> {
  return invoke("stop_simulation", { appId });
}

/** Stop `durationSecs` from now, or never with `null`. */
export function setSimulationTimer(
  appId: string,
  durationSecs: number | null
): Promise<Simulation> {
  return invoke("set_simulation_timer", { appId, durationSecs });
}

/**
 * Fires whenever simulations start or stop, including from the tray and when
 * they end on their own. Mirrors `SIMULATIONS_CHANGED_EVENT` in `src-tauri/src/lib.rs`.
 */
export function onSimulationsChanged(handler: () => void): Promise<UnlistenFn> {
  return listen("simulations-changed", () => handler());
}

/** Longest auto-stop timer the backend accepts (`MAX_DURATION`, 7 days). */
export const MAX_TIMER_MINUTES = 7 * 24 * 60;

export function iconUrl(app: Pick<AppSummary, "id" | "iconHash">, size = 64): string | null {
  return app.iconHash
    ? `https://cdn.discordapp.com/app-icons/${app.id}/${app.iconHash}.png?size=${size}`
    : null;
}

/** The saved library, or `null` before the first save. */
export function getLibrary(): Promise<Library | null> {
  return invoke("get_library");
}

export function setLibrary(library: Library): Promise<void> {
  return invoke("set_library", { library });
}

export function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export function setSettings(settings: Settings): Promise<Settings> {
  return invoke("set_settings", { settings });
}

/** Looks for a newer release; `null` when Orbsmith is up to date. */
export function checkForUpdate(): Promise<UpdateInfo | null> {
  return invoke("check_for_update");
}

/**
 * Installs the update found by the last check. Running games stop and the
 * queue is paused, then Orbsmith restarts, so this only settles on failure.
 */
export function installUpdate(): Promise<void> {
  return invoke("install_update");
}

export function listQueue(): Promise<QueueState> {
  return invoke("list_queue");
}

/** Queues `appId` to run for `durationSecs`, as `executable` or the default one. */
export function addToQueue(
  appId: string,
  durationSecs: number,
  executable?: string
): Promise<QueueState> {
  return invoke("add_to_queue", { appId, durationSecs, executable: executable ?? null });
}

export function removeFromQueue(appId: string): Promise<QueueState> {
  return invoke("remove_from_queue", { appId });
}

/** Empties the queue and stops its running games. */
export function clearQueue(): Promise<QueueState> {
  return invoke("clear_queue");
}

/** Moves a waiting game one place towards the front (`up`) or the back. */
export function moveInQueue(appId: string, up: boolean): Promise<QueueState> {
  return invoke("move_in_queue", { appId, up });
}

export function startQueue(): Promise<QueueState> {
  return invoke("start_queue");
}

/**
 * Stops the running games, keeping their time left: they go back to the
 * front and resume where they stopped with `startQueue`.
 */
export function pauseQueue(): Promise<QueueState> {
  return invoke("pause_queue");
}

/** Mirrors `QUEUE_CHANGED_EVENT` in `src-tauri/src/lib.rs`. */
export function onQueueChanged(handler: () => void): Promise<UnlistenFn> {
  return listen("queue-changed", () => handler());
}
