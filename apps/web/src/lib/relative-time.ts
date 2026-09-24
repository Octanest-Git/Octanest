/** Short relative label for an ISO timestamp: "just now", "5m ago", "3d ago". */
export function formatRelative(iso: string, nowMs = Date.now()): string {
  const then = Date.parse(iso);
  if (Number.isNaN(then)) return "recently";
  const deltaSec = Math.max(0, Math.floor((nowMs - then) / 1000));
  if (deltaSec < 60) return "just now";
  const mins = Math.floor(deltaSec / 60);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 48) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months}mo ago`;
  const years = Math.floor(days / 365);
  return `${years}y ago`;
}

/** Relative time labels for repo list rows (UI-SPEC Label muted). */
export function formatRelativeUpdated(iso: string, nowMs = Date.now()): string {
  const rel = formatRelative(iso, nowMs);
  return rel === "recently" ? "Updated recently" : `Updated ${rel}`;
}
