use rusqlite::{params, Connection, Result, OptionalExtension};
use chrono::{Utc, DateTime};
use serde::Serialize;
use sha2::{Sha256, Digest};
use hex;

use crate::utils::haversine_distance;

#[derive(Debug, Clone, Serialize)]
pub struct DeviceScanData {
    pub name: Option<String>,
    pub address: String,
    pub rssi: i32,
    pub tx_power: i32,
    pub manufacturer_data: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone)]
pub struct DeviceDetection {
    pub timestamp: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64,
    pub rssi: i32,
    pub tx_power: i32,
    pub manufacturer_data: String,
}

#[derive(Debug, Clone)]
pub struct DeviceEntry {
    pub address: String,
    pub name: String,
    pub detections: Vec<DeviceDetection>,
}

#[derive(Debug, Clone)]
pub struct FilterOptions {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

pub struct BluetoothTracker {
    conn: Connection,
}

impl BluetoothTracker {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create tables if they don't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS devices (
                address TEXT PRIMARY KEY,
                name TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS detections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_address TEXT,
                timestamp TEXT,
                latitude REAL,
                longitude REAL,
                rssi INTEGER,
                tx_power INTEGER,
                manufacturer_data TEXT,
                FOREIGN KEY(device_address) REFERENCES devices(address)
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    /// Hashes the Bluetooth address using SHA-256
    fn hash_address(address: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(address);
        hex::encode(hasher.finalize()) // Convert to hex string
    }

    pub fn store_scan_data(&self, scan_data: DeviceScanData) -> Result<()> {
        let now = Utc::now();
        let hashed_address = Self::hash_address(&scan_data.address);

        let detection = DeviceDetection {
            timestamp: now,
            latitude: scan_data.latitude,
            longitude: scan_data.longitude,
            rssi: scan_data.rssi,
            tx_power: scan_data.tx_power,
            manufacturer_data: scan_data.manufacturer_data.clone(),
        };

        // Check if the old plaintext address exists
        let mut stmt = self.conn.prepare("SELECT address FROM devices WHERE address = ?")?;
        let old_address: Option<String> = stmt.query_row(params![scan_data.address], |row| row.get(0)).optional()?;

        if let Some(_) = old_address {
            // If old address exists, update it to the hashed version
            self.conn.execute(
                "UPDATE devices SET address = ?1 WHERE address = ?2",
                params![hashed_address, scan_data.address],
            )?;
        } else {
            // Insert hashed address if it does not exist
            let mut stmt = self.conn.prepare("SELECT name FROM devices WHERE address = ?")?;
            let device_name: Option<String> = stmt
                .query_row(params![hashed_address], |row| row.get(0))
                .optional()?;

            if device_name.is_none() {
                self.conn.execute(
                    "INSERT INTO devices (address, name) VALUES (?1, ?2)",
                    params![hashed_address, scan_data.name.clone().unwrap_or_else(|| "Unknown".to_string())],
                )?;
            }
        }

        // Insert detection with hashed address
        self.conn.execute(
            "INSERT INTO detections (device_address, timestamp, latitude, longitude, rssi, tx_power, manufacturer_data) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                hashed_address,
                detection.timestamp.to_rfc3339(),
                detection.latitude,
                detection.longitude,
                detection.rssi,
                detection.tx_power,
                detection.manufacturer_data
            ],
        )?;

        Ok(())
    }

    pub fn get_device_history(&mut self, address: &str, filters: FilterOptions) -> Result<Vec<DeviceDetection>> {
        let hashed_address = Self::hash_address(address);

        let mut query = String::from(
            "SELECT timestamp, latitude, longitude, rssi, tx_power, manufacturer_data 
             FROM detections 
             WHERE device_address = ?"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(hashed_address)];

        if let Some(start) = filters.start_time {
            query.push_str(" AND timestamp >= ?");
            params.push(Box::new(start.to_rfc3339()));
        }
        if let Some(end) = filters.end_time {
            query.push_str(" AND timestamp <= ?");
            params.push(Box::new(end.to_rfc3339()));
        }

        query.push_str(" ORDER BY timestamp DESC");

        let lim = filters.limit.unwrap_or(50);
        query.push_str(" LIMIT ?");
        params.push(Box::new(lim as i64));

        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(DeviceDetection {
                timestamp: row.get(0)?,
                latitude: row.get(1)?,
                longitude: row.get(2)?,
                rssi: row.get(3)?,
                tx_power: row.get(4)?,
                manufacturer_data: row.get(5)?,
            })
        })?;

        let history: Result<Vec<_>> = rows.collect();
        history
    }

    pub fn estimate_device_location(&self, address: &str) -> Result<Option<(f64, f64)>> {
        let hashed_address = Self::hash_address(address);

        let mut stmt = self.conn.prepare(
            "SELECT latitude, longitude 
             FROM detections 
             WHERE device_address = ? 
             ORDER BY timestamp DESC 
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![hashed_address])?;
        if let Some(row) = rows.next()? {
            let latitude: f64 = row.get(0)?;
            let longitude: f64 = row.get(1)?;
            Ok(Some((latitude, longitude)))
        } else {
            Ok(None)
        }
    }
}

pub fn get_db_path(provided_path: Option<String>) -> String {
    provided_path.unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{}/.bluetracker/bluetooth_devices.db", home)
    })
}
