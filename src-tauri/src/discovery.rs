//! LAN discovery: UDP announcements over multicast and per-interface directed broadcast,
//! plus polling of devices added manually by IP (for networks that block both).
use crate::protocol::DISCOVERY_PORT;
use crate::state::{AppState, DiscoveredDevice, VERSION};
use crate::{client, util};
use serde_json::{json, Value};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket as StdUdp};
use std::time::Duration;

const MULTICAST: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 99);

pub const ONLINE_WINDOW_MS: i64 = 12_000;
pub const FORGET_AFTER_MS: i64 = 180_000;

pub fn start(state: AppState) {
    let s1 = state.clone();
    tokio::spawn(async move { listener_loop(s1).await });
    let s2 = state.clone();
    tokio::spawn(async move { announcer_loop(s2).await });
    let s3 = state;
    tokio::spawn(async move { manual_loop(s3).await });
}

fn announce_payload(state: &AppState) -> Vec<u8> {
    json!({
        "type": "announce",
        "id": state.identity.id,
        "name": state.device_name(),
        "os": state.identity.os,
        "version": VERSION,
        "port": state.port(),
        "shared_folders": state.enabled_share_names(),
    })
    .to_string()
    .into_bytes()
}

async fn announcer_loop(state: AppState) {
    loop {
        let data = announce_payload(&state);
        let ifaces: Vec<(Ipv4Addr, Option<Ipv4Addr>)> = if_addrs::get_if_addrs()
            .unwrap_or_default()
            .into_iter()
            .filter(|i| !i.is_loopback())
            .filter_map(|i| match i.addr {
                if_addrs::IfAddr::V4(v4) if !v4.ip.is_link_local() => Some((v4.ip, v4.broadcast)),
                _ => None,
            })
            .collect();
        for (ip, bcast) in ifaces {
            if let Ok(sock) = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)) {
                let _ = sock.set_broadcast(true);
                let _ = sock.set_multicast_if_v4(&ip);
                let _ = sock.set_multicast_ttl_v4(1);
                if sock.bind(&SockAddr::from(SocketAddrV4::new(ip, 0))).is_ok() {
                    let sock: StdUdp = sock.into();
                    let _ = sock.send_to(&data, SocketAddrV4::new(MULTICAST, DISCOVERY_PORT));
                    if let Some(b) = bcast {
                        let _ = sock.send_to(&data, SocketAddrV4::new(b, DISCOVERY_PORT));
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

fn bind_listener() -> std::io::Result<tokio::net::UdpSocket> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    sock.set_reuse_address(true)?;
    #[cfg(all(unix, not(any(target_os = "solaris", target_os = "illumos", target_os = "cygwin"))))]
    let _ = sock.set_reuse_port(true);
    sock.set_nonblocking(true)?;
    sock.bind(&SockAddr::from(SocketAddr::from(([0, 0, 0, 0], DISCOVERY_PORT))))?;
    let std: StdUdp = sock.into();
    tokio::net::UdpSocket::from_std(std)
}

fn join_all(sock: &tokio::net::UdpSocket) {
    for (_, ip, _) in util::list_interfaces() {
        if let Ok(ip) = ip.parse::<Ipv4Addr>() {
            let _ = sock.join_multicast_v4(MULTICAST, ip); // errors if already joined: harmless
        }
    }
}

async fn listener_loop(state: AppState) {
    let sock = match bind_listener() {
        Ok(s) => s,
        Err(e) => {
            log::error!("discovery: cannot bind UDP {}: {}", DISCOVERY_PORT, e);
            return;
        }
    };
    join_all(&sock);
    let mut buf = vec![0u8; 8192];
    let mut rejoin = tokio::time::interval(Duration::from_secs(30));
    loop {
        tokio::select! {
            _ = rejoin.tick() => join_all(&sock),
            r = sock.recv_from(&mut buf) => {
                if let Ok((n, addr)) = r {
                    if let Ok(v) = serde_json::from_slice::<Value>(&buf[..n]) {
                        handle_announce(&state, &v, addr.ip().to_string());
                    }
                }
            }
        }
    }
}

fn handle_announce(state: &AppState, v: &Value, ip: String) {
    if v.get("type").and_then(|x| x.as_str()) != Some("announce") {
        return;
    }
    let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
    if id.is_empty() || id == state.identity.id {
        return;
    }
    let s = |k: &str, d: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or(d).chars().take(120).collect::<String>();
    let port = v.get("port").and_then(|x| x.as_u64()).unwrap_or(0) as u16;
    if port == 0 {
        return;
    }
    let shares = v
        .get("shared_folders")
        .and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.chars().take(120).collect())).take(64).collect())
        .unwrap_or_default();
    upsert(state, DiscoveredDevice {
        id,
        name: s("name", "Unknown"),
        os: s("os", ""),
        version: s("version", ""),
        ip,
        port,
        shared_folders: shares,
        status: "online".into(),
        last_seen: util::now_ms(),
        trusted: false,
    });
}

pub fn upsert(state: &AppState, dev: DiscoveredDevice) {
    let changed = {
        let mut map = state.devices.lock();
        let changed = match map.get(&dev.id) {
            None => true,
            Some(old) => old.name != dev.name || old.ip != dev.ip || old.port != dev.port || old.shared_folders != dev.shared_folders || old.last_seen + ONLINE_WINDOW_MS < dev.last_seen,
        };
        map.insert(dev.id.clone(), dev);
        changed
    };
    if changed {
        (state.emit)("devices-updated", Value::Null);
    }
}

pub fn manual_targets(state: &AppState) -> Vec<String> {
    state
        .db
        .get_setting("manual_devices")
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default()
}

/// Ask a device directly (used for "Add by IP" and for polling manually added devices).
pub async fn probe(state: &AppState, host: &str, port: u16) -> anyhow::Result<DiscoveredDevice> {
    let v = client::info(host, port).await?;
    let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
    if id.is_empty() {
        anyhow::bail!("not a LocalSendStudio device");
    }
    let shares = v
        .get("shared_folders")
        .and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let dev = DiscoveredDevice {
        id,
        name: v.get("name").and_then(|x| x.as_str()).unwrap_or("Unknown").into(),
        os: v.get("os").and_then(|x| x.as_str()).unwrap_or("").into(),
        version: v.get("version").and_then(|x| x.as_str()).unwrap_or("").into(),
        ip: host.to_string(),
        port,
        shared_folders: shares,
        status: "online".into(),
        last_seen: util::now_ms(),
        trusted: false,
    };
    upsert(state, dev.clone());
    Ok(dev)
}

async fn manual_loop(state: AppState) {
    loop {
        for t in manual_targets(&state) {
            let (host, port) = match t.rsplit_once(':') {
                Some((h, p)) => (h.to_string(), p.parse().unwrap_or(crate::protocol::DEFAULT_PORT)),
                None => (t.clone(), crate::protocol::DEFAULT_PORT),
            };
            let _ = probe(&state, &host, port).await;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
