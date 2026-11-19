use crate::db::{Db, SharedFolder};
use crate::identity::DeviceIdentity;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::AtomicU16;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::oneshot;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: String,
    pub os: String,
    pub version: String,
    pub ip: String,
    pub port: u16,
    pub shared_folders: Vec<String>,
    pub status: String,
    pub last_seen: i64,
    pub trusted: bool,
}

pub type Emitter = Arc<dyn Fn(&str, serde_json::Value) + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub identity: Arc<DeviceIdentity>,
    pub name: Arc<RwLock<String>>,
    pub db: Arc<Db>,
    pub devices: Arc<Mutex<HashMap<String, DiscoveredDevice>>>,
    pub shared_folders: Arc<Mutex<Vec<SharedFolder>>>,
    pub port: Arc<AtomicU16>,
    pub emit: Emitter,
    /// Peers waiting for the user to allow or deny them: device id -> waiting connections.
    pub pending: Arc<Mutex<HashMap<String, Vec<oneshot::Sender<bool>>>>>,
    /// Peers allowed for this session only.
    pub session_allowed: Arc<Mutex<HashSet<String>>>,
    /// Peers denied recently; they are refused without prompting again for a while.
    pub denied: Arc<Mutex<HashMap<String, Instant>>>,
}

impl AppState {
    pub fn new(dir: &Path, emit: Emitter) -> Self {
        let identity = DeviceIdentity::load_or_create(dir);
        let db = Db::open(dir).expect("open database");
        let name = db.get_setting("device_name").filter(|n| !n.trim().is_empty()).unwrap_or_else(crate::util::hostname);
        let sf = db.list_shared_folders();
        Self {
            identity: Arc::new(identity),
            name: Arc::new(RwLock::new(name)),
            db: Arc::new(db),
            devices: Arc::new(Mutex::new(HashMap::new())),
            shared_folders: Arc::new(Mutex::new(sf)),
            port: Arc::new(AtomicU16::new(crate::protocol::DEFAULT_PORT)),
            emit,
            pending: Arc::new(Mutex::new(HashMap::new())),
            session_allowed: Arc::new(Mutex::new(HashSet::new())),
            denied: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn device_name(&self) -> String {
        self.name.read().clone()
    }

    pub fn port(&self) -> u16 {
        self.port.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// "ask" (default) or "open".
    pub fn trust_mode(&self) -> String {
        self.db.get_setting("trust_mode").unwrap_or_else(|| "ask".into())
    }

    pub fn enabled_share_names(&self) -> Vec<String> {
        self.shared_folders.lock().iter().filter(|f| f.enabled).map(|f| f.name.clone()).collect()
    }
}
