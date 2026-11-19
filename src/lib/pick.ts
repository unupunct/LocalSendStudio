import { open } from "@tauri-apps/plugin-dialog";

export type PickRequest = { mode: "files" | "folder"; multiple: boolean };
type Opener = (r: PickRequest) => Promise<string[] | null>;

let opener: Opener | null = null;
/** Called by <LocalPicker/>; returns an unregister function. */
export function registerPicker(o: Opener) {
  opener = o;
  return () => { if (opener === o) opener = null; };
}

const isAndroid = () => /Android/i.test(navigator.userAgent);

/** Pick files. Returns absolute paths the Rust side can read. */
export async function pickFiles(multiple = true): Promise<string[]> {
  if (isAndroid() && opener) return (await opener({ mode: "files", multiple })) ?? [];
  const r = await open({ multiple, directory: false });
  if (!r) return [];
  return Array.isArray(r) ? r : [r];
}

export async function pickFolder(): Promise<string | null> {
  if (isAndroid() && opener) return (await opener({ mode: "folder", multiple: false }))?.[0] ?? null;
  const r = await open({ directory: true, multiple: false });
  return typeof r === "string" ? r : null;
}
