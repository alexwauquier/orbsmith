import { describe, expect, it } from "vitest";
import type { QueueItem, QueueState, Simulation } from "$lib/api";
import { estimateQueue } from "$lib/queue-estimate";

const NOW = 1_000_000;
const MIN = 60_000;

function item(appId: string, minutes: number, leftMinutes: number | null = null): QueueItem {
  return {
    appId,
    appName: appId,
    durationSecs: minutes * 60,
    leftSecs: leftMinutes === null ? null : leftMinutes * 60
  };
}

function state(
  active: QueueItem[],
  queued: QueueItem[],
  extra: Partial<QueueState> = {}
): QueueState {
  return { running: active.length > 0, paused: false, active, queued, nextStartAt: null, ...extra };
}

function sim(appId: string, stopAt: number | null): Simulation {
  return { appId, appName: appId, executable: "game.exe", pid: 1, startedAt: NOW, stopAt };
}

const oneByOne = { parallelQueue: false, queueGapSecs: 0, varyQueueTimes: false };

describe("estimateQueue", () => {
  it("has nothing to estimate for an empty queue", () => {
    expect(estimateQueue(state([], []), oneByOne, [], NOW)).toBeNull();
  });

  it("adds the waiting games after the running one's timer", () => {
    const queue = state([item("a", 15)], [item("b", 30), item("c", 10, 4)]);
    const estimate = estimateQueue(queue, oneByOne, [sim("a", NOW + 5 * MIN)], NOW);
    expect(estimate).toEqual({ endsAt: NOW + (5 + 30 + 4) * MIN, approximate: false });
  });

  it("counts a wait after every game but the last", () => {
    const settings = { ...oneByOne, queueGapSecs: 120 };
    const queue = state([item("a", 15)], [item("b", 30), item("c", 10)]);
    expect(estimateQueue(queue, settings, [sim("a", NOW + 5 * MIN)], NOW)?.endsAt).toBe(
      NOW + (5 + 2 + 30 + 2 + 10) * MIN
    );
  });

  it("starts from the end of a wait in progress", () => {
    const settings = { ...oneByOne, queueGapSecs: 120 };
    const queue = state([], [item("b", 30)], { running: true, nextStartAt: NOW + MIN });
    expect(estimateQueue(queue, settings, [], NOW)?.endsAt).toBe(NOW + 31 * MIN);
  });

  it("estimates a stopped queue as if started now, without a first wait", () => {
    const settings = { ...oneByOne, queueGapSecs: 120 };
    const queue = state([], [item("a", 15), item("b", 30)]);
    expect(estimateQueue(queue, settings, [], NOW)?.endsAt).toBe(NOW + (15 + 2 + 30) * MIN);
  });

  it("takes the longest game when they all run at once", () => {
    const settings = { parallelQueue: true, queueGapSecs: 600, varyQueueTimes: true };
    const queue = state([item("a", 15)], [item("b", 30), item("c", 10)]);
    expect(estimateQueue(queue, settings, [sim("a", NOW + 40 * MIN)], NOW)).toEqual({
      endsAt: NOW + 40 * MIN,
      approximate: true
    });
  });

  it("gives up when a running game has no timer", () => {
    const queue = state([item("a", 15)], [item("b", 30)]);
    expect(estimateQueue(queue, oneByOne, [sim("a", null)], NOW)).toBeNull();
  });

  it("uses the whole duration of a game not listed as running yet", () => {
    const queue = state([item("a", 15)], []);
    expect(estimateQueue(queue, oneByOne, [], NOW)?.endsAt).toBe(NOW + 15 * MIN);
  });

  it("never ends before now", () => {
    const queue = state([item("a", 15)], []);
    expect(estimateQueue(queue, oneByOne, [sim("a", NOW - MIN)], NOW)?.endsAt).toBe(NOW);
  });
});
