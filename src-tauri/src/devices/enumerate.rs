use super::models::RawDevice;

#[cfg(windows)]
#[derive(Clone)]
struct DeviceNode {
    name: String,
    manufacturer: String,
    class_name: String,
    instance_id: String,
    parent_id: String,
    bus_reported_name: String,
    container_id: Option<String>,
    in_local_machine_container: Option<bool>,
}

#[cfg(windows)]
pub fn enumerate() -> Vec<RawDevice> {
    use std::collections::HashMap;
    use windows::{
        core::PCWSTR,
        Win32::Devices::{DeviceAndDriverInstallation::*, Properties::*},
    };

    let mut nodes = Vec::new();
    unsafe {
        let set = match SetupDiGetClassDevsW(
            None,
            PCWSTR::null(),
            None,
            DIGCF_ALLCLASSES | DIGCF_PRESENT,
        ) {
            Ok(value) => value,
            Err(_) => return Vec::new(),
        };
        let mut index = 0;
        loop {
            let mut info = SP_DEVINFO_DATA {
                cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                ..Default::default()
            };
            if SetupDiEnumDeviceInfo(set, index, &mut info).is_err() {
                break;
            }
            let name = registry_property(set, &mut info, SPDRP_FRIENDLYNAME)
                .or_else(|| registry_property(set, &mut info, SPDRP_DEVICEDESC))
                .unwrap_or_default();
            let manufacturer = registry_property(set, &mut info, SPDRP_MFG).unwrap_or_default();
            let class_name = registry_property(set, &mut info, SPDRP_CLASS).unwrap_or_default();
            let mut id = vec![0u16; 1024];
            let instance_id = SetupDiGetDeviceInstanceIdW(set, &info, Some(&mut id), None)
                .ok()
                .map(|_| wide(&id))
                .unwrap_or_default();
            if !name.is_empty() && !instance_id.is_empty() {
                let bus_reported_name = if useful_name(&name) {
                    String::new()
                } else {
                    string_property(set, &info, &DEVPKEY_Device_BusReportedDeviceDesc)
                        .unwrap_or_default()
                };
                nodes.push(DeviceNode {
                    name,
                    manufacturer,
                    class_name,
                    instance_id,
                    parent_id: string_property(set, &info, &DEVPKEY_Device_Parent)
                        .unwrap_or_default(),
                    bus_reported_name,
                    container_id: guid_property(set, &info, &DEVPKEY_Device_ContainerId),
                    in_local_machine_container: bool_property(
                        set,
                        &info,
                        &DEVPKEY_Device_InLocalMachineContainer,
                    ),
                });
            }
            index += 1;
        }
        let _ = SetupDiDestroyDeviceInfoList(set);
    }

    let lookup: HashMap<_, _> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (normalize_id(&node.instance_id), index))
        .collect();

    nodes
        .iter()
        .map(|node| enrich(node, &nodes, &lookup))
        .collect()
}

#[cfg(windows)]
fn enrich(
    node: &DeviceNode,
    nodes: &[DeviceNode],
    lookup: &std::collections::HashMap<String, usize>,
) -> RawDevice {
    let lineage = lineage(node, nodes, lookup);
    let container_id = node
        .container_id
        .clone()
        .or_else(|| lineage.iter().find_map(|item| item.container_id.clone()));
    let physical_nodes: Vec<_> = if let Some(container_id) = container_id.as_deref() {
        lineage
            .iter()
            .copied()
            .filter(|item| item.container_id.as_deref() == Some(container_id))
            .collect()
    } else {
        vec![node]
    };
    let physical_name = physical_nodes
        .iter()
        .map(|item| item.bus_reported_name.as_str())
        .find(|name| useful_name(name))
        .or_else(|| {
            physical_nodes
                .iter()
                .map(|item| item.name.as_str())
                .find(|name| useful_name(name))
        })
        .unwrap_or(&node.name)
        .to_string();
    let manufacturer = physical_nodes
        .iter()
        .map(|item| item.manufacturer.as_str())
        .find(|name| useful_manufacturer(name))
        .unwrap_or(&node.manufacturer)
        .to_string();
    let connection_type = infer_connection(&lineage);
    let is_external = infer_external(&physical_nodes, &connection_type);
    let metadata_text = lineage
        .iter()
        .flat_map(|item| {
            [
                item.name.as_str(),
                item.bus_reported_name.as_str(),
                item.class_name.as_str(),
                item.instance_id.as_str(),
            ]
        })
        .collect::<Vec<_>>()
        .join(" ");

    RawDevice {
        stable_key: container_id.unwrap_or_else(|| node.instance_id.clone()),
        name: node.name.clone(),
        physical_name,
        manufacturer,
        class_name: node.class_name.clone(),
        instance_id: node.instance_id.clone(),
        metadata_text,
        connection_type,
        is_external,
    }
}

#[cfg(windows)]
fn lineage<'a>(
    start: &'a DeviceNode,
    nodes: &'a [DeviceNode],
    lookup: &std::collections::HashMap<String, usize>,
) -> Vec<&'a DeviceNode> {
    use std::collections::HashSet;
    let mut result = Vec::new();
    let mut current = Some(start);
    let mut seen = HashSet::new();
    while let Some(node) = current {
        if result.len() >= 16 || !seen.insert(normalize_id(&node.instance_id)) {
            break;
        }
        result.push(node);
        current = lookup
            .get(&normalize_id(&node.parent_id))
            .and_then(|index| nodes.get(*index));
    }
    result
}

#[cfg(windows)]
fn infer_connection(lineage: &[&DeviceNode]) -> String {
    let text = lineage
        .iter()
        .flat_map(|node| [node.instance_id.as_str(), node.class_name.as_str()])
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    if text.contains("bthenum\\") || text.contains("bthledevice\\") || text.contains(" bluetooth") {
        "Bluetooth".into()
    } else if text.contains("usb\\") || text.contains("usb4\\") || text.contains(" usb") {
        "USB".into()
    } else if text.contains("swd\\printenum")
        || text.contains("swd\\ipp")
        || text.contains("dafwsd")
        || text.contains(" wsd")
    {
        "Network".into()
    } else if lineage
        .iter()
        .any(|node| node.in_local_machine_container == Some(true))
    {
        "BuiltIn".into()
    } else {
        "Unknown".into()
    }
}

#[cfg(windows)]
fn infer_external(nodes: &[&DeviceNode], connection_type: &str) -> bool {
    if nodes
        .iter()
        .any(|node| node.in_local_machine_container == Some(false))
    {
        return true;
    }
    if nodes
        .iter()
        .any(|node| node.in_local_machine_container == Some(true))
    {
        return false;
    }
    matches!(connection_type, "USB" | "Bluetooth" | "Network")
        && nodes.iter().any(|node| node.container_id.is_some())
}

#[cfg(windows)]
fn useful_name(name: &str) -> bool {
    let name = name.trim().to_lowercase();
    !name.is_empty()
        && ![
            "usb composite device",
            "usb input device",
            "hid-compliant device",
            "hid 규격 장치",
            "software device",
        ]
        .iter()
        .any(|generic| name == *generic)
}

#[cfg(windows)]
fn useful_manufacturer(name: &str) -> bool {
    let name = name.trim().to_lowercase();
    !name.is_empty()
        && !name.contains("standard usb")
        && !name.contains("generic usb")
        && name != "microsoft"
}

#[cfg(windows)]
fn normalize_id(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

#[cfg(windows)]
unsafe fn registry_property(
    set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    info: &mut windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
    property: windows::Win32::Devices::DeviceAndDriverInstallation::SETUP_DI_REGISTRY_PROPERTY,
) -> Option<String> {
    use windows::Win32::Devices::DeviceAndDriverInstallation::SetupDiGetDeviceRegistryPropertyW;
    let mut kind = 0u32;
    let mut bytes = vec![0u8; 2048];
    SetupDiGetDeviceRegistryPropertyW(set, info, property, Some(&mut kind), Some(&mut bytes), None)
        .ok()?;
    Some(wide_bytes(&bytes))
}

#[cfg(windows)]
unsafe fn device_property(
    set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    info: &windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
    key: &windows::Win32::Foundation::DEVPROPKEY,
) -> Option<Vec<u8>> {
    use windows::Win32::Devices::DeviceAndDriverInstallation::SetupDiGetDevicePropertyW;
    let mut kind = windows::Win32::Devices::Properties::DEVPROPTYPE(0);
    let mut required = 0u32;
    let mut bytes = vec![0u8; 2048];
    SetupDiGetDevicePropertyW(
        set,
        info,
        key,
        &mut kind,
        Some(&mut bytes),
        Some(&mut required),
        0,
    )
    .ok()?;
    bytes.truncate(required as usize);
    Some(bytes)
}

#[cfg(windows)]
unsafe fn string_property(
    set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    info: &windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
    key: &windows::Win32::Foundation::DEVPROPKEY,
) -> Option<String> {
    let value = wide_bytes(&device_property(set, info, key)?);
    (!value.is_empty()).then_some(value)
}

#[cfg(windows)]
unsafe fn guid_property(
    set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    info: &windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
    key: &windows::Win32::Foundation::DEVPROPKEY,
) -> Option<String> {
    let bytes = device_property(set, info, key)?;
    if bytes.len() < std::mem::size_of::<windows::core::GUID>() {
        return None;
    }
    let guid = std::ptr::read_unaligned(bytes.as_ptr().cast::<windows::core::GUID>());
    let value = guid.to_u128();
    if value == 0 || value == 0x0000000000000000ffffffffffffffff {
        None
    } else {
        Some(format!("{value:032x}"))
    }
}

#[cfg(windows)]
unsafe fn bool_property(
    set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    info: &windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
    key: &windows::Win32::Foundation::DEVPROPKEY,
) -> Option<bool> {
    device_property(set, info, key)?
        .first()
        .copied()
        .map(|value| value != 0)
}

#[cfg(windows)]
fn wide_bytes(value: &[u8]) -> String {
    let units: Vec<_> = value
        .chunks_exact(2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
        .collect();
    wide(&units)
}

#[cfg(windows)]
fn wide(value: &[u16]) -> String {
    String::from_utf16_lossy(&value[..value.iter().position(|v| *v == 0).unwrap_or(value.len())])
        .trim()
        .to_string()
}

#[cfg(not(windows))]
pub fn enumerate() -> Vec<RawDevice> {
    vec![]
}
