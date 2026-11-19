import { ReactNode, useEffect, useRef, useState } from "react";
import { Apple, Check, FileText, Folder, Laptop, Monitor, Smartphone, Terminal, X, ShieldQuestion, Image, Film, Music, Archive, FileCode } from "lucide-react";
import { useApp } from "../lib/store";

export function OsIcon({ os, size = 22 }: { os: string; size?: number }) {
  const o = os.toLowerCase();
  if (o.includes("android") || o.includes("ios")) return <Smartphone size={size} />;
  if (o.includes("mac")) return <Apple size={size} />;
  if (o.includes("linux")) return <Terminal size={size} />;
  if (o.includes("windows")) return <Monitor size={size} />;
  return <Laptop size={size} />;
}

export function FileIcon({ name, dir, size = 18 }: { name: string; dir?: boolean; size?: number }) {
  if (dir) return <Folder size={size} className="ico-folder" />;
  const e = name.split(".").pop()?.toLowerCase() ?? "";
  if (["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "heic"].includes(e)) return <Image size={size} />;
  if (["mp4", "mkv", "mov", "avi", "webm"].includes(e)) return <Film size={size} />;
  if (["mp3", "wav", "flac", "ogg", "m4a"].includes(e)) return <Music size={size} />;
  if (["zip", "rar", "7z", "tar", "gz"].includes(e)) return <Archive size={size} />;
  if (["js", "ts", "tsx", "rs", "py", "json", "html", "css", "md", "sh"].includes(e)) return <FileCode size={size} />;
  return <FileText size={size} />;
}

export function Dot({ on }: { on: boolean }) {
  return <span className={`dot ${on ? "on" : ""}`} aria-label={on ? "online" : "offline"} />;
}

export function Page({ title, sub, actions, children }: { title: string; sub?: string; actions?: ReactNode; children: ReactNode }) {
  return (
    <div className="page">
      <header className="page-head">
        <div>
          <h1>{title}</h1>
          {sub && <p className="sub">{sub}</p>}
        </div>
        {actions && <div className="actions">{actions}</div>}
      </header>
      {children}
    </div>
  );
}

export function Empty({ icon, title, children }: { icon: ReactNode; title: string; children?: ReactNode }) {
  return (
    <div className="empty">
      <div className="empty-ico">{icon}</div>
      <strong>{title}</strong>
      {children && <p>{children}</p>}
    </div>
  );
}

export function Bar({ value, tone }: { value: number; tone?: "ok" | "err" }) {
  return (
    <div className={`bar ${tone ?? ""}`} role="progressbar" aria-valuenow={Math.round(value)} aria-valuemin={0} aria-valuemax={100}>
      <div style={{ width: `${Math.min(100, Math.max(0, value))}%` }} />
    </div>
  );
}

export function Switch({ checked, onChange, label }: { checked: boolean; onChange: (v: boolean) => void; label: string }) {
  return (
    <button role="switch" aria-checked={checked} aria-label={label} className={`switch ${checked ? "on" : ""}`} onClick={() => onChange(!checked)}>
      <span />
    </button>
  );
}

/** Toasts, confirm/prompt dialogs and the "allow this device?" prompt. */
export function Overlays() {
  const { toasts, ask, closeAsk, approvals, answerApproval } = useApp();
  const [text, setText] = useState("");
  const [remember, setRemember] = useState(true);
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (ask?.kind === "prompt") {
      setText(ask.initial);
      setTimeout(() => input.current?.select(), 30);
    }
  }, [ask]);

  const done = (v: boolean) => {
    if (!ask) return;
    if (ask.kind === "confirm") ask.resolve(v);
    else ask.resolve(v ? text.trim() || null : null);
    closeAsk();
  };
  const approval = approvals[0];

  return (
    <>
      <div className="toasts" aria-live="polite">
        {toasts.map((t) => (
          <div key={t.id} className={`toast ${t.kind}`}>
            {t.kind === "ok" ? <Check size={16} /> : t.kind === "err" ? <X size={16} /> : null}
            <span>{t.text}</span>
          </div>
        ))}
      </div>

      {ask && (
        <div className="scrim" onMouseDown={(e) => e.target === e.currentTarget && done(false)}>
          <div className="dialog" role="dialog" aria-modal="true" aria-label={ask.title}>
            <h3>{ask.title}</h3>
            {ask.kind === "confirm" && ask.body && <p className="muted">{ask.body}</p>}
            {ask.kind === "prompt" && (
              <input ref={input} className="input" value={text} onChange={(e) => setText(e.target.value)} onKeyDown={(e) => e.key === "Enter" && done(true)} />
            )}
            <div className="dialog-actions">
              <button className="btn ghost" onClick={() => done(false)}>Cancel</button>
              <button className={`btn ${ask.kind === "confirm" && ask.danger ? "danger" : "primary"}`} onClick={() => done(true)}>{ask.ok}</button>
            </div>
          </div>
        </div>
      )}

      {approval && (
        <div className="scrim">
          <div className="dialog approval" role="alertdialog" aria-modal="true" aria-label="Allow device">
            <div className="approval-ico"><ShieldQuestion size={28} /></div>
            <h3>Allow “{approval.name}”?</h3>
            <p className="muted">
              A device at <b>{approval.ip}</b> wants to access this device
              {approvals.length > 1 ? ` (${approvals.length - 1} more waiting)` : ""}. It can only see folders you share, with the permissions you set.
            </p>
            <label className="check">
              <input type="checkbox" checked={remember} onChange={(e) => setRemember(e.target.checked)} /> Always allow this device
            </label>
            <div className="dialog-actions">
              <button className="btn ghost" onClick={() => answerApproval(approval, false, false)}>Deny</button>
              <button className="btn primary" onClick={() => answerApproval(approval, true, remember)}>Allow</button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
