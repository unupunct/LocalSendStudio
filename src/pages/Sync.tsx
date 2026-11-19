import { useState } from "react";
import { ArrowDown, ArrowUp, FolderSync, Play } from "lucide-react";
import { useApp } from "../lib/store";
import { api } from "../lib/api";
import { pickFolder } from "../lib/pick";
import { Page } from "../components/ui";

export default function Sync() {
  const { devices, toast, fail } = useApp();
  const online = devices.filter((d) => d.status === "online" && d.shared_folders.length);
  const [devId, setDevId] = useState("");
  const [share, setShare] = useState("");
  const [dir, setDir] = useState<"up" | "down">("down");
  const [local, setLocal] = useState("");
  const [remote, setRemote] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState("");
  const dev = online.find((d) => d.id === devId);

  const run = async () => {
    if (!dev || !share || !local) return;
    setBusy(true);
    setResult("");
    try {
      const r = dir === "up" ? await api.syncUp(local, dev, share, remote) : await api.syncDown(dev, share, remote, local);
      setResult(`${r.copied} file${r.copied === 1 ? "" : "s"} copied, ${r.skipped} already up to date.`);
      toast("Sync finished", "ok");
    } catch (e) { fail(e); } finally { setBusy(false); }
  };

  return (
    <Page title="Sync" sub="Copy a folder to or from another device. Files that already match are skipped, and nothing is ever deleted.">
      <div className="card stack">
        <div className="grid two">
          <label><div className="muted small">Device</div>
            <select className="input" value={devId} onChange={(e) => { setDevId(e.target.value); setShare(online.find((d) => d.id === e.target.value)?.shared_folders[0] ?? ""); }}>
              <option value="">Choose a device…</option>
              {online.map((d) => <option key={d.id} value={d.id}>{d.name}</option>)}
            </select></label>
          <label><div className="muted small">Shared folder</div>
            <select className="input" value={share} onChange={(e) => setShare(e.target.value)} disabled={!dev}>
              {dev?.shared_folders.map((s) => <option key={s}>{s}</option>)}
            </select></label>
        </div>
        <div className="perm" style={{ alignSelf: "flex-start" }}>
          <button className={dir === "down" ? "on" : ""} onClick={() => setDir("down")}><ArrowDown size={14} style={{ verticalAlign: -2 }} /> Download to this device</button>
          <button className={dir === "up" ? "on" : ""} onClick={() => setDir("up")}><ArrowUp size={14} style={{ verticalAlign: -2 }} /> Upload from this device</button>
        </div>
        <div className="row">
          <input className="input grow" placeholder="Folder on this device" value={local} onChange={(e) => setLocal(e.target.value)} />
          <button className="btn" onClick={async () => { const p = await pickFolder(); if (p) setLocal(p); }}>Choose…</button>
        </div>
        <input className="input" placeholder="Subfolder on the other device (optional)" value={remote} onChange={(e) => setRemote(e.target.value)} />
        <div className="row">
          <button className="btn primary" disabled={busy || !dev || !share || !local} onClick={run}><Play size={16} /> {busy ? "Syncing…" : "Start sync"}</button>
          {result && <span className="pill ok"><FolderSync size={13} /> {result}</span>}
        </div>
      </div>
    </Page>
  );
}
