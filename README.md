# BlueTracker CLI Tool

## Table of Contents
1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Usage](#usage)
4. [Global Tracking & GPS Integration](#global-tracking--gps-integration)
5. [Database](#database)
6. [License](#license)

## Introduction
The BlueTracker CLI tool allows users to scan for Bluetooth devices, connect to them, track their location, and retrieve history data. The tool integrates SQLite for persistent storage of Bluetooth device detections.
Here's the updated **Installation** section with the quick installation method included:

---

## Installation

### Option 1: Manual Installation
1. Clone the repository:
   ```sh
   git clone https://github.com/dev-diaries41/bluetracker.git
   cd bluetracker
   ```
2. Install dependencies:
   ```sh
   cargo build --release
   ```
3. Run the tool:
   ```sh
   ./target/release/bluetracker --help
   ```

### Option 2: Quick Installation (Using Install Script)
1. Clone the repository:
   ```sh
   git clone https://github.com/dev-diaries41/bluetracker.git
   cd bluetracker
   ```
2. Make the install script executable:
   ```sh
   chmod 755 install.sh
   ```
3. Run the install script:
   ```sh
   ./install.sh
   ```
4. Run the tool:
   ```sh
   blet --help
   ```

---

## Usage

**`blet <COMMAND>`**

### Commands:
- **scan**: Scan for Bluetooth devices
- **connect**: Connect to a Bluetooth device by address
- **location**: Get the last known location of a device
- **nearby**: Find nearby devices within a given radius
- **history**: Get the detection history of a device
- **devices**: Get the detection history of a device
- **brand**: Find the manufacturer name
- **help**: Print this message or the help of the given subcommand(s)

### Options:
- `-h`, `--help`: Print help
- `-V`, `--version`: Print version

---

## Global Tracking & GPS Integration
To enable **global tracking**, BlueTracker includes **GPS coordinates** with each scan, allowing:

✅ Tracking **where** devices were detected.  
✅ Identifying **movement patterns** of frequent devices.  
✅ Implementing **geo-fencing** for businesses or security use cases.  

---

### Example Use Cases
🔵 **Retail Stores** → Track the number of new vs. returning customers.  
🔵 **Traffic Analytics** → Monitor Bluetooth signals from cars & phones to analyze congestion.  
🔵 **Security** → Identify unknown devices in **restricted areas**.  
🔵 **Lost & Found** → Track last seen **locations** of devices over time.  

---


## Database
The tool uses an SQLite database to store scan results and history.

## License
This project is licensed under the MIT License.

