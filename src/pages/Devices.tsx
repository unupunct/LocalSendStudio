import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { FolderOpen, LayoutGrid, Laptop, Network as Net, Plus, Send, ShieldCheck, Trash2 } from "lucide-react";
import { api, Device } from "../lib/api";
import { useApp } from "../lib/store";
import { ago } from "../lib/format";
import { Dot, Empty, OsIcon, Page } from "../components/ui";

function MapView({ devices, me }: { devices: Device[]; me: string }) {
  const nav = useNavigate();
  const n = devices.length;
  return (
    <div className="map">
      <svg viewBox="0 0 100 100" preserveAspectRatio="none">
        {devices.map((d, i) => {
          const a = (i / Math.max(n, 1)) * Math.PI * 2 - Math.PI / 2;
          return <line key={d.id} x1="50" y1="50" x2={50 + 36 * Math.cos(a)} y2={50 + 38 * Math.sin(a)} stroke="var(--brand)" strokeOpacity={d.status === "online" ? 0.45 : 0.15} strokeWidth="0.4" strokeDasharray="1.2 1.2" vectorEffect="non-scaling-stroke" />;
        })}
      </svg>
      <div className="node me" style={{ left: "50%", top: "50%" }}>
        <div className="bubble"><Laptop size={22} /></div>
        <span className="ellipsis" style={{ maxWidth: 100 }}>{me} (you)</span>
      </div>
      {devices.map((d, i) => {
        const a = (i / Math.max(n, 1)) * Math.PI * 2 - Math.PI / 2;
        return (
          <button key={d.id} className={`node ${d.status === "online" ? "" : "off"}`} style={{ left: `${50 + 36 * Math.cos(a)}%`, top: `${50 + 38 * Math.sin(a)}%` }} onClick={() => nav(`/send?dev=${d.id}`)}>
            <div className="bubble"><OsIcon os={d.os} /></div>
            <span className="ellipsis" style={{ maxWidth: 100 }}>{d.name}</span>
          </button>
        );
      })}
    </div>
  );
}

export default function Devices() {
  const { devices, identity, refreshDevices, toast, fail, confirm } = useApp();
  const nav = useNavigate();
  const [view, setView] = useState<"grid" | "map">("grid");
  const [host, setHost] = useState("");
  const [busy, setBusy] = useState(false);

  const add = async () => {
    if (!host.trim()) return;
    setBusy(true);
    try {
      const d = await api.addDeviceByIp(host);
      toast(`Found ${d.name}`, "ok");
      setHost("");
      refreshDevices();
    } catch (e) {
      fail(e);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Page
      title="Devices"
      sub="Devices on this network running LocalSendStudio."
      actions={
        <div className="perm">
          <button className={view === "grid" ? "on" : ""} onClick={() => setView("grid")} aria-label="Grid view"><LayoutGrid size={15} /></button>
          <button className={view === "map" ? "on" : ""} onClick={() => setView("map")} aria-label="Map view"><Net size={15} /></button>
        </div>
      }
    >
      <div className="stack">
        <div className="card">
          <h2><Plus size={16} /> Add a device by IP</h2>
          <p className="muted small" style={{ marginTop: -6 }}>Use this when automatic discovery is blocked (guest Wi-Fi, VPN, some routers).</p>
          <div className="row">
            <input className="input grow" style={{ minWidth: 180 }} placeholder="192.168.1.20" value={host} onChange={(e) => setHost(e.target.value)} onKeyDown={(e) => e.key === "Enter" && add()} inputMode="decimal" />
            <button className="btn primary" disabled={busy || !host.trim()} onClick={add}>{busy ? "Connecting…" : "Add"}</button>
          </div>
        </div>

        {devices.length === 0 ? (
          <div className="card"><Empty icon={<Laptop size={24} />} title="No devices yet">Install LocalSendStudio on another device and connect it to the same network.</Empty></div>
        ) : view === "map" ? (
          <MapView devices={devices} me={identity?.name ?? "This device"} />
        ) : (
          <div className="grid cols">
            {devices.map((d) => (
              <div key={d.id} className={`card dev ${d.status === "online" ? "" : "off"}`}>
                <div className="dev-top">
                  <div className="dev-ico"><OsIcon os={d.os} /></div>
                  <div className="grow">
                    <div className="dev-name ellipsis">{d.name}</div>
                    <div className="muted small">{d.os} · v{d.version}</div>
                  </div>
                  <Dot on={d.status === "online"} />
                </div>
                <div className="chips">
                  <span className="pill mono">{d.ip}:{d.port}</span>
                  {d.trusted && <span className="pill ok"><ShieldCheck size={12} /> trusted</span>}
                  <span className="pill">{d.status === "online" ? "online" : `seen ${ago(d.last_seen)}`}</span>
                </div>
                <div className="muted small">
                  {d.shared_folders.length ? `Shares: ${d.shared_folders.join(", ")}` : "Not sharing any folders"}
                </div>
                <div className="row">
                  <button className="btn primary sm grow" disabled={d.status !== "online"} onClick={() => nav(`/send?dev=${d.id}`)}><Send size={15} /> Send</button>
                  <button className="btn sm grow" disabled={d.status !== "online" || !d.shared_folders.length} onClick={() => nav(`/browse?dev=${d.id}`)}><FolderOpen size={15} /> Browse</button>
                  <button className="btn ghost icon" aria-label="Forget device" title="Forget" onClick={async () => { if (await confirm(`Forget ${d.name}?`, { body: "It will reappear if it is still on the network.", ok: "Forget" })) { await api.forgetDevice(d.id); refreshDevices(); } }}><Trash2 size={16} /></button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </Page>
  );
}
