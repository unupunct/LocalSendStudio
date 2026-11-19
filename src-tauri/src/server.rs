use crate::auth;
use crate::db::{SharedFolder, TransferRecord};
use crate::protocol::{self, Request, Response, CHUNK_SIZE, KIND_CHUNK, KIND_END, KIND_JSON, PROTOCOL_VERSION};
use crate::state::{AppState, VERSION};
use crate::util;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const IDLE_TIMEOUT: Duration = Duration::from_secs(120);
const IO_TIMEOUT: Duration = Duration::from_secs(60);
pub const PART_SUFFIX: &str = ".lsspart";

/// Bind the first free port starting at `start` (so a second instance does not fail silently).
pub fn bind(start: u16) -> std::io::Result<(std::net::TcpListener, u16)> {
    let mut last = None;
    for p in start..start.saturating_add(20) {
        match std::net::TcpListener::bind(("0.0.0.0", p)) {
            Ok(l) => {
                l.set_nonblocking(true)?;
                let port = l.local_addr()?.port();
                return Ok((l, port));
            }
            Err(e) => last = Some(e),
        }
    }
    Err(last.unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::AddrInUse, "no free port")))
}

pub async fn run_server(state: AppState, listener: std::net::TcpListener) {
    let listener = match TcpListener::from_std(listener) {
        Ok(l) => l,
        Err(e) => {
            log::error!("server: {}", e);
            return;
        }
    };
    log::info!("server listening on {:?}", listener.local_addr());
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let st = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_conn(st, stream, addr.ip().to_string()).await {
                        log::debug!("connection ended: {}", e);
                    }
                });
            }
            Err(e) => {
                log::warn!("accept failed: {}", e);
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

async fn handle_conn(state: AppState, mut stream: TcpStream, ip: String) -> anyhow::Result<()> {
    let _ = stream.set_nodelay(true);
    let mut allowed: Option<bool> = None;
    loop {
        let (kind, buf) = match tokio::time::timeout(IDLE_TIMEOUT, protocol::read_frame(&mut stream)).await {
            Ok(r) => r?,
            Err(_) => return Ok(()),
        };
        if kind != KIND_JSON {
            anyhow::bail!("unexpected frame");
        }
        let req: Request = serde_json::from_slice(&buf)?;
        if req.method != "info" {
            if allowed.is_none() {
                allowed = Some(auth::authorize(&state, req.from.as_ref(), &ip).await);
            }
            if allowed != Some(true) {
                protocol::write_json(&mut stream, &Response::err(&req.id, "not authorized")).await?;
                return Ok(());
            }
        }
        if let Some(resp) = dispatch(&state, &req, &mut stream).await? {
            protocol::write_json(&mut stream, &resp).await?;
        }
    }
}

fn param<'a>(req: &'a Request, key: &str) -> &'a str {
    req.params.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

#[derive(Clone, Copy, PartialEq)]
enum Need {
    Read,
    Upload,
    Manage,
}

fn find_share(state: &AppState, req: &Request, need: Need) -> Result<(SharedFolder, PathBuf), &'static str> {
    let share = param(req, "share");
    let folder = state
        .shared_folders
        .lock()
        .iter()
        .find(|f| f.enabled && f.name == share)
        .cloned()
        .ok_or("share not found")?;
    let ok = match need {
        Need::Read => folder.read || folder.manage,
        Need::Upload => folder.upload || folder.manage,
        Need::Manage => folder.manage,
    };
    if !ok {
        return Err(match need {
            Need::Read => "reading is not allowed on this folder",
            Need::Upload => "uploading is not allowed on this folder",
            Need::Manage => "managing files is not allowed on this folder",
        });
    }
    let target = util::safe_join(Path::new(&folder.path), param(req, "path")).ok_or("invalid path")?;
    Ok((folder, target))
}

fn peer_name(req: &Request) -> String {
    req.from.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "Unknown device".into())
}

async fn dispatch(state: &AppState, req: &Request, stream: &mut TcpStream) -> anyhow::Result<Option<Response>> {
    let id = req.id.as_str();
    match req.method.as_str() {
        "info" => Ok(Some(Response::ok(
            id,
            json!({
                "id": state.identity.id,
                "name": state.device_name(),
                "os": state.identity.os,
                "version": VERSION,
                "protocol": PROTOCOL_VERSION,
                "shared_folders": state.enabled_share_names(),
            }),
        ))),
        "list_dir" => {
            let (folder, target) = match find_share(state, req, Need::Read) {
                Ok(v) => v,
                Err(e) => return Ok(Some(Response::err(id, e))),
            };
            if !target.is_dir() {
                return Ok(Some(Response::err(id, "not a folder")));
            }
            let mut entries = Vec::new();
            for e in std::fs::read_dir(&target)?.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.ends_with(PART_SUFFIX) {
                    continue;
                }
                // `metadata` follows symlinks; skip anything that cannot be resolved.
                let Ok(md) = std::fs::metadata(e.path()) else { continue };
                let modified = md
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                entries.push(json!({ "name": name, "is_dir": md.is_dir(), "size": md.len(), "modified": modified }));
            }
            entries.sort_by(|a, b| {
                let ad = a["is_dir"].as_bool().unwrap_or(false);
                let bd = b["is_dir"].as_bool().unwrap_or(false);
                bd.cmp(&ad).then_with(|| {
                    a["name"].as_str().unwrap_or("").to_lowercase().cmp(&b["name"].as_str().unwrap_or("").to_lowercase())
                })
            });
            Ok(Some(Response::ok(
                id,
                json!({ "entries": entries, "can_upload": folder.upload || folder.manage, "can_manage": folder.manage }),
            )))
        }
        "download" => {
            let (_f, target) = match find_share(state, req, Need::Read) {
                Ok(v) => v,
                Err(e) => return Ok(Some(Response::err(id, e))),
            };
            if !target.is_file() {
                return Ok(Some(Response::err(id, "not a file")));
            }
            let offset = req.params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
            let mut file = File::open(&target).await?;
            let size = file.metadata().await?.len();
            if offset > size {
                return Ok(Some(Response::err(id, "offset beyond end of file")));
            }
            protocol::write_json(stream, &Response::ok(id, json!({ "size": size, "offset": offset }))).await?;
            let started = util::now_ms();
            let mut hasher = util::new_hasher();
            let mut buf = vec![0u8; CHUNK_SIZE];
            // Hash the part the client already has so the final digest covers the whole file.
            let mut skipped = 0u64;
            while skipped < offset {
                let want = ((offset - skipped) as usize).min(buf.len());
                let n = file.read(&mut buf[..want]).await?;
                if n == 0 {
                    break;
                }
                sha2::Digest::update(&mut hasher, &buf[..n]);
                skipped += n as u64;
            }
            loop {
                let n = file.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                sha2::Digest::update(&mut hasher, &buf[..n]);
                tokio::time::timeout(IO_TIMEOUT, protocol::write_frame(stream, KIND_CHUNK, &buf[..n])).await??;
            }
            let digest = util::hex(sha2::Digest::finalize(hasher));
            protocol::write_frame(stream, KIND_END, digest.as_bytes()).await?;
            tokio::io::AsyncWriteExt::flush(stream).await?;
            record(state, req, &target, size, started, "completed", "served");
            Ok(None)
        }
        "upload" => handle_upload(state, req, stream).await,
        "mkdir" => {
            let (_f, target) = match find_share(state, req, Need::Manage) {
                Ok(v) => v,
                Err(e) => return Ok(Some(Response::err(id, e))),
            };
            match std::fs::create_dir_all(&target) {
                Ok(_) => Ok(Some(Response::ok(id, json!({})))),
                Err(e) => Ok(Some(Response::err(id, &e.to_string()))),
            }
        }
        "delete" => {
            let (f, target) = match find_share(state, req, Need::Manage) {
                Ok(v) => v,
                Err(e) => return Ok(Some(Response::err(id, e))),
            };
            if target == Path::new(&f.path) || target.canonicalize().ok() == Path::new(&f.path).canonicalize().ok() {
                return Ok(Some(Response::err(id, "cannot delete the shared folder itself")));
            }
            let res = if target.is_dir() { std::fs::remove_dir_all(&target) } else { std::fs::remove_file(&target) };
            match res {
                Ok(_) => Ok(Some(Response::ok(id, json!({})))),
                Err(e) => Ok(Some(Response::err(id, &e.to_string()))),
            }
        }
        "rename" => {
            let (_f, target) = match find_share(state, req, Need::Manage) {
                Ok(v) => v,
                Err(e) => return Ok(Some(Response::err(id, e))),
            };
            let new_name = param(req, "new_name");
            if !util::valid_name(new_name) {
                return Ok(Some(Response::err(id, "invalid name")));
            }
            let Some(parent) = target.parent() else { return Ok(Some(Response::err(id, "invalid path"))) };
            let dest = parent.join(new_name);
            if dest.exists() {
                return Ok(Some(Response::err(id, "a file with that name already exists")));
            }
            match std::fs::rename(&target, &dest) {
                Ok(_) => Ok(Some(Response::ok(id, json!({})))),
                Err(e) => Ok(Some(Response::err(id, &e.to_string()))),
            }
        }
        "speedtest" => {
            let bytes = req.params.get("bytes").and_then(|v| v.as_u64()).unwrap_or(16 << 20).min(256 << 20);
            protocol::write_json(stream, &Response::ok(id, json!({ "bytes": bytes }))).await?;
            let chunk = vec![0u8; CHUNK_SIZE];
            let mut left = bytes;
            while left > 0 {
                let n = (left as usize).min(chunk.len());
                tokio::time::timeout(IO_TIMEOUT, protocol::write_frame(stream, KIND_CHUNK, &chunk[..n])).await??;
                left -= n as u64;
            }
            protocol::write_frame(stream, KIND_END, b"").await?;
            tokio::io::AsyncWriteExt::flush(stream).await?;
            Ok(None)
        }
        _ => Ok(Some(Response::err(id, "unknown method"))),
    }
}

async fn handle_upload(state: &AppState, req: &Request, stream: &mut TcpStream) -> anyhow::Result<Option<Response>> {
    let id = req.id.as_str();
    let (_f, requested) = match find_share(state, req, Need::Upload) {
        Ok(v) => v,
        Err(e) => return Ok(Some(Response::err(id, e))),
    };
    if requested.is_dir() && param(req, "path").trim_matches(|c| c == '/' || c == '\\').is_empty() {
        return Ok(Some(Response::err(id, "missing file name")));
    }
    let size = req.params.get("size").and_then(|v| v.as_u64());
    let want_sha = param(req, "sha256").to_ascii_lowercase();
    let Some(size) = size else { return Ok(Some(Response::err(id, "missing size"))) };
    // Never overwrite: an existing file gets a numbered name.
    let final_path = util::unique_path(&requested);
    if let Some(parent) = final_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Ok(Some(Response::err(id, &e.to_string())));
        }
    }
    let part = PathBuf::from(format!("{}{}", final_path.display(), PART_SUFFIX));
    let mut out = match File::create(&part).await {
        Ok(f) => f,
        Err(e) => return Ok(Some(Response::err(id, &e.to_string()))),
    };
    protocol::write_json(stream, &Response::ok(id, json!({ "ready": true }))).await?;

    let who = peer_name(req);
    let file_name = final_path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    (state.emit)("incoming", json!({ "from": who, "file": file_name, "size": size }));
    let started = util::now_ms();

    let result: anyhow::Result<()> = async {
        let mut hasher = util::new_hasher();
        let mut total = 0u64;
        loop {
            let (kind, buf) = tokio::time::timeout(IO_TIMEOUT, protocol::read_frame(stream)).await??;
            match kind {
                KIND_CHUNK => {
                    total += buf.len() as u64;
                    if total > size {
                        anyhow::bail!("sender sent more data than announced");
                    }
                    sha2::Digest::update(&mut hasher, &buf);
                    out.write_all(&buf).await?;
                }
                KIND_END => break,
                _ => anyhow::bail!("unexpected frame"),
            }
        }
        out.flush().await?;
        if total != size {
            anyhow::bail!("incomplete upload ({} of {} bytes)", total, size);
        }
        let got = util::hex(sha2::Digest::finalize(hasher));
        if !want_sha.is_empty() && got != want_sha {
            anyhow::bail!("checksum mismatch");
        }
        Ok(())
    }
    .await;
    drop(out);

    match result {
        Ok(()) => {
            if let Err(e) = std::fs::rename(&part, &final_path) {
                let _ = std::fs::remove_file(&part);
                return Ok(Some(Response::err(id, &e.to_string())));
            }
            record(state, req, &final_path, size, started, "completed", "received");
            (state.emit)("incoming-done", json!({ "from": who, "file": file_name, "status": "completed" }));
            let rel = final_path
                .strip_prefix(Path::new(&_f.path))
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or(file_name);
            Ok(Some(Response::ok(id, json!({ "path": rel, "size": size }))))
        }
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            record(state, req, &final_path, size, started, "failed", "received");
            (state.emit)("incoming-done", json!({ "from": who, "file": file_name, "status": "failed" }));
            // Best effort: tell the sender why, then close the connection.
            let _ = protocol::write_json(stream, &Response::err(id, &e.to_string())).await;
            Err(e)
        }
    }
}

fn record(state: &AppState, req: &Request, path: &Path, size: u64, started: i64, status: &str, direction: &str) {
    let who = peer_name(req);
    let me = state.device_name();
    let (source, destination) = if direction == "received" { (who, me) } else { (me, who) };
    let ended = util::now_ms();
    let secs = ((ended - started).max(1)) as f64 / 1000.0;
    state.db.insert_transfer(&TransferRecord {
        id: uuid::Uuid::new_v4().to_string(),
        filename: path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
        source,
        destination,
        size: size as i64,
        started,
        ended: Some(ended),
        speed: size as f64 / secs,
        status: status.into(),
        direction: direction.into(),
    });
    (state.emit)("history-updated", Value::Null);
}
