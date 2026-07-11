mod classify; mod deduplicate; mod enumerate; mod models; mod monitors;
pub use models::DeviceSnapshot;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn snapshot() -> DeviceSnapshot {
    let mut devices: Vec<_> = enumerate::enumerate().into_iter().filter_map(classify::classify).collect();
    devices = deduplicate::merge(devices);
    devices.extend(monitors::enumerate());
    devices.sort_by(|a,b| a.category.cmp(&b.category).then(a.id.cmp(&b.id)));
    DeviceSnapshot { generated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64, devices }
}

#[cfg(test)]
mod tests { use super::*; #[test] fn snapshot_has_unique_ids() { let data = snapshot(); let mut ids: Vec<_> = data.devices.iter().map(|d| &d.id).collect(); ids.sort(); ids.dedup(); assert_eq!(ids.len(), data.devices.len()); } }
