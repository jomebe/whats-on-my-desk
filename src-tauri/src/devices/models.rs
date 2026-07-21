use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualDevice {
    pub id: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    pub connection_type: String,
    pub icon_key: String,
    pub count: u32,
    pub is_external: bool,
    pub is_virtual: bool,
    pub present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_hint: Option<PositionHint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visual_variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midi: Option<MidiInfo>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSnapshot {
    pub revision: u64,
    pub source: String,
    pub generated_at: u64,
    pub raw_device_count: u32,
    pub filtered_device_count: u32,
    pub merged_physical_device_count: u32,
    pub devices: Vec<VisualDevice>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionHint {
    pub x: i32,
    pub y: i32,
    pub primary: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MidiInfo {
    pub has_input: bool,
    pub has_output: bool,
    pub port_count: u32,
}

pub struct RawDevice {
    pub stable_key: String,
    pub name: String,
    pub physical_name: String,
    pub manufacturer: String,
    pub class_name: String,
    pub instance_id: String,
    pub metadata_text: String,
    pub connection_type: String,
    pub is_external: bool,
}
