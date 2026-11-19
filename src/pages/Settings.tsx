import { useCallback, useEffect, useState } from "react";
import { HardDrive, ShieldCheck, ShieldOff, Trash2 } from "lucide-react";
import { api, Trusted } from "../lib/api";
import { useApp } from "../lib/store";
import { bytes, ago } from "../lib/format";
import { pickFolder } from "../lib/pick";
import { Page, Switch } from "../components/ui";

export default function Settings() {
  const { identity, refreshIdentity, toast, fail, confirm } = useApp();
  const [name, setName] = useState("");
  const [dl, setDl] = useState("");
  const [trusted, setTrusted] = useState<Trusted[]>([]);
  const [tray, setTray] = useState(true);
  const [theme, setTheme] = useState<string>(() => { try { return localStorage.getItem("theme") ?? "auto"; } catch { return "auto"; } });
  const [store, setStore] = useState<{ available: number; total: number } | null>(null);

  const load = useCallback(() => {
    api.downloadDir().then(setDl).catch(() => {});
    api.trusted().then(setTrusted).catch(() => {});
    api.getSetting("close_to_tray").then((v) => setTray(v !== "false")).catch(() => {});
    api.storage().then(setStore).catch(() => {});
  }, []);
  useEffect(() => { load(); }, [load]);
  useEffect(() => { if (identity) setName(identity.name); }, [identity]);

  const applyTheme = (t: string) => {
    setTheme(t);
    try { localStorage.setItem("theme", t); } catch {}
    if (t === "auto") document.documentElement.removeAttribute("data-theme");
    else document.documentElement.setAttribute("data-theme", t);
  };

  const saveName = async () => {
    try { await api.setDeviceName(name); refreshIdentity(); toast("Name saved — other devices see it within seconds", "ok"); } catch (e) { fail(e); }
  };
  const setMode = async (m: "ask" | "open") => {
    if (m === "open" && !(await confirm("Trust every device on this network?", { body: "Anyone on your Wi-Fi or LAN will be able to use your shared folders without asking. Only choose this on a network you fully control.", ok: "Trust everyone", danger: true }))) return;
    await api.setSetting("trust_mode", m);
    refreshIdentity();
  };
  const pickDl = async () => {
    const p = await pickFolder();
    if (!p) return;
    await api.setSetting("download_dir", p);
    load();
  };

  const mode = identity?.trust_mode ?? "ask";
  return (
    <Page title="Settings">
      <div className="stack">
        <div className="card">
          <h2>This device</h2>
          <div className="row">
            <input className="input grow" value={name} maxLength={60} onChange={(e) => setName(e.target.value)} onKeyDown={(e) => e.key === "Enter" && saveName()} aria-label="Device name" />
            <button className="btn primary" disabled={!name.trim() || name === identity?.name} onClick={saveName}>Save</button>
          </div>
        </div>

        <div className="card">
          <h2>Who can connect</h2>
          <div className="perm" role="radiogroup" aria-label="Trust mode">
            <button role="radio" aria-checked={mode === "ask"} className={mode === "ask" ? "on" : ""} onClick={() => setMode("ask")}><ShieldCheck size={14} style={{ verticalAlign: -2 }} /> Ask first</button>
            <button role="radio" aria-checked={mode === "open"} className={`${mode === "open" ? "on risk" : ""}`} onClick={() => setMode("open")}><ShieldOff size={14} style={{ verticalAlign: -2 }} /> Trust everyone</button>
          </div>
          <p className="muted small">
            {mode === "ask" ? "A new device has to be approved by you the first time. Approved devices are remembered." : "Every device on the network is allowed without asking. Not recommended on public or shared networks."}
          </p>
          {trusted.length > 0 && (
            <>
              <h2 style={{ marginTop: 18 }}>Approved devices</h2>
              {trusted.map((t) => (
                <div key={t.id} className="file">
                  <ShieldCheck size={17} color="var(--ok)" />
                  <div className="grow"><div>{t.name}</div><div className="muted small">approved {ago(t.added)}</div></div>
                  <button className="btn ghost sm" onClick={async () => { await api.untrust(t.id); load(); }}><Trash2 size={14} /> Revoke</button>
                </div>
              ))}
            </>
          )}
        </div>

        <div className="card">
          <h2><HardDrive size={16} /> Received files</h2>
          <div className="row">
            <input className="input grow mono" readOnly value={dl} aria-label="Download folder" />
            <button className="btn" onClick={pickDl}>Change…</button>
          </div>
          {store && store.total > 0 && <p className="muted small">{bytes(store.available)} free of {bytes(store.total)}</p>}
        </div>

        <div className="card">
          <h2>App</h2>
          <div className="row" style={{ justifyContent: "space-between", marginBottom: 14 }}>
            <div><div>Theme</div><div className="muted small">Follows your system by default.</div></div>
            <div className="perm">
              {["auto", "light", "dark"].map((t) => <button key={t} className={theme === t ? "on" : ""} onClick={() => applyTheme(t)}>{t[0].toUpperCase() + t.slice(1)}</button>)}
            </div>
          </div>
          <div className="row" style={{ justifyContent: "space-between" }}>
            <div><div>Keep running in the background</div><div className="muted small">Closing the window hides it to the tray so you can still receive files.</div></div>
            <Switch checked={tray} label="Close to tray" onChange={(v) => { setTray(v); api.setSetting("close_to_tray", String(v)); }} />
          </div>
        </div>

        <p className="muted small" style={{ textAlign: "center" }}>LocalSendStudio v{identity?.version} · listening on port {identity?.port} · transfers are verified with SHA-256 but not encrypted</p>
      </div>
    </Page>
  );
}
