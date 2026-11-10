import type { QueueItem, QueueState, Settings, Simulation } from "$lib/api";

export interface QueueEstimate {
  /** When the last game should stop, in milliseconds since the Unix epoch. */
  endsAt: number;
  /** Varied times make the real end up to 10% later per game, and waits differ. */
  approximate: boolean;
}

/**
 * When the queue should be done if it runs (or is started) now, following the
 * backend's pacing (`orbsmith_core::queue::Queue::next`). Active games end at
 * their timer, which already includes any variation. `null` when an active
 * game has no timer, so its end is unknown.
 */
export function estimateQueue(
  queue: QueueState,
  settings: Pick<Settings, "parallelQueue" | "queueGapSecs" | "varyQueueTimes">,
  simulations: Simulation[],
  now: number
): QueueEstimate | null {
  if (queue.active.length === 0 && queue.queued.length === 0) return null;

  const stopAt = new Map(simulations.map((sim) => [sim.appId, sim.stopAt]));
  const activeEnds: number[] = [];
  for (const item of queue.active) {
    // Not listed yet right after it started: it runs for its whole time.
    const end = stopAt.has(item.appId) ? stopAt.get(item.appId) : now + runMs(item);
    if (end === null || end === undefined) return null;
    activeEnds.push(Math.max(now, end));
  }
  const approximate = settings.varyQueueTimes;

  if (settings.parallelQueue) {
    const ends = [...activeEnds, ...queue.queued.map((item) => now + runMs(item))];
    return { endsAt: Math.max(...ends), approximate };
  }

  const gapMs = settings.queueGapSecs * 1000;
  let at = now;
  if (activeEnds.length > 0) {
    at = Math.max(...activeEnds);
    if (queue.queued.length > 0) at += gapMs;
  } else if (queue.running && queue.nextStartAt !== null) {
    at = Math.max(now, queue.nextStartAt);
  }
  queue.queued.forEach((item, index) => {
    at += runMs(item);
    if (index < queue.queued.length - 1) at += gapMs;
  });
  return { endsAt: at, approximate };
}

/** How long a waiting game runs once started: what a pause left it, or its duration. */
function runMs(item: QueueItem): number {
  return (item.leftSecs ?? item.durationSecs) * 1000;
}
