//! End-to-end tests: a real server on a loopback port, driven by the real client.
use crate::client;
use crate::db::SharedFolder;
use crate::protocol::Peer;
use crate::server;
use crate::state::AppState;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct Env {
    state: AppState,
    port: u16,
    share_dir: tempfile::TempDir,
    _data: tempfile::TempDir,
    work: tempfile::TempDir,
}

fn peer(id: &str) -> Peer {
    Peer { id: id.into(), name: format!("peer-{id}") }
}

async fn env_with(mode: &str, read: bool, upload: bool, manage: bool, emit: crate::state::Emitter) -> Env {
    let data = tempfile::tempdir().unwrap();
    let share_dir = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let state = AppState::new(data.path(), emit);
    state.db.set_setting("trust_mode", mode);
    let f = SharedFolder {
        id: "1".into(),
        name: "Docs".into(),
        path: share_dir.path().to_string_lossy().to_string(),
        enabled: true,
        read,
        upload,
        manage,
    };
    state.shared_folders.lock().push(f);
    let (listener, port) = server::bind(0).unwrap();
    let st = state.clone();
    tokio::spawn(async move { server::run_server(st, listener).await });
    Env { state, port, share_dir, _data: data, work }
}

async fn env(mode: &str, read: bool, upload: bool, manage: bool) -> Env {
    env_with(mode, read, upload, manage, Arc::new(|_, _| {})).await
}

fn write_file(p: &Path, len: usize) -> Vec<u8> {
    let data: Vec<u8> = (0..len).map(|i| (i * 31 % 251) as u8).collect();
    std::fs::write(p, &data).unwrap();
    data
}

#[tokio::test]
async fn upload_then_download_roundtrip_is_verified() {
    let e = env("open", true, true, false).await;
    let me = peer("a");
    let src = e.work.path().join("big.bin");
    let data = write_file(&src, 3 * 1024 * 1024 + 17);

    let (stored, size) = client::upload(&me, "127.0.0.1", e.port, "Docs", "in/big.bin", &src, None, |_, _| {}).await.unwrap();
    assert_eq!(stored, "in/big.bin");
    assert_eq!(size as usize, data.len());
    assert_eq!(std::fs::read(e.share_dir.path().join("in/big.bin")).unwrap(), data);
    // No temp files left behind.
    assert!(!e.share_dir.path().join("in/big.bin.lsspart").exists());

    let dest = e.work.path().join("out.bin");
    let (final_path, _) = client::download(&me, "127.0.0.1", e.port, "Docs", "in/big.bin", &dest, None, |_, _| {}).await.unwrap();
    assert_eq!(std::fs::read(final_path).unwrap(), data);
}

#[tokio::test]
async fn upload_never_overwrites() {
    let e = env("open", true, true, false).await;
    let me = peer("a");
    let src = e.work.path().join("note.txt");
    std::fs::write(&src, "one").unwrap();
    client::upload(&me, "127.0.0.1", e.port, "Docs", "note.txt", &src, None, |_, _| {}).await.unwrap();
    std::fs::write(&src, "two").unwrap();
    let (stored, _) = client::upload(&me, "127.0.0.1", e.port, "Docs", "note.txt", &src, None, |_, _| {}).await.unwrap();
    assert_eq!(stored, "note (1).txt");
    assert_eq!(std::fs::read_to_string(e.share_dir.path().join("note.txt")).unwrap(), "one");
    assert_eq!(std::fs::read_to_string(e.share_dir.path().join("note (1).txt")).unwrap(), "two");
}

#[tokio::test]
async fn interrupted_download_resumes() {
    let e = env("open", true, false, false).await;
    let me = peer("a");
    let data = write_file(&e.share_dir.path().join("movie.bin"), 2 * 1024 * 1024);
    let dest = e.work.path().join("movie.bin");
    // A previous attempt left the first 700 000 bytes behind.
    std::fs::write(format!("{}.lsspart", dest.display()), &data[..700_000]).unwrap();
    let mut first_progress = None;
    let (p, _) = client::download(&me, "127.0.0.1", e.port, "Docs", "movie.bin", &dest, None, |done, _| {
        first_progress.get_or_insert(done);
    })
    .await
    .unwrap();
    assert_eq!(first_progress, Some(700_000), "must start from the existing offset");
    assert_eq!(std::fs::read(p).unwrap(), data);
}

#[tokio::test]
async fn stale_partial_file_is_discarded_and_download_restarts() {
    let e = env("open", true, false, false).await;
    let me = peer("a");
    let data = write_file(&e.share_dir.path().join("f.bin"), 500_000);
    let dest = e.work.path().join("f.bin");
    std::fs::write(format!("{}.lsspart", dest.display()), vec![0xAAu8; 100_000]).unwrap(); // wrong bytes
    let (p, _) = client::download(&me, "127.0.0.1", e.port, "Docs", "f.bin", &dest, None, |_, _| {}).await.unwrap();
    assert_eq!(std::fs::read(p).unwrap(), data);
}

#[tokio::test]
async fn folder_permissions_are_enforced() {
    // read-only share
    let e = env("open", true, false, false).await;
    let me = peer("a");
    write_file(&e.share_dir.path().join("a.txt"), 10);
    assert!(client::list_dir(&me, "127.0.0.1", e.port, "Docs", "").await.is_ok());
    let src = e.work.path().join("x.txt");
    std::fs::write(&src, "x").unwrap();
    let up = client::upload(&me, "127.0.0.1", e.port, "Docs", "x.txt", &src, None, |_, _| {}).await;
    assert!(up.unwrap_err().to_string().contains("not allowed"));
    assert!(client::delete(&me, "127.0.0.1", e.port, "Docs", "a.txt").await.is_err());
    assert!(client::rename(&me, "127.0.0.1", e.port, "Docs", "a.txt", "b.txt").await.is_err());
    assert!(client::mkdir(&me, "127.0.0.1", e.port, "Docs", "new").await.is_err());
    assert!(e.share_dir.path().join("a.txt").exists());

    // no-read share: cannot list or download
    let e2 = env("open", false, true, false).await;
    write_file(&e2.share_dir.path().join("a.txt"), 10);
    assert!(client::list_dir(&me, "127.0.0.1", e2.port, "Docs", "").await.is_err());
    let r = client::download(&me, "127.0.0.1", e2.port, "Docs", "a.txt", &e2.work.path().join("a.txt"), None, |_, _| {}).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn path_traversal_is_blocked_over_the_wire() {
    let e = env("open", true, true, true).await;
    let me = peer("a");
    let outside: PathBuf = e.share_dir.path().parent().unwrap().join("lss-outside-secret.txt");
    std::fs::write(&outside, "secret").unwrap();
    for bad in ["../lss-outside-secret.txt", "..\\lss-outside-secret.txt", "sub/../../lss-outside-secret.txt"] {
        let r = client::download(&me, "127.0.0.1", e.port, "Docs", bad, &e.work.path().join("s.txt"), None, |_, _| {}).await;
        assert!(r.is_err(), "download of {bad:?} must fail");
        assert!(client::delete(&me, "127.0.0.1", e.port, "Docs", bad).await.is_err());
    }
    // Renaming into another folder via the new name is rejected too.
    write_file(&e.share_dir.path().join("a.txt"), 5);
    assert!(client::rename(&me, "127.0.0.1", e.port, "Docs", "a.txt", "../moved.txt").await.is_err());
    assert!(outside.exists());
    std::fs::remove_file(outside).ok();
}

#[tokio::test]
async fn cannot_delete_the_share_root() {
    let e = env("open", true, true, true).await;
    let me = peer("a");
    assert!(client::delete(&me, "127.0.0.1", e.port, "Docs", "").await.is_err());
    assert!(e.share_dir.path().exists());
}

#[tokio::test]
async fn unknown_peer_is_refused_in_ask_mode_without_ui() {
    let e = env("ask", true, true, true).await;
    // No identity at all -> refused immediately.
    let r = client::call(None, "127.0.0.1", e.port, "list_dir", serde_json::json!({"share":"Docs","path":""})).await;
    assert!(r.unwrap_err().to_string().contains("not authorized"));
    // `info` stays public.
    assert!(client::info("127.0.0.1", e.port).await.is_ok());
}

#[tokio::test]
async fn approval_flow_allow_and_remember() {
    use std::sync::Mutex;
    let slot: Arc<Mutex<Option<AppState>>> = Arc::new(Mutex::new(None));
    let slot2 = slot.clone();
    let emit: crate::state::Emitter = Arc::new(move |ev, payload| {
        if ev == "approval-request" {
            let st = slot2.lock().unwrap().clone().unwrap();
            let id = payload["id"].as_str().unwrap().to_string();
            std::thread::spawn(move || crate::auth::resolve(&st, &id, true, true));
        }
    });
    let e = env_with("ask", true, false, false, emit).await;
    *slot.lock().unwrap() = Some(e.state.clone());
    let me = peer("new-device");
    assert!(client::list_dir(&me, "127.0.0.1", e.port, "Docs", "").await.is_ok());
    assert!(e.state.db.is_trusted("new-device"), "'remember' must persist the peer");
    // Second connection needs no prompt.
    assert!(client::list_dir(&me, "127.0.0.1", e.port, "Docs", "").await.is_ok());
}

#[tokio::test]
async fn approval_flow_deny() {
    use std::sync::Mutex;
    let slot: Arc<Mutex<Option<AppState>>> = Arc::new(Mutex::new(None));
    let slot2 = slot.clone();
    let emit: crate::state::Emitter = Arc::new(move |ev, payload| {
        if ev == "approval-request" {
            let st = slot2.lock().unwrap().clone().unwrap();
            let id = payload["id"].as_str().unwrap().to_string();
            std::thread::spawn(move || crate::auth::resolve(&st, &id, false, false));
        }
    });
    let e = env_with("ask", true, false, false, emit).await;
    *slot.lock().unwrap() = Some(e.state.clone());
    let me = peer("bad-device");
    assert!(client::list_dir(&me, "127.0.0.1", e.port, "Docs", "").await.is_err());
    assert!(!e.state.db.is_trusted("bad-device"));
}

#[tokio::test]
async fn malicious_oversized_frame_does_not_crash_server() {
    use tokio::io::AsyncWriteExt;
    let e = env("open", true, false, false).await;
    let mut s = tokio::net::TcpStream::connect(("127.0.0.1", e.port)).await.unwrap();
    s.write_u32(u32::MAX).await.unwrap();
    s.write_u8(1).await.unwrap();
    drop(s);
    // Server still answers.
    assert!(client::info("127.0.0.1", e.port).await.is_ok());
}

#[tokio::test]
async fn speedtest_reports_throughput() {
    let e = env("open", false, false, false).await;
    let (bps, _rtt) = client::speedtest(&peer("a"), "127.0.0.1", e.port, 8 << 20).await.unwrap();
    assert!(bps > 0.0);
}
