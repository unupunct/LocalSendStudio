use crate::db::{DeviceGroup, SharedFolder, TransferRecord, TrustedDevice};
use crate::protocol::Peer;
use crate::state::{AppState, DiscoveredDevice};
use crate::transfer::{self, CancelMap, Job, Kind};
use crate::{auth, client, discovery, util};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tauri::State;

type R<T> = Result<T, String>;

fn me(state: &AppState) -> Peer {
    Peer { id: state.identity.id.clone(), name: state.device_name() }
}

fn s<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

fn join_remote(dir: &str, name: &str) -> String {
    let dir = dir.trim_matches('/');
    if dir.is_empty() { name.to_string() } else { format!("{}/{}", dir, name) }
}

// ---- identity & settings -------------------------------------------------

#[tauri::command]
pub fn get_identity(state: State<AppState>) -> Value {
    json!({
        "id": state.identity.id,
        "name": state.device_name(),
        "os": state.identity.os,
        "version": crate::state::VERSION,
        "port": state.port(),
        "trust_mode": state.trust_mode(),
    })
}

#[tauri::command]
pub fn set_device_name(state: State<AppState>, name: String) -> R<()> {
    let name: String = name.trim().chars().take(60).collect();
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    state.db.set_setting("device_name", &name);
    *state.name.write() = name;
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<AppState>, key: String) -> Option<String> {
    state.db.get_setting(&key)
}

#[tauri::command]
pub fn set_settings(state: State<AppState>, key: String, value: String) {
    state.db.set_setting(&key, &value);
    // The Inbox always follows the download folder.
    if key == "download_dir" && Path::new(&value).is_dir() {
        let mut sf = state.shared_folders.lock();
        if let Some(f) = sf.iter_mut().find(|f| f.name == "Inbox") {
            f.path = value.clone();
            state.db.upsert_shared_folder(f);
        }
    }
}

#[tauri::command]
pub fn get_download_dir(state: State<AppState>) -> String {
    state
        .db
        .get_setting("download_dir")
        .filter(|d| !d.trim().is_empty())
        .unwrap_or_else(|| util::default_download_dir().to_string_lossy().to_string())
}

// ---- devices ---------------------------------------------------------------

#[tauri::command]
pub fn get_devices(state: State<AppState>) -> Vec<DiscoveredDevice> {
    let now = util::now_ms();
    let manual = discovery::manual_targets(&state);
    let mut map = state.devices.lock();
    map.retain(|_, d| {
        now - d.last_seen < discovery::FORGET_AFTER_MS || manual.iter().any(|m| m.starts_with(&format!("{}:", d.ip)))
    });
    let mut out: Vec<DiscoveredDevice> = map
        .values()
        .cloned()
        .map(|mut d| {
            d.status = if now - d.last_seen < discovery::ONLINE_WINDOW_MS { "online" } else { "offline" }.into();
            d.trusted = state.db.is_trusted(&d.id);
            d
        })
        .collect();
    out.sort_by(|a, b| (a.status != "online").cmp(&(b.status != "online")).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    out
}

#[tauri::command]
pub async fn add_device_by_ip(state: State<'_, AppState>, host: String) -> R<DiscoveredDevice> {
    let host = host.trim().to_string();
    let (h, port) = match host.rsplit_once(':') {
        Some((h, p)) if p.parse::<u16>().is_ok() => (h.to_string(), p.parse::<u16>().unwrap()),
        _ => (host.clone(), crate::protocol::DEFAULT_PORT),
    };
    if h.is_empty() || h.len() > 255 {
        return Err("Enter an IP address, for example 192.168.1.20".into());
    }
    let dev = discovery::probe(&state, &h, port).await.map_err(|e| format!("Could not reach {}:{} - {}", h, port, e))?;
    let mut list = discovery::manual_targets(&state);
    let key = format!("{}:{}", h, port);
    if !list.contains(&key) {
        list.push(key);
        state.db.set_setting("manual_devices", &serde_json::to_string(&list).unwrap());
    }
    Ok(dev)
}

#[tauri::command]
pub fn forget_device(state: State<AppState>, id: String) {
    let ip = state.devices.lock().remove(&id).map(|d| d.ip);
    if let Some(ip) = ip {
        let list: Vec<String> = discovery::manual_targets(&state).into_iter().filter(|m| !m.starts_with(&format!("{}:", ip))).collect();
        state.db.set_setting("manual_devices", &serde_json::to_string(&list).unwrap());
    }
}

#[tauri::command]
pub fn get_interfaces() -> Vec<Value> {
    util::list_interfaces().into_iter().map(|(n, ip, k)| json!({"name": n, "ip": ip, "kind": k})).collect()
}

// ---- trust -------------------------------------------------------------------

#[tauri::command]
pub fn respond_approval(state: State<AppState>, id: String, allow: bool, remember: bool) {
    auth::resolve(&state, &id, allow, remember);
}

#[tauri::command]
pub fn list_trusted(state: State<AppState>) -> Vec<TrustedDevice> {
    state.db.list_trusted()
}

#[tauri::command]
pub fn untrust_device(state: State<AppState>, id: String) {
    state.db.untrust(&id);
    state.session_allowed.lock().remove(&id);
}

// ---- shared folders ----------------------------------------------------------

#[tauri::command]
pub fn get_shared_folders(state: State<AppState>) -> Vec<SharedFolder> {
    state.shared_folders.lock().clone()
}

#[tauri::command]
pub fn add_shared_folder(state: State<AppState>, path: String, name: Option<String>) -> R<SharedFolder> {
    if !Path::new(&path).is_dir() {
        return Err("That folder does not exist".into());
    }
    let base = name
        .filter(|n| util::valid_name(n))
        .or_else(|| Path::new(&path).file_name().map(|s| s.to_string_lossy().to_string()).filter(|n| util::valid_name(n)))
        .unwrap_or_else(|| "Shared".into());
    let mut sf = state.shared_folders.lock();
    // Shares are addressed by name, so names must be unique.
    let mut nm = base.clone();
    let mut i = 2;
    while sf.iter().any(|f| f.name == nm) {
        nm = format!("{} ({})", base, i);
        i += 1;
    }
    // Safe defaults: others can look and download, not change anything.
    let f = SharedFolder { id: uuid::Uuid::new_v4().to_string(), name: nm, path, enabled: true, read: true, upload: false, manage: false };
    state.db.upsert_shared_folder(&f);
    sf.push(f.clone());
    Ok(f)
}

#[tauri::command]
pub fn remove_shared_folder(state: State<AppState>, id: String) {
    state.db.delete_shared_folder(&id);
    state.shared_folders.lock().retain(|f| f.id != id);
}

#[tauri::command]
pub fn toggle_shared_folder(state: State<AppState>, id: String, enabled: bool) {
    let mut sf = state.shared_folders.lock();
    if let Some(f) = sf.iter_mut().find(|f| f.id == id) {
        f.enabled = enabled;
        state.db.upsert_shared_folder(f);
    }
}

#[tauri::command]
pub fn set_folder_perms(state: State<AppState>, id: String, read: bool, upload: bool, manage: bool) {
    let mut sf = state.shared_folders.lock();
    if let Some(f) = sf.iter_mut().find(|f| f.id == id) {
        f.read = read;
        f.upload = upload;
        f.manage = manage;
        state.db.upsert_shared_folder(f);
    }
}

// ---- remote browsing -----------------------------------------------------------

#[tauri::command]
pub async fn list_remote_dir(state: State<'_, AppState>, ip: String, port: u16, share: String, path: String) -> R<Value> {
    client::list_dir(&me(&state), &ip, port, &share, &path).await.map_err(s)
}

#[tauri::command]
pub async fn remote_info(ip: String, port: u16) -> R<Value> {
    client::info(&ip, port).await.map_err(s)
}

#[tauri::command]
pub async fn make_remote_dir(state: State<'_, AppState>, ip: String, port: u16, share: String, path: String) -> R<()> {
    client::mkdir(&me(&state), &ip, port, &share, &path).await.map_err(s)
}

#[tauri::command]
pub async fn delete_remote_path(state: State<'_, AppState>, ip: String, port: u16, share: String, path: String) -> R<()> {
    client::delete(&me(&state), &ip, port, &share, &path).await.map_err(s)
}

#[tauri::command]
pub async fn rename_remote_path(state: State<'_, AppState>, ip: String, port: u16, share: String, path: String, new_name: String) -> R<()> {
    client::rename(&me(&state), &ip, port, &share, &path, &new_name).await.map_err(s)
}

// ---- transfers -------------------------------------------------------------------

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn start_download(
    state: State<AppState>,
    cancels: State<CancelMap>,
    ip: String,
    port: u16,
    share: String,
    remote_path: String,
    filename: String,
    device_name: String,
    local_dir: Option<String>,
) -> R<String> {
    if !util::valid_name(&filename) {
        return Err("Invalid file name".into());
    }
    let dir = local_dir
        .filter(|d| !d.is_empty())
        .or_else(|| state.db.get_setting("download_dir").filter(|d| !d.is_empty()))
        .map(PathBuf::from)
        .unwrap_or_else(util::default_download_dir);
    let id = uuid::Uuid::new_v4().to_string();
    transfer::spawn(
        state.inner().clone(),
        cancels.inner().clone(),
        Job { id: id.clone(), ip, port, share, device_name, filename, kind: Kind::Download { remote_path, local_dir: dir } },
    );
    Ok(id)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn start_upload(
    state: State<AppState>,
    cancels: State<CancelMap>,
    ip: String,
    port: u16,
    share: String,
    remote_dir: String,
    local_path: String,
    device_name: String,
) -> R<String> {
    let lp = PathBuf::from(&local_path);
    if !lp.is_file() {
        return Err("Pick a file to send".into());
    }
    let filename = lp.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let id = uuid::Uuid::new_v4().to_string();
    transfer::spawn(
        state.inner().clone(),
        cancels.inner().clone(),
        Job {
            id: id.clone(),
            ip,
            port,
            share,
            device_name,
            filename: filename.clone(),
            kind: Kind::Upload { remote_path: join_remote(&remote_dir, &filename), local_path: lp },
        },
    );
    Ok(id)
}

#[tauri::command]
pub fn cancel_transfer(cancels: State<CancelMap>, id: String) {
    if let Some(f) = cancels.lock().get(&id).cloned() {
        f.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

#[tauri::command]
pub fn get_transfer_history(state: State<AppState>) -> Vec<TransferRecord> {
    state.db.list_transfers()
}

#[tauri::command]
pub fn clear_transfer_history(state: State<AppState>) {
    state.db.clear_transfers();
}

// ---- groups ----------------------------------------------------------------------

#[tauri::command]
pub fn get_groups(state: State<AppState>) -> Vec<DeviceGroup> {
    state.db.list_groups()
}

#[tauri::command]
pub fn add_group(state: State<AppState>, name: String, device_ids: String) -> DeviceGroup {
    let g = DeviceGroup { id: uuid::Uuid::new_v4().to_string(), name, device_ids };
    state.db.upsert_group(&g);
    g
}

#[tauri::command]
pub fn delete_group(state: State<AppState>, id: String) {
    state.db.delete_group(&id);
}

// ---- diagnostics -----------------------------------------------------------------

#[tauri::command]
pub async fn network_diagnostics(state: State<'_, AppState>) -> R<Value> {
    let port = state.port();
    let self_reach = client::info("127.0.0.1", port).await.is_ok();
    let now = util::now_ms();
    let online = state.devices.lock().values().filter(|d| now - d.last_seen < discovery::ONLINE_WINDOW_MS).count();
    let interfaces: Vec<Value> = util::list_interfaces().into_iter().map(|(n, ip, k)| json!({"name": n, "ip": ip, "kind": k})).collect();
    let usable = interfaces.iter().filter(|i| i["kind"] != "virtual").count();
    Ok(json!({
        "port": port,
        "self_reachable": self_reach,
        "devices_online": online,
        "interfaces": interfaces,
        "usable_interfaces": usable,
        "shared_folders": state.shared_folders.lock().len(),
        "trust_mode": state.trust_mode(),
        "discovery_port": crate::protocol::DISCOVERY_PORT,
    }))
}

#[tauri::command]
pub async fn speed_test(state: State<'_, AppState>, ip: String, port: u16) -> R<Value> {
    let (bps, rtt) = client::speedtest(&me(&state), &ip, port, 32 << 20).await.map_err(s)?;
    Ok(json!({ "bytes_per_sec": bps, "rtt_ms": rtt }))
}

#[tauri::command]
pub fn get_local_storage(state: State<AppState>) -> Value {
    use fs4::available_space;
    let dir = PathBuf::from(get_download_dir(state));
    let probe = if dir.exists() { dir } else { dir.ancestors().find(|p| p.exists()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from(".")) };
    json!({
        "path": probe.to_string_lossy(),
        "available": available_space(&probe).unwrap_or(0),
        "total": fs4::total_space(&probe).unwrap_or(0),
    })
}

// ---- one-way sync ------------------------------------------------------------------

fn walk_remote<'a>(
    me: &'a Peer,
    ip: &'a str,
    port: u16,
    share: &'a str,
    dir: String,
    out: &'a mut Vec<(String, u64)>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let data = client::list_dir(me, ip, port, share, &dir).await?;
        for e in data.get("entries").and_then(|v| v.as_array()).cloned().unwrap_or_default() {
            let name = e.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let path = join_remote(&dir, name);
            if e.get("is_dir").and_then(|v| v.as_bool()).unwrap_or(false) {
                walk_remote(me, ip, port, share, path, out).await?;
            } else {
                out.push((path, e.get("size").and_then(|v| v.as_u64()).unwrap_or(0)));
            }
        }
        Ok(())
    })
}

/// Copy files missing on the remote (same name + size = skipped). Never deletes anything.
#[tauri::command]
pub async fn sync_local_to_remote(state: State<'_, AppState>, local_path: String, ip: String, port: u16, share: String, remote_base: String) -> R<Value> {
    let me = me(&state);
    let mut remote = Vec::new();
    if client::list_dir(&me, &ip, port, &share, &remote_base).await.is_ok() {
        walk_remote(&me, &ip, port, &share, remote_base.trim_matches('/').to_string(), &mut remote).await.map_err(s)?;
    }
    let have: std::collections::HashMap<String, u64> = remote.into_iter().collect();
    let (mut copied, mut skipped) = (0u32, 0u32);
    for entry in walkdir::WalkDir::new(&local_path).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let rel = entry.path().strip_prefix(&local_path).map_err(s)?.to_string_lossy().replace('\\', "/");
        let remote_path = join_remote(&remote_base, &rel);
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        if have.get(&remote_path) == Some(&size) {
            skipped += 1;
            continue;
        }
        client::upload(&me, &ip, port, &share, &remote_path, entry.path(), None, |_, _| {}).await.map_err(s)?;
        copied += 1;
    }
    Ok(json!({ "copied": copied, "skipped": skipped }))
}

#[tauri::command]
pub async fn sync_remote_to_local(state: State<'_, AppState>, ip: String, port: u16, share: String, remote_path: String, local_base: String) -> R<Value> {
    let me = me(&state);
    let mut files = Vec::new();
    let base = remote_path.trim_matches('/').to_string();
    walk_remote(&me, &ip, port, &share, base.clone(), &mut files).await.map_err(s)?;
    let (mut copied, mut skipped) = (0u32, 0u32);
    for (path, size) in files {
        let rel = path.strip_prefix(&base).unwrap_or(&path).trim_start_matches('/').to_string();
        let mut local = PathBuf::from(&local_base);
        for part in rel.split('/') {
            if !util::valid_name(part) {
                local = PathBuf::new();
                break;
            }
            local.push(part);
        }
        if local.as_os_str().is_empty() {
            continue;
        }
        if std::fs::metadata(&local).map(|m| m.len() == size).unwrap_or(false) {
            skipped += 1;
            continue;
        }
        client::download(&me, &ip, port, &share, &path, &local, None, |_, _| {}).await.map_err(s)?;
        copied += 1;
    }
    Ok(json!({ "copied": copied, "skipped": skipped }))
}

// ---- local file picking (used where the OS dialog cannot return real paths, i.e. Android) ----

#[tauri::command]
pub fn list_local_dir(path: String) -> R<Value> {
    // Roots are platform specific; an empty path means "start here".
    let dir = if path.is_empty() {
        if cfg!(target_os = "android") {
            PathBuf::from(std::env::var("EXTERNAL_STORAGE").unwrap_or_else(|_| "/storage/emulated/0".into()))
        } else {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
        }
    } else {
        PathBuf::from(&path)
    };
    let rd = std::fs::read_dir(&dir).map_err(|e| format!("Cannot open {}: {}", dir.display(), e))?;
    let mut entries: Vec<Value> = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let Ok(md) = std::fs::metadata(e.path()) else { continue };
        entries.push(json!({ "name": name, "is_dir": md.is_dir(), "size": md.len() }));
    }
    entries.sort_by(|a, b| {
        let (ad, bd) = (a["is_dir"].as_bool().unwrap_or(false), b["is_dir"].as_bool().unwrap_or(false));
        bd.cmp(&ad).then_with(|| a["name"].as_str().unwrap_or("").to_lowercase().cmp(&b["name"].as_str().unwrap_or("").to_lowercase()))
    });
    Ok(json!({
        "path": dir.to_string_lossy(),
        "parent": dir.parent().map(|p| p.to_string_lossy().to_string()),
        "entries": entries,
    }))
}
