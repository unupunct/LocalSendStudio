import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { ArrowDownToLine, ArrowUpFromLine, FolderOpen, Send, Share2, Wifi } from "lucide-react";
import { api, HistoryItem } from "../lib/api";
import { useApp } from "../lib/store";
import { ago, bytes } from "../lib/format";
import { Dot, Empty, OsIcon, Page } from "../components/ui";

export default function Home() {
  const { identity, devices, historyTick } = useApp();
  const nav = useNavigate();
  const [hist, setHist] = useState<HistoryItem[]>([]);
  const [shares, setShares] = useState(0);
  const [ips, setIps] = useState<string[]>([]);

  useEffect(() => {
    api.history().then(setHist).catch(() => {});
  }, [historyTick]);
  useEffect(() => {
    api.sharedFolders().then((f) => setShares(f.filter((x) => x.enabled).length)).catch(() => {});
    api.interfaces().then((i) => setIps(i.filter((x) => x.kind !== "virtual").map((x) => x.ip))).catch(() => {});
  }, []);

  const online = devices.filter((d) => d.status === "online");
  const done = hist.filter((h) => h.status === "completed");
  const total = done.reduce((a, h) => a + h.size, 0);

  return (
    <Page title="Home" sub="Everything on your Wi-Fi or LAN, one tap away.">
      <div className="stack">
        <section className="hero">
          <div>
            <div className="row" style={{ gap: 12, marginBottom: 6 }}>
              <span className="radar" />
              <span className="muted">Visible to nearby devices as</span>
            </div>
            <h2>{identity?.name ?? "…"}</h2>
            <div className="muted small mono">{ips.length ? ips.join("  ·  ") : "No network connection"}</div>
          </div>
          <div className="actions">
            <button className="btn primary" onClick={() => nav("/send")}><Send size={17} /> Send files</button>
            <button className="btn" onClick={() => nav("/browse")}><FolderOpen size={17} /> Browse</button>
          </div>
        </section>

        <div className="grid stats">
          <div className="card stat"><b>{online.length}</b><span>devices nearby</span></div>
          <div className="card stat"><b>{shares}</b><span>folders shared</span></div>
          <div className="card stat"><b>{done.length}</b><span>transfers · {bytes(total)}</span></div>
        </div>

        <section>
          <h2><Wifi size={16} /> Nearby devices</h2>
          {online.length === 0 ? (
            <div className="card">
              <Empty icon={<Wifi size={24} />} title="Looking for devices…">
                Open LocalSendStudio on another device that's on the same network. If nothing shows up, add it by IP on the{" "}
                <Link to="/devices" style={{ color: "var(--brand)" }}>Devices</Link> page or run the <Link to="/network" style={{ color: "var(--brand)" }}>network check</Link>.
              </Empty>
            </div>
          ) : (
            <div className="grid cols">
              {online.map((d) => (
                <div key={d.id} className="card dev">
                  <div className="dev-top">
                    <div className="dev-ico"><OsIcon os={d.os} /></div>
                    <div className="grow">
                      <div className="dev-name ellipsis">{d.name}</div>
                      <div className="muted small">{d.os} · {d.ip}</div>
                    </div>
                    <Dot on />
                  </div>
                  <div className="row">
                    <button className="btn primary sm grow" onClick={() => nav(`/send?dev=${d.id}`)}><Send size={15} /> Send</button>
                    <button className="btn sm grow" disabled={d.shared_folders.length === 0} onClick={() => nav(`/browse?dev=${d.id}`)}>
                      <FolderOpen size={15} /> Browse
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>

        <section className="card">
          <div className="row" style={{ justifyContent: "space-between", marginBottom: 6 }}>
            <h2 style={{ margin: 0 }}><Share2 size={16} /> Recent activity</h2>
            <Link to="/transfers" className="small" style={{ color: "var(--brand)" }}>See all</Link>
          </div>
          {hist.length === 0 ? (
            <Empty icon={<ArrowDownToLine size={22} />} title="No transfers yet">Files you send and receive will show up here.</Empty>
          ) : (
            hist.slice(0, 6).map((h) => (
              <div key={h.id} className="file">
                {h.direction === "upload" || h.direction === "served" ? <ArrowUpFromLine size={17} className="muted" /> : <ArrowDownToLine size={17} className="muted" />}
                <div className="grow">
                  <div className="ellipsis">{h.filename}</div>
                  <div className="muted small">{h.direction === "upload" || h.direction === "served" ? `to ${h.destination}` : `from ${h.source}`} · {ago(h.started)}</div>
                </div>
                <span className="muted small">{bytes(h.size)}</span>
                <span className={`pill ${h.status === "completed" ? "ok" : h.status === "failed" ? "err" : ""}`}>{h.status}</span>
              </div>
            ))
          )}
        </section>
      </div>
    </Page>
  );
}
