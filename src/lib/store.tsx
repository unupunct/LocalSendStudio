import { createContext, ReactNode, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, Device, Identity } from "./api";
import { errText } from "./format";

export type Live = {
  id: string;
  filename: string;
  direction: "download" | "upload";
  device: string;
  transferred: number;
  total: number;
  speed: number;
  eta: number;
  status: "transferring" | "completed" | "failed" | "cancelled";
  error?: string;
};

type Toast = { id: number; kind: "ok" | "err" | "info"; text: string };
type Approval = { id: string; name: string; ip: string };
type Ask =
  | { kind: "confirm"; title: string; body?: string; danger?: boolean; ok: string; resolve: (v: boolean) => void }
  | { kind: "prompt"; title: string; initial: string; ok: string; resolve: (v: string | null) => void };

type Ctx = {
  identity: Identity | null;
  refreshIdentity: () => void;
  devices: Device[];
  refreshDevices: () => Promise<void>;
  live: Record<string, Live>;
  dismissLive: (id: string) => void;
  historyTick: number;
  approvals: Approval[];
  answerApproval: (a: Approval, allow: boolean, remember: boolean) => void;
  toasts: Toast[];
  toast: (text: string, kind?: Toast["kind"]) => void;
  ask: Ask | null;
  confirm: (title: string, o?: { body?: string; danger?: boolean; ok?: string }) => Promise<boolean>;
  prompt: (title: string, initial?: string, ok?: string) => Promise<string | null>;
  closeAsk: () => void;
  fail: (e: unknown) => void;
};

const C = createContext<Ctx>(null as unknown as Ctx);
export const useApp = () => useContext(C);

export function AppProvider({ children }: { children: ReactNode }) {
  const [identity, setIdentity] = useState<Identity | null>(null);
  const [devices, setDevices] = useState<Device[]>([]);
  const [live, setLive] = useState<Record<string, Live>>({});
  const [historyTick, setHistoryTick] = useState(0);
  const [approvals, setApprovals] = useState<Approval[]>([]);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [ask, setAsk] = useState<Ask | null>(null);
  const seq = useRef(0);

  const toast = useCallback((text: string, kind: Toast["kind"] = "info") => {
    const id = ++seq.current;
    setToasts((t) => [...t.slice(-3), { id, kind, text }]);
    setTimeout(() => setToasts((t) => t.filter((x) => x.id !== id)), kind === "err" ? 7000 : 3800);
  }, []);
  const fail = useCallback((e: unknown) => toast(errText(e), "err"), [toast]);

  const refreshIdentity = useCallback(() => {
    api.identity().then(setIdentity).catch(() => {});
  }, []);
  const refreshDevices = useCallback(async () => {
    try {
      setDevices(await api.devices());
    } catch {}
  }, []);

  useEffect(() => {
    refreshIdentity();
    refreshDevices();
    const t = setInterval(refreshDevices, 3000);
    const subs = [
      listen("devices-updated", refreshDevices),
      listen<any>("transfer-started", (e) => {
        const p = e.payload;
        setLive((l) => ({ ...l, [p.id]: { id: p.id, filename: p.filename, direction: p.direction, device: p.device, transferred: 0, total: 0, speed: 0, eta: -1, status: "transferring" } }));
      }),
      listen<any>("transfer-progress", (e) => {
        const p = e.payload;
        setLive((l) => (l[p.id] ? { ...l, [p.id]: { ...l[p.id], transferred: p.transferred, total: p.total, speed: p.speed, eta: p.eta } } : l));
      }),
      listen<any>("transfer-done", (e) => {
        const p = e.payload;
        setLive((l) => (l[p.id] ? { ...l, [p.id]: { ...l[p.id], status: p.status, error: p.error ?? undefined } } : l));
        setHistoryTick((n) => n + 1);
        if (p.status === "completed") toast("Transfer complete", "ok");
        else if (p.status === "failed") toast(p.error || "Transfer failed", "err");
      }),
      listen("history-updated", () => setHistoryTick((n) => n + 1)),
      listen<any>("incoming", (e) => toast(`Receiving ${e.payload.file} from ${e.payload.from}`, "info")),
      listen<any>("incoming-done", (e) => e.payload.status === "completed" && toast(`Received ${e.payload.file}`, "ok")),
      listen<Approval>("approval-request", (e) => setApprovals((a) => (a.some((x) => x.id === e.payload.id) ? a : [...a, e.payload]))),
      listen<{ id: string }>("approval-expired", (e) => setApprovals((a) => a.filter((x) => x.id !== e.payload.id))),
    ];
    return () => {
      clearInterval(t);
      subs.forEach((s) => s.then((f) => f()));
    };
  }, [refreshDevices, refreshIdentity, toast]);

  const dismissLive = useCallback((id: string) => setLive((l) => { const n = { ...l }; delete n[id]; return n; }), []);

  const answerApproval = useCallback((a: Approval, allow: boolean, remember: boolean) => {
    setApprovals((x) => x.filter((y) => y.id !== a.id));
    api.respondApproval(a.id, allow, remember).catch(() => {});
  }, []);

  const confirm = useCallback(
    (title: string, o?: { body?: string; danger?: boolean; ok?: string }) =>
      new Promise<boolean>((resolve) => setAsk({ kind: "confirm", title, body: o?.body, danger: o?.danger, ok: o?.ok ?? "OK", resolve })),
    []
  );
  const prompt = useCallback(
    (title: string, initial = "", ok = "OK") => new Promise<string | null>((resolve) => setAsk({ kind: "prompt", title, initial, ok, resolve })),
    []
  );
  const closeAsk = useCallback(() => setAsk(null), []);

  const value = useMemo(
    () => ({ identity, refreshIdentity, devices, refreshDevices, live, dismissLive, historyTick, approvals, answerApproval, toasts, toast, ask, confirm, prompt, closeAsk, fail }),
    [identity, refreshIdentity, devices, refreshDevices, live, dismissLive, historyTick, approvals, answerApproval, toasts, toast, ask, confirm, prompt, closeAsk, fail]
  );
  return <C.Provider value={value}>{children}</C.Provider>;
}
