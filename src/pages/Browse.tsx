import { useCallback, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { ChevronRight, Download, FolderOpen, FolderPlus, Lock, Pencil, RefreshCw, Trash2, Upload } from "lucide-react";
import { api, Device, Listing } from "../lib/api";
import { useApp } from "../lib/store";
import { bytes, joinPath, when } from "../lib/format";
import { pickFiles } from "../lib/pick";
import { Empty, FileIcon, OsIcon, Page } from "../components/ui";

export default function Browse() {
  const { devices, toast, fail, confirm, prompt } = useApp();
  const [params, setParams] = useSearchParams();
  const [path, setPath] = useState("");
  const [data, setData] = useState<Listing | null>(null);
  const [loading, setLoading] = useState(false);
  const [err, setErr] = useState("");

  const devId = params.get("dev");
  const share = params.get("share");
  const device: Device | undefined = useMemo(() => devices.find((d) => d.id === devId), [devices, devId]);

  // Pick something sensible automatically.
  useEffect(() => {
    if (!devId) return;
    const d = devices.find((x) => x.id === devId);
    if (d && !share && d.shared_folders.length) setParams({ dev: d.id, share: d.shared_folders[0] }, { replace: true });
  }, [devId, share, devices, setParams]);

  const load = useCallback(async () => {
    if (!device || !share) return;
    setLoading(true);
    setErr("");
    try {
      setData(await api.listRemote(device, share, path));
    } catch (e) {
      setData(null);
      setErr(String((e as any)?.message ?? e));
    } finally {
      setLoading(false);
    }
  }, [device, share, path]);

  useEffect(() => { load(); }, [load]);
  useEffect(() => setPath(""), [devId, share]);

  const crumbs = path.split("/").filter(Boolean);
  const cd = (i: number) => setPath(crumbs.slice(0, i).join("/"));

  const download = async (name: string) => {
    if (!device || !share) return;
    try {
      await api.download(device, share, joinPath(path, name), name);
      toast(`Downloading ${name} — see Transfers`, "info");
    } catch (e) { fail(e); }
  };
  const upload = async () => {
    if (!device || !share) return;
    try {
      const files = await pickFiles(true);
      for (const f of files) await api.upload(device, share, path, f);
      if (files.length) toast(`Uploading ${files.length} file${files.length > 1 ? "s" : ""}`, "info");
      setTimeout(load, 1500);
    } catch (e) { fail(e); }
  };
  const mkdir = async () => {
    const n = await prompt("New folder", "", "Create");
    if (!n || !device || !share) return;
    try { await api.mkdirRemote(device, share, joinPath(path, n)); load(); } catch (e) { fail(e); }
  };
  const rename = async (name: string) => {
    const n = await prompt("Rename", name, "Rename");
    if (!n || n === name || !device || !share) return;
    try { await api.renameRemote(device, share, joinPath(path, name), n); load(); } catch (e) { fail(e); }
  };
  const remove = async (name: string) => {
    if (!device || !share) return;
    if (!(await confirm(`Delete “${name}”?`, { body: "It will be deleted on the other device. This cannot be undone.", danger: true, ok: "Delete" }))) return;
    try { await api.deleteRemote(device, share, joinPath(path, name)); load(); } catch (e) { fail(e); }
  };

  const withShares = devices.filter((d) => d.shared_folders.length > 0);

  return (
    <Page title="Browse" sub="Look inside folders other devices are sharing.">
      <div className="browser">
        <div className="card" style={{ padding: 10 }}>
          {withShares.length === 0 ? (
            <Empty icon={<FolderOpen size={22} />} title="Nothing shared">No device is sharing a folder yet.</Empty>
          ) : (
            withShares.map((d) => (
              <div key={d.id} className="tree-dev" style={{ opacity: d.status === "online" ? 1 : 0.5 }}>
                <div className="row" style={{ gap: 8 }}><OsIcon os={d.os} size={16} /><span className="ellipsis">{d.name}</span></div>
                {d.shared_folders.map((s) => (
                  <button key={s} className={`tree-share ${d.id === devId && s === share ? "active" : ""}`} onClick={() => setParams({ dev: d.id, share: s })}>
                    <FileIcon name={s} dir size={16} /> <span className="ellipsis">{s}</span>
                  </button>
                ))}
              </div>
            ))
          )}
        </div>

        <div className="card">
          {!device || !share ? (
            <Empty icon={<FolderOpen size={24} />} title="Choose a folder">Pick a shared folder on the left to see its files.</Empty>
          ) : (
            <>
              <div className="row" style={{ justifyContent: "space-between", marginBottom: 6 }}>
                <div className="crumbs">
                  <button onClick={() => cd(0)}>{device.name} / {share}</button>
                  {crumbs.map((c, i) => (<span key={i} className="row" style={{ gap: 0 }}><ChevronRight size={14} className="sep" /><button onClick={() => cd(i + 1)}>{c}</button></span>))}
                </div>
                <div className="row">
                  <button className="btn ghost icon" onClick={load} aria-label="Refresh"><RefreshCw size={16} /></button>
                  {data?.can_upload && <button className="btn sm" onClick={upload}><Upload size={15} /> Upload</button>}
                  {data?.can_manage && <button className="btn sm" onClick={mkdir}><FolderPlus size={15} /> New folder</button>}
                </div>
              </div>
              {err ? (
                <Empty icon={<Lock size={22} />} title="Can't open this folder">{err}</Empty>
              ) : loading && !data ? (
                <p className="muted">Loading…</p>
              ) : data && data.entries.length === 0 ? (
                <Empty icon={<FolderOpen size={22} />} title="Empty folder" />
              ) : (
                data?.entries.map((e) => (
                  <div key={e.name} className="file">
                    <button className="name" onClick={() => (e.is_dir ? setPath(joinPath(path, e.name)) : download(e.name))}>
                      <FileIcon name={e.name} dir={e.is_dir} /> <span className="ellipsis">{e.name}</span>
                    </button>
                    <span className="muted small" style={{ minWidth: 64, textAlign: "right" }}>{e.is_dir ? "" : bytes(e.size)}</span>
                    <span className="muted small" style={{ minWidth: 100 }}>{e.modified ? when(e.modified * 1000) : ""}</span>
                    <div className="acts">
                      {!e.is_dir && <button className="btn ghost icon" aria-label={`Download ${e.name}`} onClick={() => download(e.name)}><Download size={16} /></button>}
                      {data.can_manage && <button className="btn ghost icon" aria-label={`Rename ${e.name}`} onClick={() => rename(e.name)}><Pencil size={16} /></button>}
                      {data.can_manage && <button className="btn ghost icon" aria-label={`Delete ${e.name}`} onClick={() => remove(e.name)}><Trash2 size={16} /></button>}
                    </div>
                  </div>
                ))
              )}
            </>
          )}
        </div>
      </div>
    </Page>
  );
}
