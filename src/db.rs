use rusqlite::{params, Connection, Result, OptionalExtension};
use chrono::{Utc, DateTime};
use serde::Serialize;

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
    pub name: String, // Device name (default: "Unknown")
    pub detections: Vec<DeviceDetection>, // Stores location history
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
                manufacturer_data TEXT,  -- This column is now TEXT but can store NULL
                FOREIGN KEY(device_address) REFERENCES devices(address)
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn store_scan_data(&self, scan_data: DeviceScanData) -> Result<()> {
        let now = Utc::now();
        let detection = DeviceDetection {
            timestamp: now,
            latitude: scan_data.latitude,
            longitude: scan_data.longitude,
            rssi: scan_data.rssi,
            tx_power: scan_data.tx_power,
            manufacturer_data: scan_data.manufacturer_data.clone(),
        };

        // Check if the device exists in the database
        let mut stmt = self.conn.prepare("SELECT name FROM devices WHERE address = ?")?;
        let device_name: Option<String> = stmt
            .query_row(params![scan_data.address], |row| row.get(0))
            .optional()?;

        // If device doesn't exist, insert it into the devices table
        if device_name.is_none() {
            self.conn.execute(
                "INSERT INTO devices (address, name) VALUES (?1, ?2)",
                params![
                    scan_data.address,
                    scan_data.name.clone().unwrap_or_else(|| "Unknown".to_string())
                ],
            )?;
        }

        // Insert the detection into the detections table
        self.conn.execute(
            "INSERT INTO detections (device_address, timestamp, latitude, longitude, rssi, tx_power, manufacturer_data) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                scan_data.address,
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

    // Changed &self to &mut self to allow mutable access for the transaction.
    pub fn store_scan_data_batch(&mut self, scan_data_list: &Vec<DeviceScanData>) -> Result<()> {
        let transaction = self.conn.transaction()?;
        
        for scan_data in scan_data_list {
            let now = Utc::now();
            let detection = DeviceDetection {
                timestamp: now,
                latitude: scan_data.latitude,
                longitude: scan_data.longitude,
                rssi: scan_data.rssi,
                tx_power: scan_data.tx_power,
                manufacturer_data: scan_data.manufacturer_data.clone(),
            };
    
            // Check if the device exists
            let mut stmt = transaction.prepare("SELECT name FROM devices WHERE address = ?")?;
            let device_name: Option<String> = stmt
                .query_row(params![scan_data.address], |row| row.get(0))
                .optional()?;
    
            // Insert device if it doesn't exist
            if device_name.is_none() {
                transaction.execute(
                    "INSERT INTO devices (address, name) VALUES (?1, ?2)",
                    params![
                        scan_data.address,
                        scan_data.name.clone().unwrap_or_else(|| "Unknown".to_string())
                    ],
                )?;
            }
    
            // Insert detection
            transaction.execute(
                "INSERT INTO detections (device_address, timestamp, latitude, longitude, rssi, tx_power, manufacturer_data) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    scan_data.address,
                    detection.timestamp.to_rfc3339(),
                    detection.latitude,
                    detection.longitude,
                    detection.rssi,
                    detection.tx_power,
                    detection.manufacturer_data
                ],
            )?;
        }
    
        transaction.commit()?;
        Ok(())
    }
    
    pub fn get_device_history(&mut self, address: &str, filters: FilterOptions) -> Result<Vec<DeviceDetection>> {
        let mut query = String::from(
            "SELECT timestamp, latitude, longitude, rssi, tx_power, manufacturer_data 
             FROM detections 
             WHERE device_address = ?"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(address.to_string())];

        if let Some(start) = filters.start_time {
            query.push_str(" AND timestamp >= ?");
            params.push(Box::new(start.to_rfc3339()));
        }
        if let Some(end) = filters.end_time {
            query.push_str(" AND timestamp <= ?");
            params.push(Box::new(end.to_rfc3339()));
        }

        query.push_str(" ORDER BY timestamp DESC");

        // Use a default limit of 50 if no limit is provided.
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
        let mut stmt = self.conn.prepare(
            "SELECT latitude, longitude 
             FROM detections 
             WHERE device_address = ? 
             ORDER BY timestamp DESC 
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![address])?;
        if let Some(row) = rows.next()? {
            let latitude: f64 = row.get(0)?;
            let longitude: f64 = row.get(1)?;
            Ok(Some((latitude, longitude)))
        } else {
            Ok(None)
        }
    }

    pub fn find_devices_near(&self, latitude: f64, longitude: f64, radius_km: f64) -> Result<Vec<String>> {
        // Bounding box filter for rough pre-selection (1-degree lat/lon ≈ 111 km)
        let lat_range = radius_km / 111.0;
        // Removed lon_range as it is not used.
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT device_address, latitude, longitude 
             FROM detections 
             WHERE latitude BETWEEN ?1 - ?3 AND ?1 + ?3 
             AND longitude BETWEEN ?2 - ?3 AND ?2 + ?3",
        )?;

        let rows = stmt.query_map(params![latitude, longitude, lat_range], |row| {
            let address: String = row.get(0)?;
            let lat: f64 = row.get(1)?;
            let lon: f64 = row.get(2)?;
            Ok((address, lat, lon))
        })?;

        let mut devices = Vec::new();
        for row in rows {
            let (address, lat, lon) = row?;
            if haversine_distance(latitude, longitude, lat, lon) <= radius_km {
                devices.push(address);
            }
        }

        Ok(devices)
    }
}
