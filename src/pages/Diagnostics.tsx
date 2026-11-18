import { useEffect, useState } from "react";
import { api } from "../lib/api";

export default function Diagnostics() {
  const [diag, setDiag] = useState<any>(null);
  const [target, setTarget] = useState("");
  const [devices, setDevices] = useState<any[]>([]);
  const [speed, setSpeed] = useState<any>(null);

  useEffect(() => {
    (async () => {
      setDiag(await api.diagnostics());
      setDevices(await api.devices());
    })();
  }, []);

  const runSpeed = async () => {
    const d = devices.find(x => x.id === target);
    if (!d) return;
    setSpeed(await api.speedTest(d.ip, d.port));
  };

  return (
    <div>
      <h1>Network Diagnostics</h1>
      <div className="card">
        <h2>Results</h2>
        {diag && Object.entries(diag).map(([k, v]) => (
          <div key={k}><strong>{k}:</strong> {typeof v === "object" ? JSON.stringify(v) : String(v)}</div>
        ))}
      </div>
      <div className="card" style={{ marginTop: 16 }}>
        <h2>Speed Test</h2>
        <select className="input" value={target} onChange={e => setTarget(e.target.value)}>
          <option value="">Select device</option>
          {devices.map(d => <option key={d.id} value={d.id}>{d.name}</option>)}
        </select>
        <button className="btn" style={{ marginLeft: 8 }} onClick={runSpeed}>Run</button>
        {speed && <div className="muted" style={{ marginTop: 8 }}>{JSON.stringify(speed)}</div>}
      </div>
    </div>
  );
}
