use crate::protocol::{self, KIND_CHUNK, KIND_END, KIND_JSON, Request, Response};
use crate::state::AppState;
use crate::util;
use serde_json::json;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn run_server(state: AppState, port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => { log::error!("bind fail: {}", e); return; }
    };
    log::info!("server listening {}", addr);
    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let st = state.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_conn(st, stream).await {
                    log::debug!("conn err: {}", e);
                }
            });
        }
    }
}

async fn handle_conn(state: AppState, mut stream: TcpStream) -> anyhow::Result<()> {
    loop {
        let (kind, buf) = protocol::read_frame(&mut stream).await?;
        if kind != KIND_JSON { continue; }
        let req: Request = serde_json::from_slice(&buf)?;
        let resp = handle_request(&state, &req, &mut stream).await?;
        // for methods that stream, handle_request already wrote frames; but we still may send response
        if let Some(r) = resp {
            let data = serde_json::to_vec(&r)?;
            protocol::write_frame(&mut stream, KIND_JSON, &data).await?;
        }
    }
}

async fn handle_request(state: &AppState, req: &Request, stream: &mut TcpStream) -> anyhow::Result<Option<Response>> {
    match req.method.as_str() {
        "info" => {
            let folders = state.shared_folders.lock().clone();
            let names: Vec<String> = folders.iter().filter(|f| f.enabled).map(|f| f.name.clone()).collect();
            Ok(Some(Response { id: req.id.clone(), ok: true, data: json!({
                "id": state.identity.id,
                "name": state.identity.name,
                "os": state.identity.os,
                "version": state.identity.version,
                "shared_folders": names,
            }), error: None }))
        }
        "list_dir" => {
            let share = req.params.get("share").and_then(|v| v.as_str()).unwrap_or("");
            let path = req.params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let folders = state.shared_folders.lock().clone();
            let folder = folders.iter().find(|f| f.enabled && f.name == share).cloned();
            if folder.is_none() {
                return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("share not found".into()) }));
            }
            let folder = folder.unwrap();
            let root = Path::new(&folder.path);
            let target = match util::safe_join(root, path) {
                Some(p) => p,
                None => return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("invalid path".into()) })),
            };
            if !target.exists() {
                return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("not found".into()) }));
            }
            let mut entries = Vec::new();
            if target.is_dir() {
                if let Ok(rd) = std::fs::read_dir(&target) {
                    for e in rd.flatten() {
                        let md = e.metadata().ok();
                        let is_dir = md.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                        let size = md.as_ref().map(|m| m.len()).unwrap_or(0);
                        let modified = md.as_ref().and_then(|m| m.modified().ok())
                            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64).unwrap_or(0);
                        entries.push(json!({
                            "name": e.file_name().to_string_lossy(),
                            "is_dir": is_dir,
                            "size": size,
                            "modified": modified,
                        }));
                    }
                }
            }
            Ok(Some(Response { id: req.id.clone(), ok: true, data: json!({ "entries": entries }), error: None }))
        }
        "download" => {
            let share = req.params.get("share").and_then(|v| v.as_str()).unwrap_or("");
            let path = req.params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let folders = state.shared_folders.lock().clone();
            let folder = folders.iter().find(|f| f.enabled && f.name == share).cloned();
            if folder.is_none() {
                return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("share not found".into()) }));
            }
            let folder = folder.unwrap();
            let root = Path::new(&folder.path);
            let target = match util::safe_join(root, path) {
                Some(p) => p, None => return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("invalid path".into()) })),
            };
            if !target.is_file() {
                return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("not a file".into()) }));
            }
            let size = std::fs::metadata(&target)?.len();
            let sha = util::sha256_file(&target).unwrap_or_default();
            // send metadata response first
            let meta = Response { id: req.id.clone(), ok: true, data: json!({ "size": size, "sha256": sha }), error: None };
            let data = serde_json::to_vec(&meta)?;
            protocol::write_frame(stream, KIND_JSON, &data).await?;
            // stream chunks
            use tokio::fs::File;
            use tokio::io::AsyncReadExt;
            let mut f = File::open(&target).await?;
            let mut buf = vec![0u8; 65536];
            loop {
                let n = f.read(&mut buf).await?;
                if n == 0 { break; }
                protocol::write_frame(stream, KIND_CHUNK, &buf[..n]).await?;
            }
            protocol::write_frame(stream, KIND_END, &[]).await?;
            Ok(None) // already responded
        }
        "upload" => {
            // params: share, path, size
            let share = req.params.get("share").and_then(|v| v.as_str()).unwrap_or("");
            let path = req.params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let size = req.params.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
            let folders = state.shared_folders.lock().clone();
            let folder = folders.iter().find(|f| f.enabled && f.name == share).cloned();
            if folder.is_none() {
                return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("share not found".into()) }));
            }
            let f = folder.unwrap();
            if !f.upload && !f.manage {
                return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("upload not allowed".into()) }));
            }
            let root = Path::new(&f.path);
            let target = match util::safe_join(root, path) {
                Some(p) => p, None => return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("invalid path".into()) })),
            };
            if let Some(parent) = target.parent() { let _ = std::fs::create_dir_all(parent); }
            // ready response
            let ready = Response { id: req.id.clone(), ok: true, data: json!({ "ready": true }), error: None };
            let data = serde_json::to_vec(&ready)?;
            protocol::write_frame(stream, KIND_JSON, &data).await?;
            // receive chunks
            use tokio::fs::File;
            use tokio::io::AsyncWriteExt;
            let mut out = File::create(&target).await?;
            let mut total = 0u64;
            loop {
                let (kind, buf) = protocol::read_frame(stream).await?;
                match kind {
                    KIND_CHUNK => {
                        out.write_all(&buf).await?;
                        total += buf.len() as u64;
                    }
                    KIND_END => break,
                    _ => {}
                }
                if size > 0 && total >= size { 
                    // wait for END anyway; but break if exact
                }
            }
            out.flush().await?;
            Ok(None)
        }
        "mkdir" => {
            let share = req.params.get("share").and_then(|v| v.as_str()).unwrap_or("");
            let path = req.params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let folders = state.shared_folders.lock().clone();
            let folder = folders.iter().find(|f| f.enabled && f.name == share).cloned();
            if folder.is_none() { return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("share not found".into()) })); }
            let f = folder.unwrap();
            if !f.manage { return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("manage not allowed".into()) })); }
            let root = Path::new(&f.path);
            let target = match util::safe_join(root, path) { Some(p)=>p, None=>return Ok(Some(Response{id:req.id.clone(),ok:false,data:json!(null),error:Some("invalid path".into())})) };
            let _ = std::fs::create_dir_all(&target);
            Ok(Some(Response { id: req.id.clone(), ok: true, data: json!({}), error: None }))
        }
        "delete" => {
            let share = req.params.get("share").and_then(|v| v.as_str()).unwrap_or("");
            let path = req.params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let folders = state.shared_folders.lock().clone();
            let folder = folders.iter().find(|f| f.enabled && f.name == share).cloned();
            if folder.is_none() { return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("share not found".into()) })); }
            let f = folder.unwrap();
            if !f.manage { return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("manage not allowed".into()) })); }
            let root = Path::new(&f.path);
            let target = match util::safe_join(root, path) { Some(p)=>p, None=>return Ok(Some(Response{id:req.id.clone(),ok:false,data:json!(null),error:Some("invalid path".into())})) };
            if target.is_dir() { let _ = std::fs::remove_dir_all(&target); } else { let _ = std::fs::remove_file(&target); }
            Ok(Some(Response { id: req.id.clone(), ok: true, data: json!({}), error: None }))
        }
        "rename" => {
            let share = req.params.get("share").and_then(|v| v.as_str()).unwrap_or("");
            let path = req.params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let new_name = req.params.get("new_name").and_then(|v| v.as_str()).unwrap_or("");
            let folders = state.shared_folders.lock().clone();
            let folder = folders.iter().find(|f| f.enabled && f.name == share).cloned();
            if folder.is_none() { return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("share not found".into()) })); }
            let f = folder.unwrap();
            if !f.manage { return Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("manage not allowed".into()) })); }
            let root = Path::new(&f.path);
            let target = match util::safe_join(root, path) { Some(p)=>p, None=>return Ok(Some(Response{id:req.id.clone(),ok:false,data:json!(null),error:Some("invalid path".into())})) };
            if let Some(parent)=target.parent() {
                let newp = parent.join(new_name);
                let _ = std::fs::rename(&target, &newp);
            }
            Ok(Some(Response { id: req.id.clone(), ok: true, data: json!({}), error: None }))
        }
        _ => Ok(Some(Response { id: req.id.clone(), ok: false, data: json!(null), error: Some("unknown method".into()) })),
    }
}
