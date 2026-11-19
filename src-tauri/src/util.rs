use std::path::{Path, PathBuf};

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LocalSendStudio")
}

/// Where received files go by default.
pub fn default_download_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        let base = std::env::var("EXTERNAL_STORAGE").unwrap_or_else(|_| "/storage/emulated/0".into());
        return PathBuf::from(base).join("Download").join("LocalSendStudio");
    }
    #[cfg(not(target_os = "android"))]
    dirs::download_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("LocalSendStudio")
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn new_hasher() -> sha2::Sha256 {
    use sha2::Digest;
    sha2::Sha256::new()
}

pub fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn sha256_file(path: &Path) -> anyhow::Result<String> {
    use sha2::Digest;
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut hasher = new_hasher();
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(hasher.finalize()))
}

const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1",
    "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// A single file or folder name: no separators, no traversal, nothing Windows treats specially.
pub fn valid_name(part: &str) -> bool {
    if part.is_empty() || part == "." || part == ".." || part.len() > 255 {
        return false;
    }
    if part
        .chars()
        .any(|c| c.is_control() || matches!(c, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*'))
    {
        return false;
    }
    if part.ends_with('.') || part.ends_with(' ') {
        return false;
    }
    let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
    !RESERVED.contains(&stem.as_str())
}

/// Resolve a client-supplied relative path inside `root`.
/// Rejects traversal, absolute paths, Windows drive/ADS tricks and symlinks that leave the root.
pub fn safe_join(root: &Path, rel: &str) -> Option<PathBuf> {
    let rel = rel.replace('\\', "/");
    let mut out = root.to_path_buf();
    for part in rel.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if !valid_name(part) {
            return None;
        }
        out.push(part);
    }
    // Symlink escape: canonicalize the deepest existing ancestor and require it to stay inside root.
    let canon_root = root.canonicalize().ok()?;
    let mut probe = out.as_path();
    loop {
        if let Ok(c) = probe.canonicalize() {
            return if c.starts_with(&canon_root) { Some(out) } else { None };
        }
        probe = probe.parent()?;
    }
}

/// `name (1).ext`, `name (2).ext` ... until a free name is found.
pub fn unique_path(target: &Path) -> PathBuf {
    if !target.exists() {
        return target.to_path_buf();
    }
    let stem = target.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = target.extension().map(|s| format!(".{}", s.to_string_lossy())).unwrap_or_default();
    let dir = target.parent().unwrap_or_else(|| Path::new("."));
    for i in 1..10_000 {
        let c = dir.join(format!("{} ({}){}", stem, i, ext));
        if !c.exists() {
            return c;
        }
    }
    target.to_path_buf()
}

/// (name, ip, kind) for each usable IPv4 interface.
pub fn list_interfaces() -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    if let Ok(list) = if_addrs::get_if_addrs() {
        for i in list {
            if i.is_loopback() {
                continue;
            }
            if let if_addrs::IfAddr::V4(v4) = &i.addr {
                if v4.ip.is_link_local() {
                    continue;
                }
                let n = i.name.to_lowercase();
                let kind = if n.contains("wi-fi") || n.contains("wlan") || n.starts_with("wl") || n.contains("wireless") {
                    "wifi"
                } else if n.contains("vethernet")
                    || n.contains("docker")
                    || n.starts_with("veth")
                    || n.contains("vmware")
                    || n.contains("virtualbox")
                    || n.starts_with("br-")
                    || n.contains("tailscale")
                    || n.contains("vpn")
                    || n.starts_with("tun")
                {
                    "virtual"
                } else {
                    "ethernet"
                };
                out.push((i.name.clone(), v4.ip.to_string(), kind.to_string()));
            }
        }
    }
    out
}


pub fn hostname() -> String {
    for var in ["COMPUTERNAME", "HOSTNAME"] {
        if let Ok(s) = std::env::var(var) {
            if !s.trim().is_empty() {
                return s.trim().to_string();
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(s) = std::fs::read_to_string("/etc/hostname") {
            if !s.trim().is_empty() {
                return s.trim().to_string();
            }
        }
        if let Ok(o) = std::process::Command::new("hostname").output() {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    if cfg!(target_os = "android") {
        if let Ok(o) = std::process::Command::new("getprop").arg("ro.product.model").output() {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
        "Android device".into()
    } else {
        "LocalSendStudio device".into()
    }
}

pub fn os_name() -> String {
    if cfg!(target_os = "windows") {
        "Windows".into()
    } else if cfg!(target_os = "macos") {
        "macOS".into()
    } else if cfg!(target_os = "android") {
        "Android".into()
    } else if cfg!(target_os = "linux") {
        "Linux".into()
    } else if cfg!(target_os = "ios") {
        "iOS".into()
    } else {
        "Unknown".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("sub")).unwrap();
        std::fs::write(d.path().join("sub/a.txt"), "x").unwrap();
        d
    }

    #[test]
    fn allows_normal_paths() {
        let d = root();
        assert!(safe_join(d.path(), "").is_some());
        assert!(safe_join(d.path(), "sub/a.txt").is_some());
        assert!(safe_join(d.path(), "/sub//a.txt").is_some());
        assert!(safe_join(d.path(), "new/file.bin").is_some());
    }

    #[test]
    fn rejects_traversal_and_tricks() {
        let d = root();
        for bad in [
            "..", "../x", "sub/../../x", "sub\\..\\..\\x", "C:\\Windows", "C:evil", "a.txt:stream", "CON", "nul.txt",
            "x/aux", "trailing.", "bad|name", "q?",
        ] {
            assert!(safe_join(d.path(), bad).is_none(), "should reject {bad:?}");
        }
    }

    #[test]
    fn valid_name_rules() {
        assert!(valid_name("report v2.pdf"));
        assert!(valid_name("файл.txt"));
        assert!(!valid_name("../x"));
        assert!(!valid_name("a/b"));
        assert!(!valid_name(""));
    }

    #[test]
    fn unique_path_adds_counter() {
        let d = root();
        let p = d.path().join("sub/a.txt");
        assert_eq!(unique_path(&p), d.path().join("sub/a (1).txt"));
        assert_eq!(unique_path(&d.path().join("sub/none.txt")), d.path().join("sub/none.txt"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() {
        let d = root();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("secret"), "s").unwrap();
        std::os::unix::fs::symlink(outside.path(), d.path().join("link")).unwrap();
        assert!(safe_join(d.path(), "link/secret").is_none());
        assert!(safe_join(d.path(), "link").is_none());
    }
}
