/** `1:05:09` or `5:09`, rounded up so a running timer never shows `0:00`. */
export function formatCountdown(ms: number): string {
  const total = Math.max(0, Math.ceil(ms / 1000));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, "0");
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, "0")}:${seconds}`
    : `${minutes}:${seconds}`;
}

const relative = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });

/** `just now`, `5 minutes ago`, `yesterday`… */
export function formatAgo(timestamp: number, now: number): string {
  const seconds = Math.round((timestamp - now) / 1000);
  if (Math.abs(seconds) < 60) return "just now";
  const minutes = Math.round(seconds / 60);
  if (Math.abs(minutes) < 60) return relative.format(minutes, "minute");
  const hours = Math.round(minutes / 60);
  if (Math.abs(hours) < 24) return relative.format(hours, "hour");
  return relative.format(Math.round(hours / 24), "day");
}

/** A duration in whole minutes: `15 min`, `1 h 30 min`, `2 h`. */
export function formatDuration(durationSecs: number): string {
  const minutes = Math.floor(durationSecs / 60);
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) return `${minutes} min`;
  return rest === 0 ? `${hours} h` : `${hours} h ${rest} min`;
}

/** A time of day, `14:32` or `2:32 PM`, with the weekday when not today. */
export function formatClock(timestamp: number, now: number, locale?: string): string {
  const date = new Date(timestamp);
  const today = date.toDateString() === new Date(now).toDateString();
  return date.toLocaleString(locale, {
    weekday: today ? undefined : "short",
    hour: "numeric",
    minute: "2-digit"
  });
}
