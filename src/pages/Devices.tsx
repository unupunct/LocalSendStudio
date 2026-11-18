import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { api } from "../lib/api";
import { listen } from "@tauri-apps/api/event";

export default function Devices() {
  const [devices, setDevices] = useState<any[]>([]);
  const [filterOs, setFilterOs] = useState("");
  const [filterStatus, setFilterStatus] = useState("");
  const nav = useNavigate();

  const refresh = async () => setDevices(await api.devices());
  useEffect(() => {
    refresh();
    const un = listen("devices-updated", refresh);
    const t = setInterval(refresh, 4000);
    return () => { un.then(f => f()); clearInterval(t); };
  }, []);

  const filtered = devices.filter(d =>
    (!filterOs || d.os === filterOs) && (!filterStatus || d.status === filterStatus)
  );

  const browse = (d: any) => {
    localStorage.setItem("selectedDevice", JSON.stringify(d));
    nav("/explorer");
  };

  return (
    <div>
      <h1>Devices</h1>
      <div className="row" style={{ marginBottom: 12 }}>
        <select className="input" value={filterOs} onChange={e => setFilterOs(e.target.value)}>
          <option value="">All OS</option>
          {[...new Set(devices.map(d => d.os))].map(o => <option key={o} value={o}>{o}</option>)}
        </select>
        <select className="input" value={filterStatus} onChange={e => setFilterStatus(e.target.value)}>
          <option value="">All Status</option>
          <option value="online">Online</option>
          <option value="offline">Offline</option>
        </select>
      </div>
      <div className="grid grid-3">
        {filtered.map(d => (
          <div key={d.id} className="device-card">
            <div style={{ fontWeight: 600 }}>{d.name}</div>
            <div className="muted">{d.os} · {d.ip}</div>
            <span className={`status ${d.status}`}>{d.status}</span>
            <div className="muted" style={{ marginTop: 6 }}>Shared: {d.shared_folders.join(", ") || "none"}</div>
            <div className="row" style={{ marginTop: 8 }}>
              <button className="btn" onClick={() => browse(d)}>Browse</button>
              <button className="btn secondary" onClick={() => { localStorage.setItem("quickDevice", JSON.stringify(d)); nav("/quick"); }}>Send</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
