import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { listen } from "@tauri-apps/api/event";

export default function Transfers() {
  const [progress, setProgress] = useState<Record<string, any>>({});
  const [history, setHistory] = useState<any[]>([]);

  const refreshHistory = async () => setHistory(await api.transferHistory());

  useEffect(() => {
    refreshHistory();
    const un1 = listen("transfer-progress", (e: any) => {
      const p = e.payload;
      setProgress(prev => ({ ...prev, [p.id]: { ...prev[p.id], ...p } }));
    });
    const un2 = listen("transfer-done", (e: any) => {
      const p = e.payload;
      setProgress(prev => ({ ...prev, [p.id]: { ...prev[p.id], status: p.status } }));
      refreshHistory();
    });
    const t = setInterval(refreshHistory, 5000);
    return () => { un1.then(f => f()); un2.then(f => f()); clearInterval(t); };
  }, []);

  return (
    <div>
      <h1>Transfers</h1>
      <div className="card">
        <h2>Active</h2>
        {Object.entries(progress).map(([id, p]) => {
          const pct = p.total ? Math.round((p.transferred / p.total) * 100) : 0;
          return (
            <div key={id} style={{ marginBottom: 12 }}>
              <div className="row">
                <span>{id.slice(0, 8)}</span>
                <span className="muted">{p.status}</span>
                <button className="btn danger" onClick={() => api.cancelTransfer(id)}>Cancel</button>
              </div>
              <div className="progress"><div style={{ width: pct + "%" }} /></div>
              <div className="muted">{pct}% · {Math.round((p.speed || 0) / 1024)} KB/s · {p.transferred}/{p.total}</div>
            </div>
          );
        })}
        {Object.keys(progress).length === 0 && <div className="muted">No active transfers.</div>}
      </div>
      <div className="card" style={{ marginTop: 16 }}>
        <h2>History</h2>
        <table>
          <thead><tr><th>File</th><th>Source</th><th>Dest</th><th>Size</th><th>Status</th><th>Speed</th></tr></thead>
          <tbody>
            {history.map(t => (
              <tr key={t.id}><td>{t.filename}</td><td>{t.source}</td><td>{t.destination}</td><td>{(t.size/1024/1024).toFixed(1)} MB</td><td>{t.status}</td><td>{Math.round(t.speed/1024)} KB/s</td></tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
