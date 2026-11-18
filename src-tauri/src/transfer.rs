use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferJob {
    pub id: String,
    pub direction: String,
    pub source_device: String,
    pub dest_device: String,
    pub filename: String,
    pub remote_path: String,
    pub local_path: String,
    pub share: String,
    pub ip: String,
    pub port: u16,
    pub size: u64,
    pub transferred: u64,
    pub speed: f64,
    pub status: String,
    pub device_id: String,
}

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc as StdArc;

pub type CancelMap = StdArc<Mutex<HashMap<String, StdArc<AtomicBool>>>>;

pub fn new_cancel_map() -> CancelMap {
    StdArc::new(Mutex::new(HashMap::new()))
}
