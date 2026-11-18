import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { open } from "@tauri-apps/plugin-dialog";

export default function SharedFolders() {
  const [folders, setFolders] = useState<any[]>([]);
  const refresh = async () => setFolders(await api.sharedFolders());
  useEffect(() => { refresh(); }, []);

  const add = async () => {
    const d = await open({ directory: true, multiple: false });
    if (d) {
      await api.addSharedFolder(d as string, undefined);
      refresh();
    }
  };

  return (
    <div>
      <h1>Shared Folders</h1>
      <button className="btn" onClick={add}>Add Folder</button>
      <div className="grid" style={{ marginTop: 16 }}>
        {folders.map(f => (
          <div key={f.id} className="card">
            <div className="row">
              <strong>{f.name}</strong>
              <span className="muted">{f.path}</span>
            </div>
            <div className="row" style={{ marginTop: 8 }}>
              <label><input type="checkbox" checked={f.enabled} onChange={e => api.toggleSharedFolder(f.id, e.target.checked).then(refresh)} /> Enabled</label>
              <label><input type="checkbox" checked={f.read} onChange={e => api.setFolderPerms(f.id, e.target.checked, f.upload, f.manage).then(refresh)} /> Read</label>
              <label><input type="checkbox" checked={f.upload} onChange={e => api.setFolderPerms(f.id, f.read, e.target.checked, f.manage).then(refresh)} /> Upload</label>
              <label><input type="checkbox" checked={f.manage} onChange={e => api.setFolderPerms(f.id, f.read, f.upload, e.target.checked).then(refresh)} /> Manage</label>
              <button className="btn danger" onClick={() => api.removeSharedFolder(f.id).then(refresh)}>Remove</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
