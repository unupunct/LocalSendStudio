//! Wire protocol: length-prefixed frames over TCP. See docs/PROTOCOL.md.
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const DEFAULT_PORT: u16 = 47821;
pub const DISCOVERY_PORT: u16 = 47822;
pub const PROTOCOL_VERSION: u32 = 2;

pub const KIND_JSON: u8 = 1;
pub const KIND_CHUNK: u8 = 2;
pub const KIND_END: u8 = 3;

/// Upper bound for one frame payload. Without a cap a peer could make us
/// allocate 4 GiB with a single header.
pub const MAX_FRAME: usize = 4 * 1024 * 1024;
pub const CHUNK_SIZE: usize = 256 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub from: Option<Peer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub ok: bool,
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default)]
    pub error: Option<String>,
}

impl Response {
    pub fn ok(id: &str, data: serde_json::Value) -> Self {
        Self { id: id.into(), ok: true, data, error: None }
    }
    pub fn err(id: &str, msg: &str) -> Self {
        Self { id: id.into(), ok: false, data: serde_json::Value::Null, error: Some(msg.into()) }
    }
}

pub async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, kind: u8, payload: &[u8]) -> anyhow::Result<()> {
    if payload.len() > MAX_FRAME {
        anyhow::bail!("frame too large");
    }
    let len = payload.len() as u32 + 1;
    let mut head = [0u8; 5];
    head[..4].copy_from_slice(&len.to_be_bytes());
    head[4] = kind;
    w.write_all(&head).await?;
    w.write_all(payload).await?;
    Ok(())
}

pub async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> anyhow::Result<(u8, Vec<u8>)> {
    let len = r.read_u32().await? as usize;
    if len == 0 {
        anyhow::bail!("empty frame");
    }
    if len - 1 > MAX_FRAME {
        anyhow::bail!("frame too large ({} bytes)", len - 1);
    }
    let kind = r.read_u8().await?;
    let mut buf = vec![0u8; len - 1];
    r.read_exact(&mut buf).await?;
    Ok((kind, buf))
}

pub async fn write_json<W: AsyncWrite + Unpin, T: Serialize>(w: &mut W, v: &T) -> anyhow::Result<()> {
    write_frame(w, KIND_JSON, &serde_json::to_vec(v)?).await?;
    w.flush().await?;
    Ok(())
}

pub async fn read_json<R: AsyncRead + Unpin, T: serde::de::DeserializeOwned>(r: &mut R) -> anyhow::Result<T> {
    let (kind, buf) = read_frame(r).await?;
    if kind != KIND_JSON {
        anyhow::bail!("expected JSON frame");
    }
    Ok(serde_json::from_slice(&buf)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn frame_roundtrip() {
        let (mut a, mut b) = tokio::io::duplex(1024);
        write_frame(&mut a, KIND_CHUNK, b"hello").await.unwrap();
        let (k, p) = read_frame(&mut b).await.unwrap();
        assert_eq!(k, KIND_CHUNK);
        assert_eq!(p, b"hello");
    }

    #[tokio::test]
    async fn oversized_frame_is_rejected_without_allocating() {
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_u32(u32::MAX).await.unwrap();
        assert!(read_frame(&mut b).await.is_err());
    }

    #[tokio::test]
    async fn zero_length_frame_is_rejected() {
        let (mut a, mut b) = tokio::io::duplex(64);
        a.write_u32(0).await.unwrap();
        assert!(read_frame(&mut b).await.is_err());
    }
}
