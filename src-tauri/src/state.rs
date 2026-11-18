use crate::db::Db;
use crate::identity::DeviceIdentity;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: String,
    pub os: String,
    pub version: String,
    pub ip: String,
    pub port: u16,
    pub shared_folders: Vec<String>,
    pub status: String, // online/connecting/transferring/offline/unavailable
    pub last_seen: i64,
}

#[derive(Clone)]
pub struct AppState {
    pub identity: Arc<DeviceIdentity>,
    pub db: Arc<Db>,
    pub devices: Arc<Mutex<HashMap<String, DiscoveredDevice>>>,
    pub shared_folders: Arc<Mutex<Vec<crate::db::SharedFolder>>>,
    pub transfer_active: Arc<Mutex<Vec<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        let identity = DeviceIdentity::load_or_create();
        let db = Db::open().expect("db open");
        let sf = db.list_shared_folders();
        Self {
            identity: Arc::new(identity),
            db: Arc::new(db),
            devices: Arc::new(Mutex::new(HashMap::new())),
            shared_folders: Arc::new(Mutex::new(sf)),
            transfer_active: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
