import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { listen } from "@tauri-apps/api/event";

export default function Dashboard() {
  const [identity, setIdentity] = useState<any>(null);
  const [devices, setDevices] = useState<any[]>([]);
  const [transfers, setTransfers] = useState<any[]>([]);
  const [shared, setShared] = useState<any[]>([]);

  const refresh = async () => {
    setIdentity(await api.identity());
    setDevices(await api.devices());
    setTransfers(await api.transferHistory());
    setShared(await api.sharedFolders());
  };

  useEffect(() => {
    refresh();
    const un = listen("devices-updated", refresh);
    const t = setInterval(refresh, 4000);
    return () => { un.then(f => f()); clearInterval(t); };
  }, []);

  const online = devices.filter(d => d.status === "online").length;

  return (
    <div>
      <h1>Dashboard</h1>
      <div className="grid grid-3">
        <div className="card">
          <h2>Local Device</h2>
          <div>{identity?.name}</div>
          <div className="muted">{identity?.os} · {identity?.id?.slice(0,8)}</div>
        </div>
        <div className="card">
          <h2>Network Status</h2>
          <div>{online} online devices</div>
          <div className="muted">{devices.length} discovered</div>
        </div>
        <div className="card">
          <h2>Discovery</h2>
          <div>Active</div>
          <div className="muted">mDNS/UDP multicast</div>
        </div>
      </div>
      <div className="card" style={{ marginTop: 16 }}>
        <h2>Shared Folders</h2>
        {shared.map(s => <div key={s.id}>{s.name} — {s.path} {s.enabled ? "(on)" : "(off)"}</div>)}
        {shared.length === 0 && <div className="muted">No shared folders yet.</div>}
      </div>
      <div className="card" style={{ marginTop: 16 }}>
        <h2>Recent Transfers</h2>
        <table>
          <thead><tr><th>File</th><th>Source</th><th>Dest</th><th>Status</th></tr></thead>
          <tbody>
            {transfers.slice(0, 8).map(t => (
              <tr key={t.id}><td>{t.filename}</td><td>{t.source}</td><td>{t.destination}</td><td>{t.status}</td></tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
