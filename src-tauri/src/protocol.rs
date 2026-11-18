use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub const DEFAULT_PORT: u16 = 47821;
pub const DISCOVERY_PORT: u16 = 47822;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub ok: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

pub async fn write_frame(stream: &mut TcpStream, kind: u8, payload: &[u8]) -> anyhow::Result<()> {
    let len = (payload.len() as u32) + 1;
    stream.write_u32(len).await?;
    stream.write_u8(kind).await?;
    stream.write_all(payload).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn read_frame(stream: &mut TcpStream) -> anyhow::Result<(u8, Vec<u8>)> {
    let len = stream.read_u32().await?;
    if len == 0 { return Err(anyhow::anyhow!("empty frame")); }
    let kind = stream.read_u8().await?;
    let mut buf = vec![0u8; (len - 1) as usize];
    stream.read_exact(&mut buf).await?;
    Ok((kind, buf))
}

pub const KIND_JSON: u8 = 1;
pub const KIND_CHUNK: u8 = 2;
pub const KIND_END: u8 = 3;
