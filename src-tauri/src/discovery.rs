use crate::protocol::DISCOVERY_PORT;
use crate::state::{AppState, DiscoveredDevice};
use serde_json::json;
use std::net::{Ipv4Addr, SocketAddrV4};
use tauri::{AppHandle, Emitter};

pub async fn start(state: AppState, app: AppHandle) {
    // listener
    let st = state.clone();
    let ap = app.clone();
    tokio::spawn(async move { listener_loop(st, ap).await; });
    // announcer
    let st2 = state.clone();
    tokio::spawn(async move { announcer_loop(st2).await; });
}

async fn announcer_loop(state: AppState) {
    use tokio::net::UdpSocket;
    let sock = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(_) => return,
    };
    let _ = sock.set_broadcast(true);
    loop {
        let folders = state.shared_folders.lock().clone();
        let names: Vec<String> = folders.iter().filter(|f| f.enabled).map(|f| f.name.clone()).collect();
        let msg = json!({
            "type": "announce",
            "id": state.identity.id,
            "name": state.identity.name,
            "os": state.identity.os,
            "version": state.identity.version,
            "port": 47821u16,
            "shared_folders": names,
        });
        let data = msg.to_string().into_bytes();
        // multicast
        let maddr: SocketAddrV4 = "239.255.42.99:47822".parse().unwrap();
        let _ = sock.send_to(&data, maddr).await;
        // broadcast
        let baddr: SocketAddrV4 = "255.255.255.255:47822".parse().unwrap();
        let _ = sock.send_to(&data, baddr).await;
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn listener_loop(state: AppState, app: AppHandle) {
    use tokio::net::UdpSocket;
    let sock = match UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)).await {
        Ok(s) => s,
        Err(e) => { log::error!("discovery bind fail: {}", e); return; }
    };
    // join multicast
    if let Ok(multi) = "239.255.42.99".parse::<Ipv4Addr>() {
        let _ = sock.join_multicast_v4(multi, Ipv4Addr::UNSPECIFIED);
    }
    let mut buf = vec![0u8; 65536];
    loop {
        match sock.recv_from(&mut buf).await {
            Ok((n, addr)) => {
                if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&buf[..n]) {
                    if v.get("type").and_then(|x| x.as_str()) == Some("announce") {
                        let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
                        if id == state.identity.id { continue; }
                        let ip = addr.ip().to_string();
                        let port = v.get("port").and_then(|x| x.as_u64()).unwrap_or(47821) as u16;
                        let sf = v.get("shared_folders").and_then(|x| x.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_default();
                        let dev = DiscoveredDevice {
                            id: id.clone(),
                            name: v.get("name").and_then(|x| x.as_str()).unwrap_or("Unknown").to_string(),
                            os: v.get("os").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                            version: v.get("version").and_then(|x| x.as_str()).unwrap_or("").to_string(),
                            ip,
                            port,
                            shared_folders: sf,
                            status: "online".into(),
                            last_seen: chrono::Utc::now().timestamp_millis(),
                        };
                        state.devices.lock().insert(id, dev);
                        let _ = app.emit("devices-updated", ());
                    }
                }
            }
            Err(_) => {}
        }
    }
}
