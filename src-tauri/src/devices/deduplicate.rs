use super::models::VisualDevice;
use std::collections::BTreeMap;

pub fn merge(devices: Vec<VisualDevice>) -> Vec<VisualDevice> {
    let mut groups: BTreeMap<String, VisualDevice> = BTreeMap::new();
    for device in devices {
        let name = device.display_name.as_deref().unwrap_or("").to_lowercase();
        let family = name
            .replace(" hid", "")
            .replace(" bluetooth", "")
            .replace(" hands-free", "")
            .replace(" a2dp", "")
            .replace(" avrcp", "");
        let key = format!(
            "{}:{}:{}",
            device.category,
            family,
            device.manufacturer.as_deref().unwrap_or("")
        );
        groups
            .entry(key)
            .and_modify(|existing| existing.count += 1)
            .or_insert(device);
    }
    groups.into_values().collect()
}
