import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { open } from "@tauri-apps/plugin-dialog";

export default function QuickShare() {
  const [devices, setDevices] = useState<any[]>([]);
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [selectedDevices, setSelectedDevices] = useState<string[]>([]);
  const [status, setStatus] = useState("");
  const [groups, setGroups] = useState<any[]>([]);

  useEffect(() => {
    (async () => {
      setDevices(await api.devices());
      setGroups(await api.groups());
    })();
  }, []);

  const pickFiles = async () => {
    const res = await open({ multiple: true });
    if (res) setSelectedFiles(Array.isArray(res) ? res as string[] : [res as string]);
  };

  const toggleDevice = (id: string) => {
    setSelectedDevices(prev => prev.includes(id) ? prev.filter(x => x !== id) : [...prev, id]);
  };

  const send = async (targetDevices: any[]) => {
    if (selectedFiles.length === 0) { setStatus("No files selected"); return; }
    setStatus("Sending...");
    for (const d of targetDevices) {
      const share = d.shared_folders?.[0];
      if (!share) { setStatus(d.name + " has no shared folder"); continue; }
      for (const f of selectedFiles) {
        const name = f.split(/[\\/]/).pop()!;
        const remotePath = name;
        try {
          await api.upload(d.ip, d.port, share, remotePath, f);
          setStatus("Sent " + name + " to " + d.name);
        } catch (e) {
          setStatus("Failed " + name + " to " + d.name + ": " + String(e));
        }
      }
    }
    setStatus("Done");
  };

  const sendAll = () => send(devices.filter(d => d.status === "online"));
  const sendSelected = () => send(devices.filter(d => selectedDevices.includes(d.id)));
  const sendGroup = (g: any) => {
    const ids = g.device_ids.split(",").map((s: string) => s.trim());
    send(devices.filter(d => ids.includes(d.id)));
  };

  return (
    <div>
      <h1>Quick Share</h1>
      <div className="card">
        <h2>Files</h2>
        <button className="btn" onClick={pickFiles}>Select Files</button>
        <div className="muted" style={{ marginTop: 8 }}>{selectedFiles.length} selected</div>
        {selectedFiles.map(f => <div key={f} className="muted">{f}</div>)}
      </div>
      <div className="card" style={{ marginTop: 16 }}>
        <h2>Destination Devices</h2>
        <div className="grid grid-3">
          {devices.map(d => (
            <div key={d.id} className={`device-card ${selectedDevices.includes(d.id) ? "selected" : ""}`}
              onClick={() => toggleDevice(d.id)} style={{ cursor: "pointer", borderColor: selectedDevices.includes(d.id) ? "var(--accent)" : undefined }}>
              <div>{d.name}</div>
              <div className="muted">{d.os} · {d.status}</div>
            </div>
          ))}
        </div>
        <div className="row" style={{ marginTop: 12 }}>
          <button className="btn" onClick={sendSelected}>Send to Selected</button>
          <button className="btn secondary" onClick={sendAll}>Send to All Online</button>
          {groups.map(g => <button key={g.id} className="btn secondary" onClick={() => sendGroup(g)}>Send to {g.name}</button>)}
        </div>
        {status && <div className="muted" style={{ marginTop: 8 }}>{status}</div>}
      </div>
    </div>
  );
}
