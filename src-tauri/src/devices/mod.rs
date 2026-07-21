mod classify;
mod deduplicate;
mod enumerate;
mod midi;
mod models;
mod monitors;
mod raw_input;
pub mod watcher;
pub use models::DeviceSnapshot;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn snapshot() -> DeviceSnapshot {
    let raw = enumerate::enumerate();
    let raw_input = raw_input::enumerate();
    let raw_device_count = raw.len() as u32;
    let mut devices: Vec<_> = raw
        .into_iter()
        .filter_map(|device| classify::classify(device, &raw_input))
        .collect();
    let filtered_device_count = devices.len() as u32;
    devices = merge_physical_devices(devices);
    devices = deduplicate::merge(devices);
    devices.extend(monitors::enumerate());
    let midi_devices = midi::enumerate();
    let midi_count = midi_devices.len();
    devices.extend(midi_devices);
    devices.push(computer());
    devices.sort_by(|a, b| a.category.cmp(&b.category).then(a.id.cmp(&b.id)));
    let merged_physical_device_count = devices.len() as u32;
    eprintln!(
        "[devices] raw={raw_device_count} midi={midi_count} visual={merged_physical_device_count}"
    );
    DeviceSnapshot {
        revision: 0,
        source: "agent".into(),
        generated_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        raw_device_count,
        filtered_device_count,
        merged_physical_device_count,
        devices,
    }
}

fn computer() -> models::VisualDevice {
    models::VisualDevice {
        id: "this-computer".into(),
        category: "computer".into(),
        display_name: Some("This computer".into()),
        manufacturer: None,
        connection_type: "BuiltIn".into(),
        icon_key: "computer".into(),
        count: 1,
        is_external: false,
        is_virtual: false,
        present: true,
        position_hint: None,
        visual_variant: Some("desktop".into()),
        midi: None,
    }
}

fn merge_physical_devices(devices: Vec<models::VisualDevice>) -> Vec<models::VisualDevice> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<models::VisualDevice>> = BTreeMap::new();
    for device in devices {
        groups.entry(device.id.clone()).or_default().push(device);
    }
    groups
        .into_values()
        .map(|group| {
            let mut category_counts: BTreeMap<String, usize> = BTreeMap::new();
            for device in &group {
                *category_counts.entry(device.category.clone()).or_default() += 1;
            }
            let best_category = category_counts
                .into_iter()
                .max_by_key(|(category, count)| {
                    (
                        category_quality(category),
                        *count,
                        category_tie_break(category),
                    )
                })
                .map(|(category, _)| category)
                .unwrap_or_default();
            let mut chosen = group
                .iter()
                .find(|device| device.category == best_category)
                .cloned()
                .unwrap_or_else(|| group[0].clone());
            chosen.count = 1;
            chosen.is_external = group.iter().any(|device| device.is_external);
            if let Some(connection) = group
                .iter()
                .map(|device| device.connection_type.as_str())
                .find(|connection| !matches!(*connection, "Unknown" | "BuiltIn"))
            {
                chosen.connection_type = connection.into();
            }
            if chosen.manufacturer.is_none() {
                chosen.manufacturer = group.iter().find_map(|device| device.manufacturer.clone());
            }
            chosen
        })
        .collect()
}

fn category_quality(category: &str) -> u8 {
    match category {
        "midiKeyboard" | "midiController" | "midiInterface" => 12,
        "storage" | "camera" | "gameController" => 11,
        "headset" | "phone" | "printer" => 10,
        "keyboard" | "mouse" => 9,
        "speaker" | "microphone" => 8,
        "usbGeneric" => 2,
        "unknown" => 1,
        _ => 5,
    }
}

fn category_tie_break(category: &str) -> u8 {
    match category {
        "mouse" => 2,
        "keyboard" => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_has_unique_ids() {
        let data = snapshot();
        let mut ids: Vec<_> = data.devices.iter().map(|d| &d.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), data.devices.len());
    }

    #[test]
    fn physical_container_prefers_specific_device_over_generic_usb() {
        let generic = test_device("same", "usbGeneric");
        let headset = test_device("same", "headset");
        let merged = merge_physical_devices(vec![generic, headset]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].category, "headset");
    }

    #[test]
    fn physical_container_prefers_external_connection() {
        let mut built_in = test_device("same", "storage");
        built_in.connection_type = "BuiltIn".into();
        let usb = test_device("same", "storage");
        let merged = merge_physical_devices(vec![built_in, usb]);
        assert_eq!(merged[0].connection_type, "USB");
    }

    fn test_device(id: &str, category: &str) -> models::VisualDevice {
        models::VisualDevice {
            id: id.into(),
            category: category.into(),
            display_name: Some("Physical device".into()),
            manufacturer: None,
            connection_type: "USB".into(),
            icon_key: category.into(),
            count: 1,
            is_external: true,
            is_virtual: false,
            present: true,
            position_hint: None,
            visual_variant: None,
            midi: None,
        }
    }
}
