mod classify;
mod deduplicate;
mod enumerate;
mod midi;
mod models;
mod monitors;
pub mod watcher;
pub use models::DeviceSnapshot;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn snapshot() -> DeviceSnapshot {
    let raw = enumerate::enumerate();
    let raw_device_count = raw.len() as u32;
    let mut devices: Vec<_> = raw.into_iter().filter_map(classify::classify).collect();
    let filtered_device_count = devices.len() as u32;
    devices = deduplicate::merge(devices);
    devices.extend(monitors::enumerate());
    let midi_devices = midi::enumerate();
    let midi_count = midi_devices.len();
    devices.extend(midi_devices);
    devices.push(computer());
    limit_categories(&mut devices);
    devices.sort_by(|a, b| a.category.cmp(&b.category).then(a.id.cmp(&b.id)));
    let merged_physical_device_count = devices.len() as u32;
    eprintln!(
        "[devices] raw={raw_device_count} midi={midi_count} visual={merged_physical_device_count}"
    );
    DeviceSnapshot {
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

fn limit_categories(devices: &mut Vec<models::VisualDevice>) {
    use std::collections::BTreeMap;
    let limits = [
        ("display", 4),
        ("computer", 1),
        ("keyboard", 1),
        ("mouse", 1),
        ("headset", 1),
        ("speaker", 2),
        ("microphone", 1),
        ("camera", 1),
        ("phone", 1),
        ("storage", 3),
        ("gameController", 2),
        ("midiKeyboard", 1),
        ("midiController", 2),
        ("midiInterface", 1),
        ("printer", 1),
        ("usbGeneric", 1),
        ("unknown", 1),
    ];
    let map: BTreeMap<_, _> = limits.into_iter().collect();
    let mut counts: BTreeMap<String, i32> = BTreeMap::new();
    devices.retain(|device| {
        let count = counts.entry(device.category.clone()).or_default();
        *count += 1;
        *count <= *map.get(device.category.as_str()).unwrap_or(&0)
    });
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
}
