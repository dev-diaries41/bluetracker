use btleplug::api::{Peripheral, Central, Manager};
use btleplug::platform::Manager as PlatformManager;
use std::error::Error;
use std::thread;
use std::time::Duration;

pub async fn connect_to_device(address: &str) -> Result<(), Box<dyn Error>> {
    // Create the Bluetooth manager and find the first available adapter
    let manager = PlatformManager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = match adapters.first() {
        Some(adapter) => adapter.clone(),
        None => {
            eprintln!("No Bluetooth adapters found.");
            return Err("No Bluetooth adapters found.".into());
        }
    };

    // Find the peripheral with the given address
    let peripherals = adapter.peripherals().await?;
    let peripheral = peripherals.into_iter().find(|p| p.address().to_string() == address);

    match peripheral {
        Some(device) => {
            println!("Attempting to connect to device: {}", device.address());

            // Retry mechanism with a delay for transient errors
            let mut attempts = 0;
            let max_attempts = 3;
            let retry_delay = Duration::from_secs(2); // 2-second delay between retries

            while attempts < max_attempts {
                match device.connect().await {
                    Ok(_) => {
                        println!("Successfully connected to {}", device.address());
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to {}: {}", device.address(), e);

                        // Check if the error is a connection refused error (e.g., br-connection-refused)
                        if e.to_string().contains("br-connection-refused") {
                            eprintln!("Connection refused. The device may require manual pairing or authorization.");
                        } else {
                            eprintln!("Connection attempt failed for another reason.");
                        }

                        attempts += 1;
                        if attempts < max_attempts {
                            eprintln!("Retrying connection attempt {} of {}...", attempts, max_attempts);
                            thread::sleep(retry_delay); // Wait before retrying
                        }
                    }
                }
            }

            eprintln!("Failed to connect to device {} after {} attempts", device.address(), max_attempts);
            Err("Max connection attempts reached.".into())
        }
        None => {
            eprintln!("Device with address {} not found.", address);
            Err("Device not found.".into())
        }
    }
}
