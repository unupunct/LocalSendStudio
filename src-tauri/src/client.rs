use crate::protocol::{self, Peer, Request, Response, CHUNK_SIZE, KIND_CHUNK, KIND_END};
use crate::server::PART_SUFFIX;
use crate::util;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// The first reply may wait for the other person to tap "Allow" (45 s on their side).
const FIRST_REPLY_TIMEOUT: Duration = Duration::from_secs(60);
const IO_TIMEOUT: Duration = Duration::from_secs(60);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

pub type Cancel = Option<Arc<AtomicBool>>;

fn cancelled(c: &Cancel) -> bool {
    c.as_ref().map(|f| f.load(Ordering::Relaxed)).unwrap_or(false)
}

pub async fn connect(ip: &str, port: u16) -> anyhow::Result<TcpStream> {
    let s = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect((ip, port)))
        .await
        .map_err(|_| anyhow::anyhow!("could not reach {}:{} (timed out)", ip, port))??;
    let _ = s.set_nodelay(true);
    Ok(s)
}

async fn send_request(s: &mut TcpStream, me: Option<&Peer>, method: &str, params: Value) -> anyhow::Result<()> {
    let req = Request { id: uuid::Uuid::new_v4().to_string(), method: method.into(), params, from: me.cloned() };
    protocol::write_json(s, &req).await
}

async fn read_response(s: &mut TcpStream, first: bool) -> anyhow::Result<Value> {
    let t = if first { FIRST_REPLY_TIMEOUT } else { IO_TIMEOUT };
    let resp: Response = tokio::time::timeout(t, protocol::read_json(s))
        .await
        .map_err(|_| anyhow::anyhow!("the other device did not answer"))??;
    if !resp.ok {
        anyhow::bail!(resp.error.unwrap_or_else(|| "request failed".into()));
    }
    Ok(resp.data)
}

pub async fn call(me: Option<&Peer>, ip: &str, port: u16, method: &str, params: Value) -> anyhow::Result<Value> {
    let mut s = connect(ip, port).await?;
    send_request(&mut s, me, method, params).await?;
    read_response(&mut s, true).await
}

pub async fn info(ip: &str, port: u16) -> anyhow::Result<Value> {
    call(None, ip, port, "info", json!({})).await
}

pub async fn list_dir(me: &Peer, ip: &str, port: u16, share: &str, path: &str) -> anyhow::Result<Value> {
    call(Some(me), ip, port, "list_dir", json!({"share": share, "path": path})).await
}

pub async fn mkdir(me: &Peer, ip: &str, port: u16, share: &str, path: &str) -> anyhow::Result<()> {
    call(Some(me), ip, port, "mkdir", json!({"share": share, "path": path})).await.map(|_| ())
}

pub async fn delete(me: &Peer, ip: &str, port: u16, share: &str, path: &str) -> anyhow::Result<()> {
    call(Some(me), ip, port, "delete", json!({"share": share, "path": path})).await.map(|_| ())
}

pub async fn rename(me: &Peer, ip: &str, port: u16, share: &str, path: &str, new_name: &str) -> anyhow::Result<()> {
    call(Some(me), ip, port, "rename", json!({"share": share, "path": path, "new_name": new_name}))
        .await
        .map(|_| ())
}

/// Download `remote_path` to `local_path`. Data goes to `<local_path>.lsspart` first and is
/// renamed only after the SHA-256 matches. An interrupted `.lsspart` is resumed.
/// Returns (final path, size).
#[allow(clippy::too_many_arguments)]
pub async fn download<F>(
    me: &Peer,
    ip: &str,
    port: u16,
    share: &str,
    remote_path: &str,
    local_path: &Path,
    cancel: Cancel,
    mut progress: F,
) -> anyhow::Result<(PathBuf, u64)>
where
    F: FnMut(u64, u64) + Send,
{
    if let Some(parent) = local_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let part = PathBuf::from(format!("{}{}", local_path.display(), PART_SUFFIX));
    let mut attempt = 0;
    loop {
        attempt += 1;
        let existing = tokio::fs::metadata(&part).await.map(|m| m.len()).unwrap_or(0);
        match download_once(me, ip, port, share, remote_path, &part, existing, &cancel, &mut progress).await {
            Ok(size) => {
                let final_path = util::unique_path(local_path);
                tokio::fs::rename(&part, &final_path).await?;
                return Ok((final_path, size));
            }
            Err(DlError::BadHash) if existing > 0 && attempt == 1 => {
                // The partial file was stale (remote file changed): start over once.
                let _ = tokio::fs::remove_file(&part).await;
            }
            Err(DlError::BadHash) => {
                let _ = tokio::fs::remove_file(&part).await;
                anyhow::bail!("checksum mismatch - the file was corrupted in transit");
            }
            Err(DlError::Other(e)) => return Err(e),
        }
    }
}

enum DlError {
    BadHash,
    Other(anyhow::Error),
}
impl<E: Into<anyhow::Error>> From<E> for DlError {
    fn from(e: E) -> Self {
        DlError::Other(e.into())
    }
}

#[allow(clippy::too_many_arguments)]
async fn download_once<F>(
    me: &Peer,
    ip: &str,
    port: u16,
    share: &str,
    remote_path: &str,
    part: &Path,
    existing: u64,
    cancel: &Cancel,
    progress: &mut F,
) -> Result<u64, DlError>
where
    F: FnMut(u64, u64) + Send,
{
    let mut s = connect(ip, port).await?;
    send_request(&mut s, Some(me), "download", json!({"share": share, "path": remote_path, "offset": existing})).await?;
    let meta = match read_response(&mut s, true).await {
        Ok(m) => m,
        // Offset past the end means our partial file is stale.
        Err(e) if existing > 0 && e.to_string().contains("offset") => return Err(DlError::BadHash),
        Err(e) => return Err(e.into()),
    };
    let size = meta.get("size").and_then(|v| v.as_u64()).unwrap_or(0);

    let mut hasher = util::new_hasher();
    let mut out;
    if existing > 0 {
        // Re-hash what we already have.
        let mut f = File::open(part).await?;
        let mut buf = vec![0u8; CHUNK_SIZE];
        let mut left = existing;
        while left > 0 {
            let n = f.read(&mut buf[..(left as usize).min(CHUNK_SIZE)]).await?;
            if n == 0 {
                break;
            }
            sha2::Digest::update(&mut hasher, &buf[..n]);
            left -= n as u64;
        }
        out = tokio::fs::OpenOptions::new().append(true).open(part).await?;
    } else {
        out = File::create(part).await?;
    }
    let mut total = existing;
    progress(total, size);
    let digest = loop {
        if cancelled(cancel) {
            let _ = out.flush().await;
            return Err(anyhow::anyhow!("cancelled").into());
        }
        let (k, buf) = tokio::time::timeout(IO_TIMEOUT, protocol::read_frame(&mut s))
            .await
            .map_err(|_| anyhow::anyhow!("connection stalled"))??;
        match k {
            KIND_CHUNK => {
                sha2::Digest::update(&mut hasher, &buf);
                out.write_all(&buf).await?;
                total += buf.len() as u64;
                progress(total, size);
            }
            KIND_END => break String::from_utf8_lossy(&buf).to_string(),
            _ => return Err(anyhow::anyhow!("unexpected frame").into()),
        }
    };
    out.flush().await?;
    drop(out);
    if total != size {
        return Err(anyhow::anyhow!("incomplete download ({} of {} bytes)", total, size).into());
    }
    if util::hex(sha2::Digest::finalize(hasher)) != digest {
        return Err(DlError::BadHash);
    }
    Ok(size)
}

/// Upload `local_path` to `remote_path` on the share. Returns the path the receiver stored it
/// under (it adds " (1)" instead of overwriting). The receiver verifies size and SHA-256.
#[allow(clippy::too_many_arguments)]
pub async fn upload<F>(
    me: &Peer,
    ip: &str,
    port: u16,
    share: &str,
    remote_path: &str,
    local_path: &Path,
    cancel: Cancel,
    mut progress: F,
) -> anyhow::Result<(String, u64)>
where
    F: FnMut(u64, u64) + Send,
{
    let size = tokio::fs::metadata(local_path).await?.len();
    let lp = local_path.to_path_buf();
    let sha = tokio::task::spawn_blocking(move || util::sha256_file(&lp)).await??;
    let mut s = connect(ip, port).await?;
    send_request(&mut s, Some(me), "upload", json!({"share": share, "path": remote_path, "size": size, "sha256": sha})).await?;
    read_response(&mut s, true).await?;
    let mut f = File::open(local_path).await?;
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut sent = 0u64;
    progress(0, size);
    loop {
        if cancelled(&cancel) {
            anyhow::bail!("cancelled");
        }
        let n = f.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        tokio::time::timeout(IO_TIMEOUT, protocol::write_frame(&mut s, KIND_CHUNK, &buf[..n]))
            .await
            .map_err(|_| anyhow::anyhow!("connection stalled"))??;
        sent += n as u64;
        progress(sent, size);
    }
    protocol::write_frame(&mut s, KIND_END, b"").await?;
    s.flush().await?;
    let data = read_response(&mut s, false).await?;
    let stored = data.get("path").and_then(|v| v.as_str()).unwrap_or(remote_path).to_string();
    Ok((stored, size))
}

/// Download `bytes` of zeros and report (bytes/sec, round trip ms).
pub async fn speedtest(me: &Peer, ip: &str, port: u16, bytes: u64) -> anyhow::Result<(f64, u128)> {
    let t0 = std::time::Instant::now();
    let mut s = connect(ip, port).await?;
    send_request(&mut s, Some(me), "speedtest", json!({ "bytes": bytes })).await?;
    read_response(&mut s, true).await?;
    let rtt = t0.elapsed().as_millis();
    let t1 = std::time::Instant::now();
    let mut total = 0u64;
    loop {
        let (k, buf) = tokio::time::timeout(IO_TIMEOUT, protocol::read_frame(&mut s))
            .await
            .map_err(|_| anyhow::anyhow!("connection stalled"))??;
        match k {
            KIND_CHUNK => total += buf.len() as u64,
            KIND_END => break,
            _ => anyhow::bail!("unexpected frame"),
        }
    }
    Ok((total as f64 / t1.elapsed().as_secs_f64().max(0.001), rtt))
}
