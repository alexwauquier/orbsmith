import { describe, expect, it } from "vitest";
import { formatAgo, formatClock, formatCountdown, formatDuration } from "$lib/time";

describe("formatCountdown", () => {
  it("formats minutes and seconds", () => {
    expect(formatCountdown(65_000)).toBe("1:05");
    expect(formatCountdown(0)).toBe("0:00");
  });

  it("adds hours when needed", () => {
    expect(formatCountdown(3_725_000)).toBe("1:02:05");
  });

  it("rounds up, so a running timer never shows 0:00", () => {
    expect(formatCountdown(1)).toBe("0:01");
    expect(formatCountdown(59_001)).toBe("1:00");
  });

  it("clamps negative durations to zero", () => {
    expect(formatCountdown(-5_000)).toBe("0:00");
  });
});

describe("formatAgo", () => {
  const relative = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });
  const now = Date.UTC(2026, 10, 9, 12);

  it("says just now under a minute", () => {
    expect(formatAgo(now - 59_000, now)).toBe("just now");
  });

  it("picks minutes, hours, then days", () => {
    expect(formatAgo(now - 5 * 60_000, now)).toBe(relative.format(-5, "minute"));
    expect(formatAgo(now - 3 * 3_600_000, now)).toBe(relative.format(-3, "hour"));
    expect(formatAgo(now - 2 * 86_400_000, now)).toBe(relative.format(-2, "day"));
  });
});

describe("formatDuration", () => {
  it("shows whole minutes", () => {
    expect(formatDuration(15 * 60)).toBe("15 min");
    expect(formatDuration(60)).toBe("1 min");
  });

  it("adds hours from an hour on", () => {
    expect(formatDuration(90 * 60)).toBe("1 h 30 min");
    expect(formatDuration(2 * 3600)).toBe("2 h");
  });
});

describe("formatClock", () => {
  const now = new Date(2026, 10, 10, 9, 0).getTime();

  it("shows the time of day for today", () => {
    expect(formatClock(new Date(2026, 10, 10, 14, 32).getTime(), now, "en-GB")).toBe("14:32");
  });

  it("adds the weekday for another day", () => {
    expect(formatClock(new Date(2026, 10, 11, 1, 5).getTime(), now, "en-GB")).toBe("Wed 01:05");
  });
});
