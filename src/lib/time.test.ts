import { describe, expect, it } from "vitest";
import { formatAgo, formatCountdown } from "$lib/time";

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
