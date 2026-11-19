//! Background transfer jobs with progress events and cancellation.
use crate::client;
use crate::db::TransferRecord;
use crate::protocol::Peer;
use crate::state::AppState;
use crate::util;
use parking_lot::Mutex;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub type CancelMap = Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>;

pub fn new_cancel_map() -> CancelMap {
    Arc::new(Mutex::new(HashMap::new()))
}

#[derive(Clone)]
pub enum Kind {
    /// Fetch `remote_path` into the local folder `local_dir`.
    Download { remote_path: String, local_dir: PathBuf },
    /// Send the local file `local_path` to `remote_path` on the share.
    Upload { remote_path: String, local_path: PathBuf },
}

#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub share: String,
    pub device_name: String,
    pub filename: String,
    pub kind: Kind,
}

pub fn spawn(state: AppState, cancels: CancelMap, job: Job) {
    let flag = Arc::new(AtomicBool::new(false));
    cancels.lock().insert(job.id.clone(), flag.clone());
    let direction = match job.kind {
        Kind::Download { .. } => "download",
        Kind::Upload { .. } => "upload",
    };
    (state.emit)(
        "transfer-started",
        json!({ "id": job.id, "filename": job.filename, "direction": direction, "device": job.device_name }),
    );
    tokio::spawn(async move {
        let me = Peer { id: state.identity.id.clone(), name: state.device_name() };
        let started = util::now_ms();
        let mut last_t = started;
        let mut last_b = 0u64;
        let mut speed = 0f64;
        let mut last_emit = 0i64;
        let emit = state.emit.clone();
        let id = job.id.clone();
        let mut on_progress = move |done: u64, total: u64| {
            let now = util::now_ms();
            if now - last_t >= 500 {
                let inst = (done.saturating_sub(last_b)) as f64 / ((now - last_t) as f64 / 1000.0);
                speed = if speed == 0.0 { inst } else { speed * 0.6 + inst * 0.4 };
                last_t = now;
                last_b = done;
            }
            if now - last_emit >= 150 || done == total {
                last_emit = now;
                let eta = if speed > 1.0 && total > done { ((total - done) as f64 / speed) as i64 } else { -1 };
                emit("transfer-progress", json!({ "id": id, "transferred": done, "total": total, "speed": speed, "eta": eta }));
            }
        };

        let result: anyhow::Result<(String, u64)> = match &job.kind {
            Kind::Download { remote_path, local_dir } => {
                let dest = local_dir.join(&job.filename);
                client::download(&me, &job.ip, job.port, &job.share, remote_path, &dest, Some(flag.clone()), &mut on_progress)
                    .await
                    .map(|(p, s)| (p.to_string_lossy().to_string(), s))
            }
            Kind::Upload { remote_path, local_path } => {
                client::upload(&me, &job.ip, job.port, &job.share, remote_path, local_path, Some(flag.clone()), &mut on_progress).await
            }
        };

        let was_cancelled = flag.load(Ordering::Relaxed);
        cancels.lock().remove(&job.id);
        let (status, error, size) = match &result {
            Ok((_, s)) => ("completed", None, *s),
            Err(_) if was_cancelled => ("cancelled", None, 0),
            Err(e) => ("failed", Some(e.to_string()), 0),
        };
        let ended = util::now_ms();
        let secs = ((ended - started).max(1)) as f64 / 1000.0;
        let (source, destination) = match job.kind {
            Kind::Download { .. } => (job.device_name.clone(), state.device_name()),
            Kind::Upload { .. } => (state.device_name(), job.device_name.clone()),
        };
        state.db.insert_transfer(&TransferRecord {
            id: job.id.clone(),
            filename: job.filename.clone(),
            source,
            destination,
            size: size as i64,
            started,
            ended: Some(ended),
            speed: if status == "completed" { size as f64 / secs } else { 0.0 },
            status: status.into(),
            direction: direction.into(),
        });
        (state.emit)("transfer-done", json!({ "id": job.id, "status": status, "error": error }));
        (state.emit)("history-updated", serde_json::Value::Null);
    });
}
