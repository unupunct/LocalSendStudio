import { invoke } from "@tauri-apps/api/core";

export type Device = {
  id: string;
  name: string;
  os: string;
  version: string;
  ip: string;
  port: number;
  shared_folders: string[];
  status: "online" | "offline";
  last_seen: number;
  trusted: boolean;
};

export type SharedFolder = {
  id: string;
  name: string;
  path: string;
  enabled: boolean;
  read: boolean;
  upload: boolean;
  manage: boolean;
};

export type HistoryItem = {
  id: string;
  filename: string;
  source: string;
  destination: string;
  size: number;
  started: number;
  ended: number | null;
  speed: number;
  status: string;
  direction: string;
};

export type Identity = { id: string; name: string; os: string; version: string; port: number; trust_mode: "ask" | "open" };
export type Trusted = { id: string; name: string; added: number };
export type Entry = { name: string; is_dir: boolean; size: number; modified: number };
export type Listing = { entries: Entry[]; can_upload: boolean; can_manage: boolean };
export type Group = { id: string; name: string; device_ids: string };

const target = (d: Pick<Device, "ip" | "port">) => ({ ip: d.ip, port: d.port });

export const api = {
  identity: () => invoke<Identity>("get_identity"),
  setDeviceName: (name: string) => invoke<void>("set_device_name", { name }),
  getSetting: (key: string) => invoke<string | null>("get_settings", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_settings", { key, value }),
  downloadDir: () => invoke<string>("get_download_dir"),
  devices: () => invoke<Device[]>("get_devices"),
  addDeviceByIp: (host: string) => invoke<Device>("add_device_by_ip", { host }),
  forgetDevice: (id: string) => invoke<void>("forget_device", { id }),
  interfaces: () => invoke<{ name: string; ip: string; kind: string }[]>("get_interfaces"),
  respondApproval: (id: string, allow: boolean, remember: boolean) => invoke<void>("respond_approval", { id, allow, remember }),
  trusted: () => invoke<Trusted[]>("list_trusted"),
  untrust: (id: string) => invoke<void>("untrust_device", { id }),
  sharedFolders: () => invoke<SharedFolder[]>("get_shared_folders"),
  addSharedFolder: (path: string) => invoke<SharedFolder>("add_shared_folder", { path }),
  removeSharedFolder: (id: string) => invoke<void>("remove_shared_folder", { id }),
  toggleSharedFolder: (id: string, enabled: boolean) => invoke<void>("toggle_shared_folder", { id, enabled }),
  setFolderPerms: (id: string, read: boolean, upload: boolean, manage: boolean) => invoke<void>("set_folder_perms", { id, read, upload, manage }),
  listRemote: (d: Device, share: string, path: string) => invoke<Listing>("list_remote_dir", { ...target(d), share, path }),
  mkdirRemote: (d: Device, share: string, path: string) => invoke<void>("make_remote_dir", { ...target(d), share, path }),
  deleteRemote: (d: Device, share: string, path: string) => invoke<void>("delete_remote_path", { ...target(d), share, path }),
  renameRemote: (d: Device, share: string, path: string, newName: string) => invoke<void>("rename_remote_path", { ...target(d), share, path, newName }),
  download: (d: Device, share: string, remotePath: string, filename: string, localDir?: string) =>
    invoke<string>("start_download", { ...target(d), share, remotePath, filename, deviceName: d.name, localDir: localDir ?? null }),
  upload: (d: Device, share: string, remoteDir: string, localPath: string) =>
    invoke<string>("start_upload", { ...target(d), share, remoteDir, localPath, deviceName: d.name }),
  cancelTransfer: (id: string) => invoke<void>("cancel_transfer", { id }),
  history: () => invoke<HistoryItem[]>("get_transfer_history"),
  clearHistory: () => invoke<void>("clear_transfer_history"),
  groups: () => invoke<Group[]>("get_groups"),
  addGroup: (name: string, deviceIds: string) => invoke<Group>("add_group", { name, deviceIds }),
  deleteGroup: (id: string) => invoke<void>("delete_group", { id }),
  syncUp: (localPath: string, d: Device, share: string, remoteBase: string) =>
    invoke<{ copied: number; skipped: number }>("sync_local_to_remote", { localPath, ...target(d), share, remoteBase }),
  syncDown: (d: Device, share: string, remotePath: string, localBase: string) =>
    invoke<{ copied: number; skipped: number }>("sync_remote_to_local", { ...target(d), share, remotePath, localBase }),
  diagnostics: () => invoke<any>("network_diagnostics"),
  speedTest: (d: Device) => invoke<{ bytes_per_sec: number; rtt_ms: number }>("speed_test", target(d)),
  storage: () => invoke<{ path: string; available: number; total: number }>("get_local_storage"),
};
