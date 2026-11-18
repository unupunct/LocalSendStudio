import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { open } from "@tauri-apps/plugin-dialog";

export default function GlobalExplorer() {
  const [devices, setDevices] = useState<any[]>([]);
  const [selected, setSelected] = useState<any>(null);
  const [share, setShare] = useState<string>("");
  const [path, setPath] = useState<string>("");
  const [entries, setEntries] = useState<any[]>([]);
  const [status, setStatus] = useState("");

  useEffect(() => {
    (async () => {
      const ds = await api.devices();
      setDevices(ds);
      const saved = localStorage.getItem("selectedDevice");
      if (saved) {
        const d = JSON.parse(saved);
        setSelected(d);
      }
    })();
  }, []);

  useEffect(() => {
    if (selected && share) load();
  }, [selected, share, path]);

  const load = async () => {
    try {
      const data = await api.listRemoteDir(selected.ip, selected.port, share, path);
      setEntries(data.entries || []);
    } catch (e: any) { setStatus(String(e)); }
  };

  const navigate = (name: string) => {
    setPath(p => p ? p + "/" + name : name);
  };
  const up = () => {
    const parts = path.split("/").filter(Boolean);
    parts.pop();
    setPath(parts.join("/"));
  };

  const download = async (entry: any) => {
    const remotePath = path ? path + "/" + entry.name : entry.name;
    const local = await open({ directory: true, multiple: false });
    if (!local) return;
    const localPath = (local as string) + "/" + entry.name;
    const id = crypto.randomUUID();
    await api.download(selected.ip, selected.port, share, remotePath, localPath, id, entry.name, selected.name);
    setStatus("Download started: " + entry.name);
  };

  const uploadFile = async () => {
    const file = await open({ multiple: false });
    if (!file) return;
    const fp = file as string;
    const name = fp.split(/[\\/]/).pop()!;
    const remotePath = path ? path + "/" + name : name;
    await api.upload(selected.ip, selected.port, share, remotePath, fp);
    setStatus("Uploaded " + name);
    load();
  };

  return (
    <div>
      <h1>Global Explorer</h1>
      <div className="grid grid-2">
        <div className="card">
          <h2>Network Storage</h2>
          {devices.map(d => (
            <div key={d.id} style={{ marginBottom: 8 }}>
              <div style={{ fontWeight: 600 }}>{d.name}</div>
              {d.shared_folders.map((sh: string) => (
                <div key={sh} style={{ marginLeft: 12, cursor: "pointer", color: selected?.id === d.id && share === sh ? "var(--accent)" : "var(--text)" }}
                  onClick={() => { setSelected(d); setShare(sh); setPath(""); localStorage.setItem("selectedDevice", JSON.stringify(d)); }}>
                  📁 {sh}
                </div>
              ))}
            </div>
          ))}
          {devices.length === 0 && <div className="muted">No devices discovered.</div>}
        </div>
        <div className="card">
          <h2>{selected ? selected.name + " / " + share : "Select a folder"}</h2>
          <div className="row" style={{ marginBottom: 8 }}>
            <button className="btn secondary" onClick={up} disabled={!path}>Up</button>
            <button className="btn secondary" onClick={uploadFile}>Upload</button>
            <button className="btn secondary" onClick={() => { const n = prompt("Folder name"); if (n) api.mkdirRemote(selected.ip, selected.port, share, (path ? path + "/" : "") + n).then(load); }}>New Folder</button>
          </div>
          <div className="muted" style={{ marginBottom: 8 }}>{path || "/"}</div>
          <table>
            <thead><tr><th>Name</th><th>Type</th><th>Size</th><th></th></tr></thead>
            <tbody>
              {entries.map((e, i) => (
                <tr key={i}>
                  <td style={{ cursor: e.is_dir ? "pointer" : "default" }} onClick={() => e.is_dir && navigate(e.name)}>
                    {e.is_dir ? "📁" : "📄"} {e.name}
                  </td>
                  <td>{e.is_dir ? "Folder" : "File"}</td>
                  <td>{e.is_dir ? "-" : (e.size / 1024) + " KB"}</td>
                  <td>
                    {!e.is_dir && <button className="btn secondary" onClick={() => download(e)}>Download</button>}
                    <button className="btn danger" style={{ marginLeft: 4 }} onClick={() => {
                      if (confirm("Delete " + e.name + "?")) api.deleteRemote(selected.ip, selected.port, share, (path ? path + "/" : "") + e.name).then(load);
                    }}>Del</button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {status && <div className="muted" style={{ marginTop: 8 }}>{status}</div>}
        </div>
      </div>
    </div>
  );
}
