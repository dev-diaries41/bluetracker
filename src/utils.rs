use std::collections::HashMap;
use std::error::Error;
use csv::ReaderBuilder;
use regex::Regex;
use sha2::{Sha256, Digest};
use hex;

pub fn load_manufacturer_map_from_csv(csv_path: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(csv_path)?;
    let mut manufacturer_map = HashMap::new();

    for result in rdr.records() {
        let record = result?;
        let manufacturer_id = record.get(0).ok_or("Missing manufacturer ID")?.to_string(); // First column is ID
        let manufacturer_name = record.get(1).ok_or("Missing manufacturer name")?.to_string(); // Second column is name
        manufacturer_map.insert(manufacturer_id, manufacturer_name);
    }

    Ok(manufacturer_map)
}

pub fn get_manufacturer_id(manufacturer_data: &str) -> Option<u16> {
    if manufacturer_data.is_empty() {
        return None;
    }

    // Regular expression to match the key-value pair structure: "{key: [value1, value2, ...]}"
    let re = Regex::new(r"\{(\d+): \[(.*?)\]\}").unwrap();

    if let Some(captures) = re.captures(manufacturer_data) {
        if let Some(manufacturer_id_str) = captures.get(1) {
            if let Ok(manufacturer_id) = manufacturer_id_str.as_str().parse::<u16>() {
                return Some(manufacturer_id);
            }
        }
    }
    None
}

pub fn get_manufacturer_name(id: &u16, manufacturer_map: &HashMap<String, String>) -> Option<String> {
    let manufacturer_id_hex = u16_to_hex(*id);
    manufacturer_map.get(&manufacturer_id_hex).cloned()  // Return the name if found, otherwise None
}

fn u16_to_hex(value: u16) -> String {
    format!("0x{:04X}", value)
}

pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    use std::f64::consts::PI;

    const EARTH_RADIUS_KM: f64 = 6371.0;
    let to_radians = |deg: f64| deg * PI / 180.0;

    let dlat = to_radians(lat2 - lat1);
    let dlon = to_radians(lon2 - lon1);

    let a = (dlat / 2.0).sin().powi(2)
          + (to_radians(lat1).cos() * to_radians(lat2).cos() * (dlon / 2.0).sin().powi(2));
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}



/// Hashes a string (e.g., device address) using SHA-256.
pub fn hash_data(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize()) // Convert to hex string
}
