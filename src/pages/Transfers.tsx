import { useEffect, useMemo, useState } from "react";
import { ArrowDownToLine, ArrowLeftRight, ArrowUpFromLine, Download as Dl, Trash2, X } from "lucide-react";
import { api, HistoryItem } from "../lib/api";
import { useApp } from "../lib/store";
import { ago, bytes, eta, speed, when } from "../lib/format";
import { Bar, Empty, Page } from "../components/ui";

function csv(rows: HistoryItem[]) {
  const q = (s: string | number) => `"${String(s).replace(/"/g, '""')}"`;
  const head = ["file", "direction", "from", "to", "bytes", "status", "started"];
  return [head.join(","), ...rows.map((r) => [r.filename, r.direction, r.source, r.destination, r.size, r.status, new Date(r.started).toISOString()].map(q).join(","))].join("\n");
}

export default function Transfers() {
  const { live, dismissLive, historyTick, confirm, toast } = useApp();
  const [hist, setHist] = useState<HistoryItem[]>([]);

  useEffect(() => { api.history().then(setHist).catch(() => {}); }, [historyTick]);

  const active = Object.values(live).filter((l) => l.status === "transferring");
  const finished = Object.values(live).filter((l) => l.status !== "transferring");

  const week = useMemo(() => {
    const days = Array.from({ length: 7 }, (_, i) => { const d = new Date(); d.setHours(0, 0, 0, 0); d.setDate(d.getDate() - (6 - i)); return d; });
    return days.map((d) => {
      const end = d.getTime() + 86400000;
      const sum = hist.filter((h) => h.status === "completed" && h.started >= d.getTime() && h.started < end).reduce((a, h) => a + h.size, 0);
      return { label: d.toLocaleDateString(undefined, { weekday: "short" }), sum };
    });
  }, [hist]);
  const max = Math.max(1, ...week.map((w) => w.sum));
  const ok = hist.filter((h) => h.status === "completed");
  const avg = ok.length ? ok.reduce((a, h) => a + h.speed, 0) / ok.length : 0;

  const exportCsv = () => {
    const url = URL.createObjectURL(new Blob([csv(hist)], { type: "text/csv" }));
    const a = document.createElement("a");
    a.href = url; a.download = "transfer-history.csv"; a.click();
    setTimeout(() => URL.revokeObjectURL(url), 2000);
    toast("History exported", "ok");
  };

  return (
    <Page
      title="Transfers"
      sub="Everything sent and received."
      actions={hist.length > 0 && (
        <>
          <button className="btn sm" onClick={exportCsv}><Dl size={15} /> Export CSV</button>
          <button className="btn sm ghost" onClick={async () => { if (await confirm("Clear history?", { body: "Files are not deleted — only this list.", ok: "Clear", danger: true })) { await api.clearHistory(); setHist([]); } }}><Trash2 size={15} /> Clear</button>
        </>
      )}
    >
      <div className="stack">
        {(active.length > 0 || finished.length > 0) && (
          <div className="card">
            <h2>In progress</h2>
            {[...active, ...finished].map((t) => {
              const pct = t.total ? (t.transferred / t.total) * 100 : t.status === "completed" ? 100 : 0;
              return (
                <div key={t.id} className="xfer">
                  <div className="row">
                    {t.direction === "upload" ? <ArrowUpFromLine size={17} /> : <ArrowDownToLine size={17} />}
                    <div className="grow">
                      <div className="ellipsis" style={{ fontWeight: 600 }}>{t.filename}</div>
                      <div className="muted small">{t.direction === "upload" ? "to" : "from"} {t.device}</div>
                    </div>
                    {t.status === "transferring" ? (
                      <button className="btn sm danger" onClick={() => api.cancelTransfer(t.id)}>Cancel</button>
                    ) : (
                      <>
                        <span className={`pill ${t.status === "completed" ? "ok" : t.status === "failed" ? "err" : ""}`}>{t.status}</span>
                        <button className="btn ghost icon" aria-label="Dismiss" onClick={() => dismissLive(t.id)}><X size={15} /></button>
                      </>
                    )}
                  </div>
                  <Bar value={pct} tone={t.status === "failed" ? "err" : t.status === "completed" ? "ok" : undefined} />
                  <div className="muted small">
                    {t.status === "transferring"
                      ? `${Math.round(pct)}% · ${bytes(t.transferred)} of ${bytes(t.total)} · ${speed(t.speed)} ${eta(t.eta) && "· " + eta(t.eta)}`
                      : t.error ?? (t.status === "completed" ? `${bytes(t.total)} transferred and verified` : "")}
                  </div>
                </div>
              );
            })}
          </div>
        )}

        <div className="grid two">
          <div className="card">
            <h2>Last 7 days</h2>
            <div className="bars">
              {week.map((w, i) => (<div key={i}><i style={{ height: `${(w.sum / max) * 100}%`, opacity: w.sum ? 1 : 0.25 }} title={bytes(w.sum)} /><span>{w.label}</span></div>))}
            </div>
          </div>
          <div className="card row" style={{ alignItems: "stretch", justifyContent: "space-around" }}>
            <div className="stat"><b>{ok.length}</b><span>completed</span></div>
            <div className="stat"><b>{bytes(ok.reduce((a, h) => a + h.size, 0))}</b><span>moved in total</span></div>
            <div className="stat"><b>{speed(avg)}</b><span>average speed</span></div>
          </div>
        </div>

        <div className="card">
          <h2>History</h2>
          {hist.length === 0 ? (
            <Empty icon={<ArrowLeftRight size={22} />} title="Nothing here yet">Completed transfers are listed here.</Empty>
          ) : (
            <div className="tablewrap">
              <table>
                <thead><tr><th>File</th><th>From → To</th><th>Size</th><th>Speed</th><th>When</th><th>Status</th></tr></thead>
                <tbody>
                  {hist.slice(0, 200).map((h) => (
                    <tr key={h.id}>
                      <td style={{ maxWidth: 240 }} className="ellipsis">{h.filename}</td>
                      <td className="muted small">{h.source} → {h.destination}</td>
                      <td>{bytes(h.size)}</td>
                      <td className="muted">{h.speed ? speed(h.speed) : "–"}</td>
                      <td className="muted small" title={when(h.started)}>{ago(h.started)}</td>
                      <td><span className={`pill ${h.status === "completed" ? "ok" : h.status === "failed" ? "err" : ""}`}>{h.status}</span></td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </Page>
  );
}
