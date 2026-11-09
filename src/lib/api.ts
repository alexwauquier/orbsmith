import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** An app that can be simulated on the current platform. */
export interface AppSummary {
  id: string;
  name: string;
  iconHash: string | null;
  aliases: string[];
  /** Executable path the dummy runs as, e.g. `game/bin/game.exe`. */
  executable: string;
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
  /** Start every queued quest at once instead of one after another. */
  questsInParallel: boolean;
}

/** Mirrors `orbsmith_core::quests::QuestInfo`. */
export interface Quest {
  appId: string;
  appName: string;
  /** How long the game runs, auto-stop margin included. */
  durationSecs: number;
}

/** Mirrors `orbsmith_core::QuestsState`. */
export interface QuestsState {
  /** Whether the queue is being worked through. */
  running: boolean;
  /** Quests whose game is running. */
  active: Quest[];
  /** Quests waiting their turn, in order. */
  queued: Quest[];
}

/** Uses the cached list unless `refresh` is set or there is no cache yet. */
export function getApps(refresh = false): Promise<AppList> {
  return invoke("get_apps", { refresh });
}

export function listSimulations(): Promise<Simulation[]> {
  return invoke("list_simulations");
}

/** With `durationSecs`, the simulation stops automatically after that long. */
export function startSimulation(appId: string, durationSecs?: number): Promise<Simulation> {
  return invoke("start_simulation", { appId, durationSecs: durationSecs ?? null });
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

/**
 * Added to a quest's duration so the timer starts once the process is up;
 * Discord detects it almost immediately.
 */
export const DETECTION_DELAY_SECS = 1;

/** Longest auto-stop timer the backend accepts (`MAX_DURATION`, 7 days). */
export const MAX_TIMER_MINUTES = 7 * 24 * 60;

export function iconUrl(app: Pick<AppSummary, "id" | "iconHash">, size = 64): string | null {
  return app.iconHash
    ? `https://cdn.discordapp.com/app-icons/${app.id}/${app.iconHash}.png?size=${size}`
    : null;
}

export function getSettings(): Promise<Settings> {
  return invoke("get_settings");
}

export function setSettings(settings: Settings): Promise<Settings> {
  return invoke("set_settings", { settings });
}

export function listQuests(): Promise<QuestsState> {
  return invoke("list_quests");
}

/** `durationSecs` includes the auto-stop margin, like `startSimulation`. */
export function addQuest(appId: string, durationSecs: number): Promise<QuestsState> {
  return invoke("add_quest", { appId, durationSecs });
}

export function removeQuest(appId: string): Promise<QuestsState> {
  return invoke("remove_quest", { appId });
}

export function startQuests(): Promise<QuestsState> {
  return invoke("start_quests");
}

/** Pauses the quests and stops their games; they go back to the front. */
export function stopQuests(): Promise<QuestsState> {
  return invoke("stop_quests");
}

/** Mirrors `QUESTS_CHANGED_EVENT` in `src-tauri/src/lib.rs`. */
export function onQuestsChanged(handler: () => void): Promise<UnlistenFn> {
  return listen("quests-changed", () => handler());
}
