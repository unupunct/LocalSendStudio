import { useEffect, useState } from "react";
import { api } from "../lib/api";

export default function Settings() {
  const [name, setName] = useState("");
  const [mode, setMode] = useState("open");
  const [port, setPort] = useState("47821");
  const [interfaces, setInterfaces] = useState<any[]>([]);

  useEffect(() => {
    (async () => {
      const id = await api.identity();
      setName(id.name);
      setMode((await api.getSetting("network_mode")) || "open");
      setPort((await api.getSetting("port")) || "47821");
      setInterfaces(await api.interfaces());
    })();
  }, []);

  const save = async () => {
    await api.setSetting("network_mode", mode);
    await api.setSetting("port", port);
    await api.setDeviceName(name);
    alert("Saved");
  };

  return (
    <div>
      <h1>Settings</h1>
      <div className="card">
        <h2>Device</h2>
        <div className="row"><label>Device Name</label><input className="input" value={name} onChange={e => setName(e.target.value)} /></div>
      </div>
      <div className="card" style={{ marginTop: 16 }}>
        <h2>Network Mode</h2>
        <select className="input" value={mode} onChange={e => setMode(e.target.value)}>
          <option value="open">Open Network Mode (default)</option>
          <option value="restricted">Restricted Mode</option>
        </select>
        <div className="muted" style={{ marginTop: 8 }}>Open Network Mode trusts devices able to reach the app on the LAN. Only configured shared folders are exposed.</div>
      </div>
      <div className="card" style={{ marginTop: 16 }}>
        <h2>Port</h2>
        <input className="input" value={port} onChange={e => setPort(e.target.value)} />
      </div>
      <div className="card" style={{ marginTop: 16 }}>
        <h2>Interfaces</h2>
        {interfaces.map((i, idx) => <div key={idx}>{i.name} — {i.ip} ({i.kind})</div>)}
      </div>
      <button className="btn" style={{ marginTop: 16 }} onClick={save}>Save</button>
    </div>
  );
}
