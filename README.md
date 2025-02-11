# BlueTracker CLI Tool

## Table of Contents
1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Usage](#usage)
   - [Scan for Devices](#scan-for-devices)
   - [Connect to a Device](#connect-to-a-device)
   - [Get Last Known Location](#get-last-known-location)
   - [Find Nearby Devices](#find-nearby-devices)
   - [Get Device History](#get-device-history)
   - [Find Manufacturer Name](#find-manufacturer-name)
4. [Global Tracking & GPS Integration](#global-tracking--gps-integration)
5. [Database](#database)
6. [Dependencies](#dependencies)
7. [License](#license)

## Introduction
The BlueTracker CLI tool allows users to scan for Bluetooth devices, connect to them, track their location, and retrieve history data. The tool integrates SQLite for persistent storage of Bluetooth device detections.

## Installation
1. Clone the repository:
   ```sh
   git clone https://github.com/your-repo/bluetracker-cli-tool.git
   cd bluetracker-cli-tool
   ```
2. Install dependencies:
   ```sh
   cargo build --release
   ```
3. Run the tool:
   ```sh
   ./target/release/bluetracker --help
   ```

## Usage
The CLI supports multiple commands:

### Scan for Devices
Scan for Bluetooth devices and optionally store the results in a file or database.
```sh
bluetracker scan --output devices.json --db-path bluetooth.db --latitude 37.7749 --longitude -122.4194
```

### Connect to a Device
Connect to a specific Bluetooth device by its address.
```sh
bluetracker connect 00:1A:7D:DA:71:13
```

### Get Last Known Location
Retrieve the last known location of a Bluetooth device.
```sh
bluetracker location --address 00:1A:7D:DA:71:13 --db-path bluetooth.db
```

### Find Nearby Devices
Find Bluetooth devices within a specific radius.
```sh
bluetracker nearby --latitude 37.7749 --longitude -122.4194 --radius 5 --db-path bluetooth.db
```

### Get Device History
Retrieve the detection history of a Bluetooth device.
```sh
bluetracker history --address 00:1A:7D:DA:71:13 --db-path bluetooth.db --start-time "2024-01-01T00:00:00Z" --end-time "2024-01-31T23:59:59Z" --limit 50
```

### Find Manufacturer Name
Find the manufacturer name from an ID.
```sh
bluetracker brand --id 1234
```

## Global Tracking & GPS Integration
To enable **global tracking**, BlueTracker includes **GPS coordinates** with each scan, allowing:

✅ Tracking **where** devices were detected.  
✅ Identifying **movement patterns** of frequent devices.  
✅ Implementing **geo-fencing** for businesses or security use cases.  

### Example Use Cases
🔵 **Retail Stores** → Track the number of new vs. returning customers.  
🔵 **Traffic Analytics** → Monitor Bluetooth signals from cars & phones to analyze congestion.  
🔵 **Security** → Identify unknown devices in **restricted areas**.  
🔵 **Lost & Found** → Track last seen **locations** of devices over time.  

## Database
The tool uses an SQLite database to store scan results and history. The database path can be specified using `--db-path` in commands.

## Dependencies
- Rust
- tokio (for async runtime)
- clap (for command-line argument parsing)
- chrono (for date/time handling)
- SQLite (for data storage)

## License
This project is licensed under the MIT License.

