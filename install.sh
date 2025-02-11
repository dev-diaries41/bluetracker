#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

echo "Checking for required dependencies..."

# Check for SQLite
if ! command -v sqlite3 &>/dev/null; then
    echo "Error: SQLite is not installed. Please install it and try again."
    exit 1
fi

# Check for Cargo (Rust package manager)
if ! command -v cargo &>/dev/null; then
    echo "Error: Cargo is not installed. Please install Rust and Cargo."
    exit 1
fi

echo "Building BlueTracker..."
cargo build --release

echo "Moving binary to /usr/local/bin..."
sudo mv target/release/bluetracker /usr/local/bin/blet

echo "Setting executable permissions..."
sudo chmod 755 /usr/local/bin/blet

echo "Installation complete! You can now run 'blet' from anywhere."
