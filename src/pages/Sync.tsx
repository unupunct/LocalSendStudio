import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { open } from "@tauri-apps/plugin-dialog";

export default function Sync() {
  const [devices, setDevices] = useState<any[]>([]);
  const [device, setDevice] = useState<any>(null);
  const [share, setShare] = useState("");
  const [direction, setDirection] = useState<"up" | "down">("up");
  const [localPath, setLocalPath] = useState("");
  const [remoteBase, setRemoteBase] = useState("");
  const [status, setStatus] = useState("");

  useEffect(() => { (async () => setDevices(await api.devices()))(); }, []);

  const pickLocal = async () => {
    const d = await open({ directory: true, multiple: false });
    if (d) setLocalPath(d as string);
  };

  const run = async () => {
    if (!device || !share || !localPath) { setStatus("Missing inputs"); return; }
    setStatus("Syncing...");
    try {
      if (direction === "up") {
        await api.syncLocalToRemote(localPath, device.ip, device.port, share, remoteBase);
      } else {
        await api.syncRemoteToLocal(device.ip, device.port, share, remoteBase, localPath);
      }
      setStatus("Sync complete");
    } catch (e) { setStatus("Failed: " + String(e)); }
  };

  return (
    <div>
      <h1>Synchronization</h1>
      <div className="card">
        <h2>Manual Sync</h2>
        <div className="row">
          <select className="input" onChange={e => setDevice(devices.find(d => d.id === e.target.value))}>
            <option value="">Select device</option>
            {devices.map(d => <option key={d.id} value={d.id}>{d.name}</option>)}
          </select>
          {device && (
            <select className="input" value={share} onChange={e => setShare(e.target.value)}>
              <option value="">Select share</option>
              {device.shared_folders.map((s: string) => <option key={s} value={s}>{s}</option>)}
            </select>
          )}
          <select className="input" value={direction} onChange={e => setDirection(e.target.value as any)}>
            <option value="up">Local → Remote</option>
            <option value="down">Remote → Local</option>
          </select>
        </div>
        <div className="row" style={{ marginTop: 8 }}>
          <input className="input" style={{ flex: 1 }} value={localPath} onChange={e => setLocalPath(e.target.value)} placeholder="Local folder path" />
          <button className="btn secondary" onClick={pickLocal}>Browse</button>
        </div>
        <div className="row" style={{ marginTop: 8 }}>
          <input className="input" style={{ flex: 1 }} value={remoteBase} onChange={e => setRemoteBase(e.target.value)} placeholder="Remote base path (optional)" />
        </div>
        <button className="btn" style={{ marginTop: 12 }} onClick={run}>Run Sync</button>
        {status && <div className="muted" style={{ marginTop: 8 }}>{status}</div>}
        <div className="muted" style={{ marginTop: 8 }}>Note: two-way automatic sync and conflict detection are planned; this performs a one-way copy.</div>
      </div>
    </div>
  );
}
