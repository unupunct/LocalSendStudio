export function bytes(n: number): string {
  if (!isFinite(n) || n <= 0) return "0 B";
  const u = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(Math.floor(Math.log(n) / Math.log(1024)), u.length - 1);
  const v = n / Math.pow(1024, i);
  return `${v >= 100 || i === 0 ? v.toFixed(0) : v.toFixed(1)} ${u[i]}`;
}

export const speed = (bps: number) => `${bytes(bps)}/s`;

export function eta(sec: number): string {
  if (sec < 0 || !isFinite(sec)) return "";
  if (sec < 60) return `${Math.max(1, Math.round(sec))}s left`;
  if (sec < 3600) return `${Math.round(sec / 60)} min left`;
  return `${(sec / 3600).toFixed(1)} h left`;
}

export function ago(ms: number): string {
  const s = Math.max(0, (Date.now() - ms) / 1000);
  if (s < 45) return "just now";
  if (s < 3600) return `${Math.round(s / 60)} min ago`;
  if (s < 86400) return `${Math.round(s / 3600)} h ago`;
  return new Date(ms).toLocaleDateString();
}

export function when(ms: number): string {
  return new Date(ms).toLocaleString(undefined, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
}

export const fileName = (p: string) => p.split(/[\\/]/).filter(Boolean).pop() ?? p;

export const joinPath = (dir: string, name: string) => (dir ? `${dir}/${name}` : name);

export function errText(e: unknown): string {
  const m = typeof e === "string" ? e : e instanceof Error ? e.message : JSON.stringify(e);
  return m.replace(/^Error: /, "");
}
