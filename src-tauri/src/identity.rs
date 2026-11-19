use crate::util;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub id: String,
    pub os: String,
}

impl DeviceIdentity {
    pub fn load_or_create(dir: &Path) -> Self {
        let _ = std::fs::create_dir_all(dir);
        let path = dir.join("identity.json");
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(mut id) = serde_json::from_str::<DeviceIdentity>(&data) {
                id.os = util::os_name();
                return id;
            }
        }
        let id = DeviceIdentity { id: Uuid::new_v4().to_string(), os: util::os_name() };
        let _ = std::fs::write(&path, serde_json::to_string_pretty(&id).unwrap());
        id
    }
}
