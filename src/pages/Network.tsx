import { useCallback, useEffect, useState } from "react";
import { Check, Gauge, RefreshCw, TriangleAlert, X } from "lucide-react";
import { api } from "../lib/api";
import { useApp } from "../lib/store";
import { speed } from "../lib/format";
import { Page } from "../components/ui";

type Check = { level: "ok" | "warn" | "err"; title: string; detail: string };

export default function Network() {
  const { devices, fail } = useApp();
  const [diag, setDiag] = useState<any>(null);
  const [target, setTarget] = useState("");
  const [res, setRes] = useState<{ bytes_per_sec: number; rtt_ms: number } | null>(null);
  const [testing, setTesting] = useState(false);

  const run = useCallback(() => api.diagnostics().then(setDiag).catch(fail), [fail]);
  useEffect(() => { run(); }, [run]);

  const online = devices.filter((d) => d.status === "online");
  const checks: Check[] = !diag ? [] : [
    diag.usable_interfaces > 0
      ? { level: "ok", title: "Connected to a network", detail: diag.interfaces.filter((i: any) => i.kind !== "virtual").map((i: any) => `${i.name} (${i.ip})`).join(", ") }
      : { level: "err", title: "No network connection", detail: "Connect to Wi-Fi or Ethernet. Virtual adapters (VPN, Docker, Hyper-V) don't count." },
    diag.self_reachable
      ? { level: "ok", title: `File server is running on port ${diag.port}`, detail: "Other devices connect to this port." }
      : { level: "err", title: "File server is not responding", detail: "Another program may be using the port. Restart the app." },
    diag.devices_online > 0
      ? { level: "ok", title: `${diag.devices_online} device${diag.devices_online > 1 ? "s" : ""} found`, detail: "Automatic discovery is working." }
      : { level: "warn", title: "No other devices found", detail: `Check that the other device runs the app on the same network, and that your firewall allows UDP ${diag.discovery_port} and TCP ${diag.port}. Guest Wi-Fi and VPNs often block discovery — add the device by IP instead.` },
    diag.interfaces.some((i: any) => i.kind === "virtual") && diag.usable_interfaces > 0
      ? { level: "warn", title: "Virtual adapters detected", detail: "VPN, Docker or VM adapters can confuse discovery. If devices don't appear, disconnect them briefly." }
      : { level: "ok", title: "No conflicting adapters", detail: "" },
    diag.trust_mode === "open"
      ? { level: "warn", title: "Open mode is on", detail: "Any device on this network can access your shared folders without asking. Switch to “Ask first” in Settings." }
      : { level: "ok", title: "New devices must be approved", detail: "You're asked before an unknown device can access anything." },
    diag.shared_folders === 0 ? { level: "warn", title: "No folders shared", detail: "You can still send and receive through the Inbox." } : { level: "ok", title: `${diag.shared_folders} folder${diag.shared_folders > 1 ? "s" : ""} shared`, detail: "" },
  ];
  const Ico = { ok: Check, warn: TriangleAlert, err: X };

  const test = async () => {
    const d = online.find((x) => x.id === target);
    if (!d) return;
    setTesting(true); setRes(null);
    try { setRes(await api.speedTest(d)); } catch (e) { fail(e); } finally { setTesting(false); }
  };

  return (
    <Page title="Network" sub="If something isn't working, start here." actions={<button className="btn" onClick={run}><RefreshCw size={16} /> Re-check</button>}>
      <div className="stack">
        <div className="card">
          <h2>Checks</h2>
          <ul className="checks">
            {checks.map((c, i) => {
              const I = Ico[c.level];
              return (
                <li key={i}>
                  <span className={`ico ${c.level}`}><I size={16} /></span>
                  <div><div style={{ fontWeight: 600 }}>{c.title}</div>{c.detail && <div className="muted small">{c.detail}</div>}</div>
                </li>
              );
            })}
          </ul>
        </div>
        <div className="card">
          <h2><Gauge size={16} /> Speed test</h2>
          <p className="muted small" style={{ marginTop: -6 }}>Sends 32 MB between this device and another one to measure real throughput.</p>
          <div className="row">
            <select className="input grow" value={target} onChange={(e) => setTarget(e.target.value)}>
              <option value="">Choose a device…</option>
              {online.map((d) => <option key={d.id} value={d.id}>{d.name}</option>)}
            </select>
            <button className="btn primary" disabled={!target || testing} onClick={test}>{testing ? "Testing…" : "Run test"}</button>
          </div>
          {res && (
            <div className="row" style={{ marginTop: 16, gap: 36 }}>
              <div className="stat"><b>{speed(res.bytes_per_sec)}</b><span>throughput</span></div>
              <div className="stat"><b>{res.rtt_ms} ms</b><span>response time</span></div>
            </div>
          )}
        </div>
      </div>
    </Page>
  );
}
