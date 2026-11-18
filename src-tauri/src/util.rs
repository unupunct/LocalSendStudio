use std::net::SocketAddr;
use std::path::PathBuf;

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LocalSendStudio")
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn sha256_file(path: &std::path::Path) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn safe_join(root: &std::path::Path, rel: &str) -> Option<PathBuf> {
    let rel = rel.replace('\\', "/");
    let rel = rel.trim_start_matches('/');
    let mut out = root.to_path_buf();
    for part in rel.split('/') {
        if part.is_empty() || part == "." { continue; }
        if part == ".." { return None; }
        out = out.join(part);
    }
    // ensure within root
    let canon_root = root.canonicalize().ok()?;
    // target may not exist; check lexical containment
    if !out.starts_with(root) { return None; }
    let _ = canon_root;
    Some(out)
}

pub fn local_ip() -> String {
    // try connecting to a public IP to get local addr (no data sent)
    if let Ok(s) = std::net::UdpSocket::bind("0.0.0.0:0") {
        let _ = s.connect("8.8.8.8:80");
        if let Ok(a) = s.local_addr() { return a.ip().to_string(); }
    }
    "127.0.0.1".into()
}

pub fn list_interfaces() -> Vec<(String, String, String)> {
    // name, ip, kind
    let mut out = Vec::new();
    // Windows: parse ipconfig
    #[cfg(target_os = "windows")]
    {
        if let Ok(o) = std::process::Command::new("ipconfig").output() {
            let txt = String::from_utf8_lossy(&o.stdout);
            let mut current = String::from("Adapter");
            for line in txt.lines() {
                let t = line.trim();
                if t.ends_with(':') && !t.contains(" ") == false {
                    // adapter header lines like "Ethernet adapter ...:"
                }
                if line.contains("adapter") {
                    current = line.trim().trim_end_matches(':').to_string();
                }
                if t.starts_with("IPv4 Address") || t.starts_with("IPv4") {
                    if let Some(ip) = t.split(':').nth(1) {
                        let ip = ip.trim().trim_matches('.').trim().to_string();
                        let ip = ip.replace("(Preferred)", "").trim().to_string();
                        if !ip.is_empty() {
                            let kind = if current.to_lowercase().contains("wi-fi") || current.to_lowercase().contains("wireless") { "wifi" } else { "ethernet" };
                            out.push((current.clone(), ip, kind.to_string()));
                        }
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // parse ip addr / ifconfig
        if let Ok(o) = std::process::Command::new("ip").args(["-o","addr"]).output() {
            let txt = String::from_utf8_lossy(&o.stdout);
            for line in txt.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let name = parts[1].to_string();
                    let ip = parts[3].split('/').next().unwrap_or("").to_string();
                    if ip.contains('.') && !ip.starts_with("127.") {
                        let kind = if name.starts_with("wl") || name.starts_with("wlan") { "wifi" } else { "ethernet" };
                        out.push((name, ip, kind.to_string()));
                    }
                }
            }
        }
    }
    if out.is_empty() {
        out.push(("Default".into(), local_ip(), "ethernet".into()));
    }
    out
}

pub fn hostname() -> String {
    #[cfg(target_os = "windows")]
    {
        if let Ok(o) = std::process::Command::new("hostname").output() {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !s.is_empty() { return s; }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(s) = std::fs::read_to_string("/etc/hostname") {
            let s = s.trim().to_string();
            if !s.is_empty() { return s; }
        }
        if let Ok(o) = std::process::Command::new("hostname").output() {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !s.is_empty() { return s; }
        }
    }
    "LocalSendStudio-PC".into()
}

pub fn os_name() -> String {
    if cfg!(target_os = "windows") { "Windows".into() }
    else if cfg!(target_os = "macos") { "macOS".into() }
    else if cfg!(target_os = "linux") { "Linux".into() }
    else if cfg!(target_os = "android") { "Android".into() }
    else if cfg!(target_os = "ios") { "iOS".into() }
    else { "Unknown".into() }
}

#[allow(dead_code)]
pub fn socket_addr(s: &str) -> Option<SocketAddr> {
    s.parse().ok()
}
