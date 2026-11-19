use crate::util;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFolder {
    pub id: String,
    pub name: String,
    pub path: String,
    pub enabled: bool,
    pub read: bool,
    pub upload: bool,
    pub manage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    pub id: String,
    pub filename: String,
    pub source: String,
    pub destination: String,
    pub size: i64,
    pub started: i64,
    pub ended: Option<i64>,
    pub speed: f64,
    pub status: String,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceGroup {
    pub id: String,
    pub name: String,
    pub device_ids: String, // comma separated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub id: String,
    pub name: String,
    pub added: i64,
}

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(dir: &std::path::Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join("localsend.db");
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS shared_folders (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                enabled INTEGER NOT NULL,
                read INTEGER NOT NULL,
                upload INTEGER NOT NULL,
                manage INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS transfer_history (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                source TEXT NOT NULL,
                destination TEXT NOT NULL,
                size INTEGER NOT NULL,
                started INTEGER NOT NULL,
                ended INTEGER,
                speed REAL NOT NULL,
                status TEXT NOT NULL,
                direction TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS device_groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                device_ids TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS trusted_devices (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                added INTEGER NOT NULL
            );"
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn get_setting(&self, key: &str) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT value FROM settings WHERE key=?1", params![key], |r| r.get(0)).ok()
    }
    pub fn set_setting(&self, key: &str, value: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=?2", params![key, value]);
    }

    pub fn list_shared_folders(&self) -> Vec<SharedFolder> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,name,path,enabled,read,upload,manage FROM shared_folders").unwrap();
        let rows = stmt.query_map([], |r| Ok(SharedFolder {
            id: r.get(0)?, name: r.get(1)?, path: r.get(2)?,
            enabled: r.get::<_,i64>(3)? != 0,
            read: r.get::<_,i64>(4)? != 0,
            upload: r.get::<_,i64>(5)? != 0,
            manage: r.get::<_,i64>(6)? != 0,
        })).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }
    pub fn upsert_shared_folder(&self, f: &SharedFolder) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO shared_folders(id,name,path,enabled,read,upload,manage) VALUES(?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(id) DO UPDATE SET name=?2,path=?3,enabled=?4,read=?5,upload=?6,manage=?7",
            params![f.id, f.name, f.path, f.enabled as i64, f.read as i64, f.upload as i64, f.manage as i64],
        );
    }
    pub fn delete_shared_folder(&self, id: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM shared_folders WHERE id=?1", params![id]);
    }

    pub fn insert_transfer(&self, t: &TransferRecord) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT OR REPLACE INTO transfer_history(id,filename,source,destination,size,started,ended,speed,status,direction) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![t.id,t.filename,t.source,t.destination,t.size,t.started,t.ended,t.speed,t.status,t.direction],
        );
    }
    pub fn list_transfers(&self) -> Vec<TransferRecord> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,filename,source,destination,size,started,ended,speed,status,direction FROM transfer_history ORDER BY started DESC").unwrap();
        let rows = stmt.query_map([], |r| Ok(TransferRecord {
            id: r.get(0)?, filename: r.get(1)?, source: r.get(2)?, destination: r.get(3)?,
            size: r.get(4)?, started: r.get(5)?, ended: r.get(6)?, speed: r.get(7)?, status: r.get(8)?, direction: r.get(9)?,
        })).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn clear_transfers(&self) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM transfer_history", []);
    }

    pub fn is_trusted(&self, id: &str) -> bool {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT 1 FROM trusted_devices WHERE id=?1", params![id], |_| Ok(())).is_ok()
    }
    pub fn trust(&self, id: &str, name: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO trusted_devices(id,name,added) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET name=?2",
            params![id, name, util::now_ms()],
        );
    }
    pub fn untrust(&self, id: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM trusted_devices WHERE id=?1", params![id]);
    }
    pub fn list_trusted(&self) -> Vec<TrustedDevice> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,name,added FROM trusted_devices ORDER BY added DESC").unwrap();
        let rows = stmt.query_map([], |r| Ok(TrustedDevice { id: r.get(0)?, name: r.get(1)?, added: r.get(2)? })).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn list_groups(&self) -> Vec<DeviceGroup> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id,name,device_ids FROM device_groups").unwrap();
        let rows = stmt.query_map([], |r| Ok(DeviceGroup { id: r.get(0)?, name: r.get(1)?, device_ids: r.get(2)? })).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }
    pub fn upsert_group(&self, g: &DeviceGroup) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("INSERT INTO device_groups(id,name,device_ids) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET name=?2,device_ids=?3", params![g.id,g.name,g.device_ids]);
    }
    pub fn delete_group(&self, id: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM device_groups WHERE id=?1", params![id]);
    }
}
