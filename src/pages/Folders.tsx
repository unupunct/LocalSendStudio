import { useCallback, useEffect, useState } from "react";
import { AlertTriangle, FolderPlus, Share2, Trash2 } from "lucide-react";
import { api, SharedFolder } from "../lib/api";
import { useApp } from "../lib/store";
import { pickFolder } from "../lib/pick";
import { Empty, FileIcon, Page, Switch } from "../components/ui";

type Level = "none" | "download" | "dropbox" | "readwrite" | "full";

const levelOf = (f: SharedFolder): Level => (f.manage ? "full" : f.read && f.upload ? "readwrite" : f.read ? "download" : f.upload ? "dropbox" : "none");
const perms = (l: Level): [boolean, boolean, boolean] =>
  l === "full" ? [true, true, true] : l === "readwrite" ? [true, true, false] : l === "download" ? [true, false, false] : l === "dropbox" ? [false, true, false] : [false, false, false];

const LEVELS: { id: Level; label: string; hint: string; risk?: boolean }[] = [
  { id: "download", label: "View & download", hint: "Others can see and copy files. Nothing can be changed." },
  { id: "dropbox", label: "Drop box", hint: "Others can send files here but not see what's inside." },
  { id: "readwrite", label: "View & upload", hint: "Others can see, download and add files. Existing files are never overwritten." },
  { id: "full", label: "Full control", hint: "Others can also rename and delete files. Only use this with devices you own.", risk: true },
];

export default function Folders() {
  const { toast, fail, confirm } = useApp();
  const [list, setList] = useState<SharedFolder[]>([]);
  const refresh = useCallback(() => api.sharedFolders().then(setList).catch(() => {}), []);
  useEffect(() => { refresh(); }, [refresh]);

  const add = async () => {
    try {
      const p = await pickFolder();
      if (!p) return;
      const f = await api.addSharedFolder(p);
      toast(`Sharing “${f.name}” (view & download)`, "ok");
      refresh();
    } catch (e) { fail(e); }
  };

  return (
    <Page title="Shared folders" sub="Only folders you add here are visible to other devices." actions={<button className="btn primary" onClick={add}><FolderPlus size={17} /> Share a folder</button>}>
      {list.length === 0 ? (
        <div className="card"><Empty icon={<Share2 size={24} />} title="No folders shared">Add a folder so other devices can browse it. New folders start as view-only.</Empty></div>
      ) : (
        <div className="stack">
          {list.map((f) => {
            const lvl = levelOf(f);
            const info = LEVELS.find((l) => l.id === lvl);
            return (
              <div key={f.id} className="card" style={{ opacity: f.enabled ? 1 : 0.65 }}>
                <div className="row" style={{ justifyContent: "space-between", alignItems: "flex-start" }}>
                  <div className="row grow" style={{ flexWrap: "nowrap" }}>
                    <div className="dev-ico"><FileIcon name={f.name} dir size={22} /></div>
                    <div className="grow">
                      <div className="dev-name ellipsis">{f.name}</div>
                      <div className="muted small mono ellipsis" title={f.path}>{f.path}</div>
                    </div>
                  </div>
                  <div className="row">
                    <Switch checked={f.enabled} label={`Share ${f.name}`} onChange={(v) => api.toggleSharedFolder(f.id, v).then(refresh)} />
                    <button className="btn ghost icon" aria-label={`Stop sharing ${f.name}`} onClick={async () => { if (await confirm(`Stop sharing “${f.name}”?`, { body: "Your files stay where they are.", ok: "Stop sharing" })) { await api.removeSharedFolder(f.id); refresh(); } }}><Trash2 size={16} /></button>
                  </div>
                </div>
                <div className="row" style={{ marginTop: 14 }}>
                  <div className="perm" role="radiogroup" aria-label="Access level">
                    {LEVELS.map((l) => (
                      <button key={l.id} role="radio" aria-checked={lvl === l.id} className={`${lvl === l.id ? "on" : ""} ${l.risk ? "risk" : ""}`}
                        onClick={() => api.setFolderPerms(f.id, ...perms(l.id)).then(refresh)}>{l.label}</button>
                    ))}
                  </div>
                </div>
                <p className="muted small" style={{ margin: "10px 0 0", display: "flex", gap: 6, alignItems: "center" }}>
                  {info?.risk && <AlertTriangle size={14} color="var(--warn)" />} {info?.hint ?? "Others can't use this folder."}
                </p>
              </div>
            );
          })}
        </div>
      )}
    </Page>
  );
}
