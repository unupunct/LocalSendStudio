import { useEffect, useMemo, useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Laptop, Send as SendIcon, UploadCloud, X } from "lucide-react";
import { api, Device, Group } from "../lib/api";
import { useApp } from "../lib/store";
import { fileName } from "../lib/format";
import { pickFiles } from "../lib/pick";
import { Dot, Empty, FileIcon, OsIcon, Page } from "../components/ui";

export default function SendPage() {
  const { devices, toast, fail } = useApp();
  const [params] = useSearchParams();
  const nav = useNavigate();
  const [files, setFiles] = useState<string[]>([]);
  const [picked, setPicked] = useState<Set<string>>(new Set());
  const [over, setOver] = useState(false);
  const [groups, setGroups] = useState<Group[]>([]);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    const d = params.get("dev");
    if (d) setPicked(new Set([d]));
    api.groups().then(setGroups).catch(() => {});
  }, [params]);

  useEffect(() => {
    let un: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((e) => {
        const p = e.payload;
        if (p.type === "over" || p.type === "enter") setOver(true);
        else if (p.type === "leave") setOver(false);
        else if (p.type === "drop") {
          setOver(false);
          setFiles((f) => [...new Set([...f, ...p.paths])]);
        }
      })
      .then((u) => (un = u))
      .catch(() => {});
    return () => un?.();
  }, []);

  const online = useMemo(() => devices.filter((d) => d.status === "online"), [devices]);
  const toggle = (id: string) => setPicked((s) => { const n = new Set(s); n.has(id) ? n.delete(id) : n.add(id); return n; });

  const add = async () => {
    try {
      const r = await pickFiles(true);
      if (r.length) setFiles((f) => [...new Set([...f, ...r])]);
    } catch (e) { fail(e); }
  };

  const send = async () => {
    const targets: Device[] = online.filter((d) => picked.has(d.id));
    if (!files.length || !targets.length) return;
    setBusy(true);
    let started = 0;
    for (const d of targets) {
      // "Inbox" is created automatically on every device for exactly this.
      const share = d.shared_folders.includes("Inbox") ? "Inbox" : d.shared_folders[0];
      if (!share) { toast(`${d.name} isn't sharing any folder, so it can't receive files.`, "err"); continue; }
      for (const f of files) {
        try { await api.upload(d, share, "", f, ); started++; } catch (e) { fail(e); }
      }
    }
    setBusy(false);
    if (started) { setFiles([]); nav("/transfers"); }
  };

  const applyGroup = (g: Group) => {
    const ids = g.device_ids.split(",").map((s) => s.trim());
    setPicked(new Set(online.filter((d) => ids.includes(d.id)).map((d) => d.id)));
  };

  return (
    <Page title="Send" sub="Pick files, pick devices, done. The other person is asked before anything arrives.">
      <div className="stack">
        <button className={`drop ${over ? "over" : ""}`} onClick={add}>
          <UploadCloud size={30} />
          <b>{files.length ? "Add more files" : "Choose files to send"}</b>
          <span>or drop them anywhere in this window</span>
        </button>

        {files.length > 0 && (
          <div className="card">
            <h2>{files.length} file{files.length > 1 ? "s" : ""} ready</h2>
            {files.map((f) => (
              <div key={f} className="file">
                <FileIcon name={f} />
                <div className="grow"><div className="ellipsis">{fileName(f)}</div><div className="muted small ellipsis">{f}</div></div>
                <button className="btn ghost icon" aria-label="Remove" onClick={() => setFiles((x) => x.filter((y) => y !== f))}><X size={16} /></button>
              </div>
            ))}
          </div>
        )}

        <section>
          <div className="row" style={{ justifyContent: "space-between", marginBottom: 10 }}>
            <h2 style={{ margin: 0 }}>Send to</h2>
            <div className="row">
              {groups.map((g) => <button key={g.id} className="btn sm" onClick={() => applyGroup(g)}>{g.name}</button>)}
              {online.length > 1 && <button className="btn sm ghost" onClick={() => setPicked(new Set(online.map((d) => d.id)))}>Select all</button>}
            </div>
          </div>
          {online.length === 0 ? (
            <div className="card"><Empty icon={<Laptop size={24} />} title="No devices online">Open LocalSendStudio on the receiving device. Both need to be on the same network.</Empty></div>
          ) : (
            <div className="grid cols">
              {online.map((d) => (
                <button key={d.id} className={`card dev ${picked.has(d.id) ? "sel" : ""}`} style={{ textAlign: "left" }} onClick={() => toggle(d.id)} aria-pressed={picked.has(d.id)}>
                  <div className="dev-top">
                    <div className="dev-ico"><OsIcon os={d.os} /></div>
                    <div className="grow"><div className="dev-name ellipsis">{d.name}</div><div className="muted small">{d.os} · {d.ip}</div></div>
                    <Dot on />
                  </div>
                </button>
              ))}
            </div>
          )}
        </section>

        <div className="row" style={{ justifyContent: "flex-end" }}>
          <button className="btn primary" disabled={busy || !files.length || !picked.size} onClick={send}>
            <SendIcon size={17} /> Send{files.length && picked.size ? ` ${files.length} file${files.length > 1 ? "s" : ""} to ${picked.size} device${picked.size > 1 ? "s" : ""}` : ""}
          </button>
        </div>
      </div>
    </Page>
  );
}
