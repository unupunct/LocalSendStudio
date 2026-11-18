import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { listen } from "@tauri-apps/api/event";

export default function NetworkMap() {
  const [devices, setDevices] = useState<any[]>([]);
  const [identity, setIdentity] = useState<any>(null);
  const [mode, setMode] = useState<"map" | "list">("map");
  const [selected, setSelected] = useState<any>(null);

  const refresh = async () => {
    setDevices(await api.devices());
    setIdentity(await api.identity());
  };
  useEffect(() => {
    refresh();
    const un = listen("devices-updated", refresh);
    const t = setInterval(refresh, 4000);
    return () => { un.then(f => f()); clearInterval(t); };
  }, []);

  const cx = 50, cy = 50;
  const radius = 38;

  return (
    <div>
      <h1>Network Map</h1>
      <div className="row" style={{ marginBottom: 12 }}>
        <button className="btn secondary" onClick={() => setMode("map")}>Network Map</button>
        <button className="btn secondary" onClick={() => setMode("list")}>Device List</button>
      </div>
      {mode === "list" ? (
        <div className="grid grid-3">
          {devices.map(d => <DeviceCard key={d.id} d={d} />)}
        </div>
      ) : (
        <div className="map-canvas">
          <div className="node local" style={{ left: cx + "%", top: cy + "%" }}>
            {identity?.name || "You"}
          </div>
          {devices.map((d, i) => {
            const angle = (i / Math.max(devices.length, 1)) * 2 * Math.PI;
            const x = cx + radius * Math.cos(angle);
            const y = cy + radius * Math.sin(angle);
            return (
              <div key={d.id}>
                <div className="line" style={{
                  left: cx + "%", top: cy + "%",
                  width: radius + "%",
                  transform: `rotate(${angle}rad)`,
                }} />
                <div className="node" style={{ left: x + "%", top: y + "%" }} onClick={() => setSelected(d)}>
                  <div>{d.name}</div>
                  <div className="muted">{d.os}</div>
                </div>
              </div>
            );
          })}
        </div>
      )}
      {selected && (
        <div className="card" style={{ marginTop: 16 }}>
          <h2>{selected.name}</h2>
          <div className="muted">{selected.os} · {selected.ip}:{selected.port}</div>
          <div>Shared: {selected.shared_folders.join(", ") || "none"}</div>
          <button className="btn secondary" onClick={() => setSelected(null)}>Close</button>
        </div>
      )}
    </div>
  );
}

function DeviceCard({ d }: any) {
  return (
    <div className="device-card">
      <div>{d.name}</div>
      <div className="muted">{d.os} · {d.ip}</div>
      <span className={`status ${d.status}`}>{d.status}</span>
    </div>
  );
}
