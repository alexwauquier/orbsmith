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

export interface SimulationEnded {
  appId: string;
  /** `timer`: the auto-stop timer expired. `exited`: the dummy quit by itself. */
  reason: "timer" | "exited";
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

export function onSimulationEnded(handler: (ended: SimulationEnded) => void): Promise<UnlistenFn> {
  return listen<SimulationEnded>("simulation-ended", (event) => handler(event.payload));
}

/** Discord takes a moment to detect a process, so quests get a safety margin. */
export const QUEST_MARGIN_SECS = 2 * 60;

export function iconUrl(app: Pick<AppSummary, "id" | "iconHash">, size = 64): string | null {
  return app.iconHash
    ? `https://cdn.discordapp.com/app-icons/${app.id}/${app.iconHash}.png?size=${size}`
    : null;
}
