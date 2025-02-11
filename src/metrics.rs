use std::collections::HashSet;
use chrono::Duration;

pub fn count_unique_and_returning_devices(devices: &[DeviceEntry]) -> (usize, usize) {
    let mut unique_devices = HashSet::new();
    let mut returning_devices = 0;

    for device in devices {
        if unique_devices.contains(&device.address) {
            returning_devices += 1;
        } else {
            unique_devices.insert(device.address.clone());
        }
    }

    (unique_devices.len(), returning_devices)
}

pub fn get_visit_frequency(device: &DeviceEntry) -> usize {
    device.detections.len()
}



pub fn get_peak_visit_hour(devices: &[DeviceEntry]) -> Option<u32> {
    let mut hour_counts = HashMap::new();

    for device in devices {
        for detection in &device.detections {
            let hour = detection.timestamp.hour();
            *hour_counts.entry(hour).or_insert(0) += 1;
        }
    }

    hour_counts.into_iter().max_by_key(|&(_, count)| count).map(|(hour, _)| hour)
}

pub fn calculate_average_stay_duration(devices: &[DeviceEntry]) -> Option<Duration> {
    let mut total_duration = Duration::zero();
    let mut count = 0;

    for device in devices {
        if let (Some(first), Some(last)) = (device.detections.first(), device.detections.last()) {
            total_duration = total_duration + (last.timestamp - first.timestamp);
            count += 1;
        }
    }

    if count > 0 {
        Some(total_duration / count)
    } else {
        None
    }
}
