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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_external: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_virtual: Option<bool>,
    pub present: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSnapshot { pub generated_at: u64, pub devices: Vec<VisualDevice> }

pub struct RawDevice { pub stable_key: String, pub name: String, pub manufacturer: String, pub class_name: String, pub instance_id: String }
