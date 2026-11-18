import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export const api = {
  identity: () => invoke<any>("get_identity"),
  setDeviceName: (name: string) => invoke<void>("set_device_name", { name }),
  devices: () => invoke<any[]>("get_devices"),
  interfaces: () => invoke<any[]>("get_interfaces"),
  sharedFolders: () => invoke<any[]>("get_shared_folders"),
  addSharedFolder: (path: string, name?: string) => invoke<any>("add_shared_folder", { path, name }),
  removeSharedFolder: (id: string) => invoke<void>("remove_shared_folder", { id }),
  toggleSharedFolder: (id: string, enabled: boolean) => invoke<void>("toggle_shared_folder", { id, enabled }),
  setFolderPerms: (id: string, read: boolean, upload: boolean, manage: boolean) => invoke<void>("set_folder_perms", { id, read, upload, manage }),
  listRemoteDir: (ip: string, port: number, share: string, path: string) => invoke<any>("list_remote_dir", { ip, port, share, path }),
  remoteInfo: (ip: string, port: number) => invoke<any>("remote_info", { ip, port }),
  mkdirRemote: (ip: string, port: number, share: string, path: string) => invoke<void>("make_remote_dir", { ip, port, share, path }),
  deleteRemote: (ip: string, port: number, share: string, path: string) => invoke<void>("delete_remote_path", { ip, port, share, path }),
  renameRemote: (ip: string, port: number, share: string, path: string, newName: string) => invoke<void>("rename_remote_path", { ip, port, share, path, newName }),
  upload: (ip: string, port: number, share: string, remotePath: string, localPath: string) => invoke<void>("upload_to_remote", { ip, port, share, remotePath, localPath }),
  download: (ip: string, port: number, share: string, remotePath: string, localPath: string, transferId: string, filename: string, sourceDevice: string) =>
    invoke<void>("download_from_remote", { ip, port, share, remotePath, localPath, transferId, filename, sourceDevice }),
  cancelTransfer: (id: string) => invoke<void>("cancel_transfer", { id }),
  transferHistory: () => invoke<any[]>("get_transfer_history"),
  getSetting: (key: string) => invoke<string | null>("get_settings", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_settings", { key, value }),
  groups: () => invoke<any[]>("get_groups"),
  addGroup: (name: string, deviceIds: string) => invoke<any>("add_group", { name, deviceIds }),
  deleteGroup: (id: string) => invoke<void>("delete_group", { id }),
  syncLocalToRemote: (localPath: string, ip: string, port: number, share: string, remoteBase: string) => invoke<void>("sync_local_to_remote", { localPath, ip, port, share, remoteBase }),
  syncRemoteToLocal: (ip: string, port: number, share: string, remotePath: string, localBase: string) => invoke<void>("sync_remote_to_local", { ip, port, share, remotePath, localBase }),
  diagnostics: () => invoke<any>("network_diagnostics"),
  speedTest: (ip: string, port: number) => invoke<any>("speed_test", { ip, port }),
};

export { listen };
