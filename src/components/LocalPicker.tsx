import { useEffect, useState } from "react";
import { ArrowUp, Check } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { bytes, errText } from "../lib/format";
import { FileIcon } from "./ui";
import { registerPicker, PickRequest } from "../lib/pick";

type Listing = { path: string; parent: string | null; entries: { name: string; is_dir: boolean; size: number }[] };

/** In-app file picker. Android's system picker returns content:// URIs that native code cannot read,
 *  so on Android we browse real paths instead (the app is granted "all files access"). */
export function LocalPicker() {
  const [req, setReq] = useState<(PickRequest & { done: (r: string[] | null) => void }) | null>(null);
  const [list, setList] = useState<Listing | null>(null);
  const [sel, setSel] = useState<string[]>([]);
  const [err, setErr] = useState("");

  useEffect(() => registerPicker((r) => new Promise((resolve) => { setSel([]); setReq({ ...r, done: resolve }); })), []);

  const go = (path: string) =>
    invoke<Listing>("list_local_dir", { path })
      .then((l) => { setList(l); setErr(""); })
      .catch((e) => setErr(errText(e)));
  useEffect(() => { if (req) go(""); }, [req]);

  if (!req) return null;
  const join = (n: string) => `${list!.path.replace(/\/$/, "")}/${n}`;
  const finish = (r: string[] | null) => { req.done(r); setReq(null); setList(null); };
  const toggle = (p: string) => setSel((s) => (s.includes(p) ? s.filter((x) => x !== p) : req.multiple ? [...s, p] : [p]));

  return (
    <div className="scrim" style={{ alignItems: "end" }} onMouseDown={(e) => e.target === e.currentTarget && finish(null)}>
      <div className="dialog" style={{ width: "min(560px,100%)", maxHeight: "86vh", display: "flex", flexDirection: "column", padding: 16 }}>
        <div className="row" style={{ flexWrap: "nowrap", marginBottom: 8 }}>
          <button className="btn icon" disabled={!list?.parent} onClick={() => list?.parent && go(list.parent)} aria-label="Up one folder"><ArrowUp size={16} /></button>
          <div className="grow mono ellipsis" title={list?.path}>{list?.path ?? "…"}</div>
        </div>
        <div style={{ overflow: "auto", flex: 1, minHeight: 200 }}>
          {err && <p className="muted">{err}</p>}
          {list?.entries.map((e) => {
            const p = join(e.name);
            const on = sel.includes(p);
            const pickable = req.mode === "files" ? !e.is_dir : false;
            return (
              <div key={e.name} className="file" style={on ? { background: "var(--panel-2)" } : undefined}>
                <button className="name" onClick={() => (e.is_dir ? go(p) : pickable && toggle(p))}>
                  <FileIcon name={e.name} dir={e.is_dir} /> <span className="ellipsis">{e.name}</span>
                </button>
                {!e.is_dir && <span className="muted small">{bytes(e.size)}</span>}
                {on && <Check size={16} color="var(--brand)" />}
              </div>
            );
          })}
          {list && list.entries.length === 0 && <p className="muted">This folder is empty.</p>}
        </div>
        <div className="dialog-actions">
          <button className="btn ghost" onClick={() => finish(null)}>Cancel</button>
          {req.mode === "folder" ? (
            <button className="btn primary" disabled={!list} onClick={() => finish([list!.path])}>Use this folder</button>
          ) : (
            <button className="btn primary" disabled={!sel.length} onClick={() => finish(sel)}>Select{sel.length ? ` (${sel.length})` : ""}</button>
          )}
        </div>
      </div>
    </div>
  );
}
