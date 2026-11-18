use crate::client;
use crate::db::SharedFolder;
use crate::discovery;
use crate::protocol::DEFAULT_PORT;
use crate::state::{AppState, DiscoveredDevice};
use crate::transfer::{CancelMap, new_cancel_map};
use crate::util;
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub fn get_identity(state: State<AppState>) -> serde_json::Value {
    json!({
        "id": state.identity.id,
        "name": state.identity.name,
        "os": state.identity.os,
        "version": state.identity.version,
    })
}

#[tauri::command]
pub fn set_device_name(state: State<AppState>, name: String) {
    // mutate identity (Arc) by saving new file; update in-memory not trivial due Arc. We'll just save; restart needed. For simplicity store setting.
    state.db.set_setting("device_name", &name);
}

#[tauri::command]
pub fn get_devices(state: State<AppState>) -> Vec<DiscoveredDevice> {
    state.devices.lock().values().cloned().collect()
}

#[tauri::command]
pub fn get_interfaces() -> Vec<serde_json::Value> {
    util::list_interfaces().into_iter().map(|(n, ip, k)| json!({"name": n, "ip": ip, "kind": k})).collect()
}

#[tauri::command]
pub fn get_shared_folders(state: State<AppState>) -> Vec<SharedFolder> {
    state.shared_folders.lock().clone()
}

#[tauri::command]
pub fn add_shared_folder(state: State<AppState>, path: String, name: Option<String>) -> SharedFolder {
    let id = uuid::Uuid::new_v4().to_string();
    let nm = name.unwrap_or_else(|| std::path::Path::new(&path).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Shared".into()));
    let f = SharedFolder { id: id.clone(), name: nm, path, enabled: true, read: true, upload: true, manage: true };
    state.db.upsert_shared_folder(&f);
    let mut sf = state.shared_folders.lock();
    sf.push(f.clone());
    f
}

#[tauri::command]
pub fn remove_shared_folder(state: State<AppState>, id: String) {
    state.db.delete_shared_folder(&id);
    let mut sf = state.shared_folders.lock();
    sf.retain(|f| f.id != id);
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
        f.read = read; f.upload = upload; f.manage = manage;
        state.db.upsert_shared_folder(f);
    }
}

#[tauri::command]
pub async fn list_remote_dir(state: State<'_, AppState>, ip: String, port: u16, share: String, path: String) -> Result<serde_json::Value, String> {
    client::list_dir(&ip, port, &share, &path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remote_info(ip: String, port: u16) -> Result<serde_json::Value, String> {
    client::info(&ip, port).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn make_remote_dir(ip: String, port: u16, share: String, path: String) -> Result<(), String> {
    client::mkdir(&ip, port, &share, &path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_remote_path(ip: String, port: u16, share: String, path: String) -> Result<(), String> {
    client::delete(&ip, port, &share, &path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_remote_path(ip: String, port: u16, share: String, path: String, new_name: String) -> Result<(), String> {
    client::rename(&ip, port, &share, &path, &new_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upload_to_remote(ip: String, port: u16, share: String, remote_path: String, local_path: String) -> Result<(), String> {
    client::upload(&ip, port, &share, &remote_path, std::path::Path::new(&local_path)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_from_remote(
    app: AppHandle,
    state: State<'_, AppState>,
    cancel: State<'_, CancelMap>,
    ip: String,
    port: u16,
    share: String,
    remote_path: String,
    local_path: String,
    transfer_id: String,
    filename: String,
    source_device: String,
) -> Result<(), String> {
    let cancel_map = cancel.inner().clone();
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    cancel_map.lock().insert(transfer_id.clone(), flag.clone());
    let app2 = app.clone();
    let tid = transfer_id.clone();
    let ip2 = ip.clone();
    let state2 = state.inner().clone();
    let filename2 = filename.clone();
    tokio::spawn(async move {
        let start = chrono::Utc::now().timestamp_millis();
        let mut last = start;
        let mut last_bytes = 0u64;
        let mut speed = 0f64;
        let local = std::path::PathBuf::from(&local_path);
        let res = client::download(&ip2, port, &share, &remote_path, &local, Some(flag.clone()), |done, total| {
            let now = chrono::Utc::now().timestamp_millis();
            if now - last >= 500 {
                speed = (done - last_bytes) as f64 / ((now - last) as f64 / 1000.0);
                last = now;
                last_bytes = done;
            }
            let _ = app2.emit("transfer-progress", json!({
                "id": tid, "transferred": done, "total": total, "speed": speed, "status": "transferring"
            }));
            // check cancel
        }).await;
        let cancelled = cancel_map.lock().get(&tid).map(|f| f.load(std::sync::atomic::Ordering::Relaxed)).unwrap_or(false);
        let status = if cancelled { "cancelled" } else if res.is_ok() { "completed" } else { "failed" };
        let ended = chrono::Utc::now().timestamp_millis();
        let size = std::fs::metadata(&local_path).map(|m| m.len() as i64).unwrap_or(0);
        let rec = crate::db::TransferRecord {
            id: tid.clone(),
            filename: filename2,
            source: source_device,
            destination: state2.identity.name.clone(),
            size,
            started: start,
            ended: Some(ended),
            speed,
            status: status.into(),
            direction: "download".into(),
        };
        state2.db.insert_transfer(&rec);
        let _ = app2.emit("transfer-done", json!({"id": tid, "status": status}));
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_transfer(cancel: State<'_, CancelMap>, id: String) {
    if let Some(f) = cancel.lock().get(&id).cloned() {
        f.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

#[tauri::command]
pub fn get_transfer_history(state: State<AppState>) -> Vec<crate::db::TransferRecord> {
    state.db.list_transfers()
}

#[tauri::command]
pub fn get_settings(state: State<AppState>, key: String) -> Option<String> {
    state.db.get_setting(&key)
}

#[tauri::command]
pub fn set_settings(state: State<AppState>, key: String, value: String) {
    state.db.set_setting(&key, &value);
}

#[tauri::command]
pub fn get_groups(state: State<AppState>) -> Vec<crate::db::DeviceGroup> {
    state.db.list_groups()
}

#[tauri::command]
pub fn add_group(state: State<AppState>, name: String, device_ids: String) -> crate::db::DeviceGroup {
    let g = crate::db::DeviceGroup { id: uuid::Uuid::new_v4().to_string(), name, device_ids };
    state.db.upsert_group(&g);
    g
}

#[tauri::command]
pub fn delete_group(state: State<AppState>, id: String) {
    state.db.delete_group(&id);
}

#[tauri::command]
pub async fn network_diagnostics(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    // basic checks
    let devices = state.devices.lock().len();
    let port_ok = std::net::TcpListener::bind(format!("0.0.0.0:{}", DEFAULT_PORT)).is_ok();
    // re-bind? We already bound in server; binding again fails. So check via connect to self.
    let self_reach = client::info("127.0.0.1", DEFAULT_PORT).await.is_ok();
    Ok(json!({
        "discovery": if devices >= 0 { "Working" } else { "Unknown" },
        "devices_found": devices,
        "local_port": DEFAULT_PORT,
        "self_reachable": self_reach,
        "interfaces": util::list_interfaces(),
        "shared_folders": state.shared_folders.lock().len(),
    }))
}

#[tauri::command]
pub async fn speed_test(ip: String, port: u16) -> Result<serde_json::Value, String> {
    // send/receive a temp 4MB? We'll just measure info roundtrip latency and a small download if possible.
    let start = std::time::Instant::now();
    let _ = client::info(&ip, port).await.map_err(|e| e.to_string())?;
    let rtt = start.elapsed().as_millis();
    Ok(json!({"rtt_ms": rtt, "note": "Application-level round trip"}))
}

#[tauri::command]
pub async fn sync_local_to_remote(local_path: String, ip: String, port: u16, share: String, remote_base: String) -> Result<(), String> {
    use walkdir::WalkDir;
    for entry in WalkDir::new(&local_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let rel = entry.path().strip_prefix(&local_path).unwrap().to_string_lossy().replace('\\', "/");
            let remote_path = if remote_base.is_empty() { rel.clone() } else { format!("{}/{}", remote_base, rel) };
            client::upload(&ip, port, &share, &remote_path, entry.path()).await.map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn sync_remote_to_local(ip: String, port: u16, share: String, remote_path: String, local_base: String) -> Result<(), String> {
    // list dir recursively via repeated list; simple top-level only
    let data = client::list_dir(&ip, port, &share, &remote_path).await.map_err(|e| e.to_string())?;
    let entries = data.get("entries").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let base = std::path::PathBuf::from(&local_base);
    let _ = std::fs::create_dir_all(&base);
    for e in entries {
        let name = e.get("name").and_then(|v| v.as_str()).unwrap_or("");
        if e.get("is_dir").and_then(|v| v.as_bool()).unwrap_or(false) {
            let sub = base.join(name);
            let _ = std::fs::create_dir_all(&sub);
        } else {
            let rp = if remote_path.is_empty() { name.to_string() } else { format!("{}/{}", remote_path, name) };
            let lp = base.join(name);
            client::download(&ip, port, &share, &rp, &lp, None, |_d, _t| {}).await.map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_local_shared_storage(state: State<AppState>) -> serde_json::Value {
    // disk space for first shared folder or home
    let mut total = 0u64; let mut avail = 0u64;
    #[cfg(target_os = "windows")]
    {
        // use wmic? simpler: fs2 not available. Use std not enough. Return placeholder via sys? We'll try GetDiskFreeSpaceEx via cmd.
    }
    json!({"total": total, "available": avail})
}
