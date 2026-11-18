use crate::protocol::{self, KIND_CHUNK, KIND_END, KIND_JSON, Request, Response};
use crate::util;
use serde_json::json;
use std::path::Path;
use tokio::net::TcpStream;

pub async fn connect(ip: &str, port: u16) -> anyhow::Result<TcpStream> {
    Ok(TcpStream::connect(format!("{}:{}", ip, port)).await?)
}

pub async fn info(ip: &str, port: u16) -> anyhow::Result<serde_json::Value> {
    let mut s = connect(ip, port).await?;
    let req = Request { id: uuid::Uuid::new_v4().to_string(), method: "info".into(), params: json!({}) };
    let data = serde_json::to_vec(&req)?;
    protocol::write_frame(&mut s, KIND_JSON, &data).await?;
    let (kind, buf) = protocol::read_frame(&mut s).await?;
    if kind != KIND_JSON { return Err(anyhow::anyhow!("bad frame")); }
    let resp: Response = serde_json::from_slice(&buf)?;
    if !resp.ok { return Err(anyhow::anyhow!(resp.error.unwrap_or_default())); }
    Ok(resp.data)
}

pub async fn list_dir(ip: &str, port: u16, share: &str, path: &str) -> anyhow::Result<serde_json::Value> {
    let mut s = connect(ip, port).await?;
    let req = Request { id: uuid::Uuid::new_v4().to_string(), method: "list_dir".into(), params: json!({"share": share, "path": path}) };
    let data = serde_json::to_vec(&req)?;
    protocol::write_frame(&mut s, KIND_JSON, &data).await?;
    let (kind, buf) = protocol::read_frame(&mut s).await?;
    if kind != KIND_JSON { return Err(anyhow::anyhow!("bad frame")); }
    let resp: Response = serde_json::from_slice(&buf)?;
    if !resp.ok { return Err(anyhow::anyhow!(resp.error.unwrap_or_default())); }
    Ok(resp.data)
}

pub async fn download<F>(ip: &str, port: u16, share: &str, remote_path: &str, local_path: &Path, cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>, mut progress: F) -> anyhow::Result<String>
where F: FnMut(u64, u64) + Send
{
    let mut s = connect(ip, port).await?;
    let req = Request { id: uuid::Uuid::new_v4().to_string(), method: "download".into(), params: json!({"share": share, "path": remote_path}) };
    let data = serde_json::to_vec(&req)?;
    protocol::write_frame(&mut s, KIND_JSON, &data).await?;
    // read metadata
    let (kind, buf) = protocol::read_frame(&mut s).await?;
    if kind != KIND_JSON { return Err(anyhow::anyhow!("bad meta")); }
    let resp: Response = serde_json::from_slice(&buf)?;
    if !resp.ok { return Err(anyhow::anyhow!(resp.error.unwrap_or_default())); }
    let size = resp.data.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
    let sha = resp.data.get("sha256").and_then(|v| v.as_str()).unwrap_or("").to_string();
    // receive chunks
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    if let Some(parent) = local_path.parent() { let _ = tokio::fs::create_dir_all(parent).await; }
    let mut out = File::create(local_path).await?;
    let mut total = 0u64;
    loop {
        if let Some(c) = &cancel { if c.load(std::sync::atomic::Ordering::Relaxed) { return Err(anyhow::anyhow!("cancelled")); } }
        let (k, buf) = protocol::read_frame(&mut s).await?;
        match k {
            KIND_CHUNK => {
                out.write_all(&buf).await?;
                total += buf.len() as u64;
                progress(total, size);
            }
            KIND_END => break,
            _ => {}
        }
    }
    out.flush().await?;
    Ok(sha)
}

pub async fn upload(ip: &str, port: u16, share: &str, remote_path: &str, local_path: &Path) -> anyhow::Result<()> {
    let mut s = connect(ip, port).await?;
    let size = tokio::fs::metadata(local_path).await?.len();
    let req = Request { id: uuid::Uuid::new_v4().to_string(), method: "upload".into(), params: json!({"share": share, "path": remote_path, "size": size}) };
    let data = serde_json::to_vec(&req)?;
    protocol::write_frame(&mut s, KIND_JSON, &data).await?;
    // ready
    let (kind, buf) = protocol::read_frame(&mut s).await?;
    if kind != KIND_JSON { return Err(anyhow::anyhow!("bad ready")); }
    let resp: Response = serde_json::from_slice(&buf)?;
    if !resp.ok { return Err(anyhow::anyhow!(resp.error.unwrap_or_default())); }
    // send chunks
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    let mut f = File::open(local_path).await?;
    let mut buf = vec![0u8; 65536];
    loop {
        let n = f.read(&mut buf).await?;
        if n == 0 { break; }
        protocol::write_frame(&mut s, KIND_CHUNK, &buf[..n]).await?;
    }
    protocol::write_frame(&mut s, KIND_END, &[]).await?;
    Ok(())
}

pub async fn mkdir(ip: &str, port: u16, share: &str, path: &str) -> anyhow::Result<()> {
    let mut s = connect(ip, port).await?;
    let req = Request { id: uuid::Uuid::new_v4().to_string(), method: "mkdir".into(), params: json!({"share": share, "path": path}) };
    let data = serde_json::to_vec(&req)?;
    protocol::write_frame(&mut s, KIND_JSON, &data).await?;
    let (kind, buf) = protocol::read_frame(&mut s).await?;
    if kind != KIND_JSON { return Err(anyhow::anyhow!("bad frame")); }
    let resp: Response = serde_json::from_slice(&buf)?;
    if !resp.ok { return Err(anyhow::anyhow!(resp.error.unwrap_or_default())); }
    Ok(())
}

pub async fn delete(ip: &str, port: u16, share: &str, path: &str) -> anyhow::Result<()> {
    let mut s = connect(ip, port).await?;
    let req = Request { id: uuid::Uuid::new_v4().to_string(), method: "delete".into(), params: json!({"share": share, "path": path}) };
    let data = serde_json::to_vec(&req)?;
    protocol::write_frame(&mut s, KIND_JSON, &data).await?;
    let (kind, buf) = protocol::read_frame(&mut s).await?;
    if kind != KIND_JSON { return Err(anyhow::anyhow!("bad frame")); }
    let resp: Response = serde_json::from_slice(&buf)?;
    if !resp.ok { return Err(anyhow::anyhow!(resp.error.unwrap_or_default())); }
    Ok(())
}

pub async fn rename(ip: &str, port: u16, share: &str, path: &str, new_name: &str) -> anyhow::Result<()> {
    let mut s = connect(ip, port).await?;
    let req = Request { id: uuid::Uuid::new_v4().to_string(), method: "rename".into(), params: json!({"share": share, "path": path, "new_name": new_name}) };
    let data = serde_json::to_vec(&req)?;
    protocol::write_frame(&mut s, KIND_JSON, &data).await?;
    let (kind, buf) = protocol::read_frame(&mut s).await?;
    if kind != KIND_JSON { return Err(anyhow::anyhow!("bad frame")); }
    let resp: Response = serde_json::from_slice(&buf)?;
    if !resp.ok { return Err(anyhow::anyhow!(resp.error.unwrap_or_default())); }
    Ok(())
}
