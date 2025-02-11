use std::error::Error;
use clap::Parser;
use chrono::{DateTime, Utc};

mod db;
mod scan;
mod connect;
mod utils;

#[derive(Parser, Debug)]
#[command(version = "1.0", about = "Bluetooth CLI tool to scan, connect and track devices plus location")]
struct Args {
    /// Command to perform (scan, connect, location, nearby, history)
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Scan for Bluetooth devices
    Scan {
        /// Path to save the list of found devices (optional)
        #[arg(short, long)]
        output: Option<String>,

        /// Store in SQLite database
        #[arg(short = 'd', long, action = clap::ArgAction::SetTrue)]
        use_db: bool,

        /// Latitude coordinate (optional)
        #[arg(long)]
        latitude: Option<f64>,

        /// Longitude coordinate (optional)
        #[arg(long)]
        longitude: Option<f64>,
    },

    /// Connect to a Bluetooth device by address
    Connect {
        /// Bluetooth address of the device to connect to
        address: String,
    },

    /// Get the last known location of a device
    Location {
        /// Bluetooth address of the device
        address: String,
    },

    /// Find nearby devices within a given radius
    Nearby {
        /// Latitude coordinate
        latitude: f64,

        /// Longitude coordinate
        longitude: f64,

        /// Radius in kilometers
        radius: f64,
    },

    /// Get the detection history of a device
    History {
        /// Bluetooth address of the device
        address: String,
        
        /// Start timestamp (RFC3339 format)
        #[arg(short, long)]
        start_time: Option<String>,

        /// End timestamp (RFC3339 format)
        #[arg(short, long)]
        end_time: Option<String>,

        /// Limit the number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },

     /// Get the detection history of a device
     Devices {
        /// Start timestamp (RFC3339 format)
        #[arg(short, long)]
        start_time: Option<String>,

        /// End timestamp (RFC3339 format)
        #[arg(short, long)]
        end_time: Option<String>,

        /// Limit the number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },

       /// Find the manufacturer name
       Brand {
        /// Manufacture id
        id: u16,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let db_path = db::get_db_path(None);
    let mut db = db::BluetoothTracker::new(&db_path)?;

    match args.command {
        Command::Scan {
            output,
            use_db,
            latitude,
            longitude,
        } => {
            let scan_options = scan::ScanOptions {
                outpath: output,
                use_db,
                latitude,
                longitude,
            };

            match scan::scan_devices(scan_options).await {
                Ok(devices) => {
                    println!("Scan completed. Found {} devices.", devices.len());
                }
                Err(e) => {
                    eprintln!("Error during scan: {}", e);
                }
            }
        }

        Command::Connect { address } => {
            match connect::connect_to_device(&address).await {
                Ok(_) => println!("Successfully connected to device: {}", address),
                Err(e) => eprintln!("Error connecting to device {}: {}", address, e),
            }
        }

        Command::Location { address } => {
           
            match db.estimate_device_location(&address)? {
                Some((lat, lon)) => println!("Last known location: ({}, {})", lat, lon),
                None => println!("No location data found for device."),
            }
        }

        Command::Nearby { latitude, longitude, radius } => {
            let devices = db.find_devices_near(latitude, longitude, radius)?;
            if devices.is_empty() {
                println!("No devices found within {} km.", radius);
            } else {
                println!("Devices found within {} km:", radius);
                for device in devices {
                    println!("- {}", device);
                }
            }
        }

        Command::History {
            address,
            start_time,
            end_time,
            limit,
        } => {
            let filters = db::FilterOptions {
                start_time: start_time
                    .as_deref()
                    .map(|s| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc)))
                    .flatten(),
                end_time: end_time
                    .as_deref()
                    .map(|s| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc)))
                    .flatten(),
                limit,
            };

            let history = db.get_device_history(&address, filters)?;
            if history.is_empty() {
                println!("No history found for device.");
            } else {
                println!("Detection history for {}:", address);
                for detection in history {
                    println!("- Time: {}, Location: ({:?}, {:?}), RSSI: {}, Tx Power: {}, Manufacturer Data: {}",
                        detection.timestamp,
                        detection.latitude,
                        detection.longitude,
                        detection.rssi,
                        detection.tx_power,
                        detection.manufacturer_data
                    );
                }
            }
        }


        Command::Devices {
            start_time,
            end_time,
            limit,
        } => {
            let filters = db::FilterOptions {
                start_time: start_time
                    .as_deref()
                    .map(|s| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc)))
                    .flatten(),
                end_time: end_time
                    .as_deref()
                    .map(|s| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc)))
                    .flatten(),
                limit,
            };

            let devices = db.get_devices(filters)?;
            if devices.is_empty() {
                println!("No devices found.");
            } else {
                println!("Stored devices for {}:",  devices.len());
                for device in devices {
                    println!("- Address: {}, Name: {}",
                        device.address,
                        device.name,
                    );
                }
            }
        }

        Command::Brand { id } => {
            let manufacturer_map = utils::load_manufacturer_map_from_csv("/home/fpf/dev/modules/bluetracker/assets/manufacturer_names.csv")?;
            match utils::get_manufacturer_name(&id, &manufacturer_map) {
                Some(name) => println!("Manufacturer Name: {}", name),
                None => println!("Manufacturer Name not found."),
            }        
        }
    }

    Ok(())
}
