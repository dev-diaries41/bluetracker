use serde_json;

use btleplug::api::{Central, Peripheral, ScanFilter, Manager};
use btleplug::platform::Manager as PlatformManager;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::Local;

use crate::db::{BluetoothTracker, DeviceScanData, get_db_path};

pub struct ScanOptions {
    pub outpath: Option<String>,  // File output path (optional)
    pub use_db: bool,  // Use database if true
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

pub async fn scan_devices(options: ScanOptions) -> Result<Vec<DeviceScanData>, Box<dyn Error>> {
    let manager = PlatformManager::new().await?;
    let adapters = manager.adapters().await?;
    let central = match adapters.first() {
        Some(adapter) => adapter.clone(),
        None => {
            eprintln!("No Bluetooth adapters found.");
            return Ok(vec![]);
        }
    };

    println!("Scanning for Bluetooth devices...");
    central.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    let devices = central.peripherals().await?;
    let mut device_list = Vec::new();

    // Only initialize the database if use_db is true
    let db = if options.use_db {
        Some(BluetoothTracker::new(&get_db_path(None))?)
    } else {
        None
    };

    if devices.is_empty() {
        println!("No devices found.");
    } else {
        println!("Found {} devices:", devices.len());

        for device in devices {
            let properties = device.properties().await?;
            if let Some(props) = properties {
                let device_data = DeviceScanData {
                    name: props.local_name.clone(),
                    address: device.address().to_string(),
                    rssi: props.rssi.unwrap_or(0) as i32,
                    tx_power: props.tx_power_level.unwrap_or(0) as i32,
                    manufacturer_data: if props.manufacturer_data.is_empty() {
                        "".to_string()
                    } else {
                        format!("{:?}", props.manufacturer_data)
                    },
                    latitude: options.latitude,
                    longitude: options.longitude,
                };

                println!("{:?}", device_data);
                device_list.push(device_data.clone());
            }
        }

        // Store scan results in the database only if it's enabled
        if let Some(mut db) = db {
            db.store_scan_data_batch(&device_list)?;
        }

        if let Some(outpath) = options.outpath {
            save_device_list(&outpath, &device_list).await?;
        }
    }

    Ok(device_list)
}

pub async fn save_device_list(output: &str, device_list: &[DeviceScanData]) -> std::io::Result<()> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();

    let path = Path::new(output);
    let new_path: PathBuf = if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            path.with_file_name(format!("{}_{}.{}", file_stem, timestamp, ext))
        } else {
            path.with_file_name(format!("{}_{}", file_stem, timestamp))
        }
    } else {
        PathBuf::from(format!("{}_{}", output, timestamp))
    };

    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)?; 
    }

    // Serialize device list to JSON
    let serialized_data = serde_json::to_string_pretty(&device_list)?;

    let mut file = File::create(&new_path)?;
    file.write_all(serialized_data.as_bytes())?;

    println!("Device list saved to {}", new_path.display());
    Ok(())
}
