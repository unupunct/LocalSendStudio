use crate::util;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub id: String,
    pub name: String,
    pub os: String,
    pub version: String,
}

impl DeviceIdentity {
    pub fn load_or_create() -> Self {
        let dir = util::app_data_dir();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("identity.json");
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(id) = serde_json::from_str::<DeviceIdentity>(&data) {
                return id;
            }
        }
        let id = DeviceIdentity {
            id: Uuid::new_v4().to_string(),
            name: util::hostname(),
            os: util::os_name(),
            version: "0.1.0".into(),
        };
        let _ = std::fs::write(&path, serde_json::to_string_pretty(&id).unwrap());
        id
    }

    pub fn save(&self) {
        let dir = util::app_data_dir();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("identity.json");
        let _ = std::fs::write(&path, serde_json::to_string_pretty(self).unwrap());
    }
}
