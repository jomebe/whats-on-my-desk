use super::models::{PositionHint, VisualDevice};
use sha2::{Digest, Sha256};

#[cfg(windows)]
#[derive(Clone)]
struct MonitorGeometry {
    gdi_name: String,
    left: i32,
    top: i32,
    primary: bool,
}

#[cfg(windows)]
struct ActiveTarget {
    stable_key: String,
    gdi_name: String,
    friendly_name: String,
    connection_type: &'static str,
    is_external: bool,
    is_virtual: bool,
}

#[cfg(windows)]
pub fn enumerate() -> Vec<VisualDevice> {
    use std::collections::HashSet;

    let geometries = monitor_geometries();
    let targets = active_targets();
    if targets.is_empty() {
        return geometries
            .into_iter()
            .enumerate()
            .map(|(index, geometry)| fallback_device(index, geometry))
            .collect();
    }

    let mut matched = HashSet::new();
    let mut devices: Vec<_> = targets
        .into_iter()
        .map(|target| {
            let geometry = geometries
                .iter()
                .find(|geometry| geometry.gdi_name.eq_ignore_ascii_case(&target.gdi_name));
            if geometry.is_some() {
                matched.insert(target.gdi_name.to_ascii_lowercase());
            }
            let mut hasher = Sha256::new();
            hasher.update(target.stable_key.as_bytes());
            VisualDevice {
                id: format!("{:x}", hasher.finalize())[..16].into(),
                category: "display".into(),
                display_name: (!target.friendly_name.is_empty()).then_some(target.friendly_name),
                manufacturer: None,
                connection_type: target.connection_type.into(),
                icon_key: "display".into(),
                count: 1,
                is_external: target.is_external,
                is_virtual: target.is_virtual,
                present: true,
                position_hint: Some(PositionHint {
                    x: geometry.map(|value| value.left).unwrap_or_default(),
                    y: geometry.map(|value| value.top).unwrap_or_default(),
                    primary: geometry.map(|value| value.primary).unwrap_or(false),
                }),
                visual_variant: Some("monitor".into()),
                midi: None,
            }
        })
        .collect();
    devices.extend(
        geometries
            .into_iter()
            .enumerate()
            .filter(|(_, geometry)| !matched.contains(&geometry.gdi_name.to_ascii_lowercase()))
            .map(|(index, geometry)| fallback_device(index, geometry)),
    );
    devices
}

#[cfg(windows)]
fn monitor_geometries() -> Vec<MonitorGeometry> {
    use windows::{
        core::BOOL,
        Win32::{
            Foundation::LPARAM,
            Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW},
            UI::WindowsAndMessaging::MONITORINFOF_PRIMARY,
        },
    };

    let mut monitors = Vec::new();
    unsafe extern "system" fn callback(
        monitor: HMONITOR,
        _: HDC,
        _: *mut windows::Win32::Foundation::RECT,
        data: LPARAM,
    ) -> BOOL {
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if unsafe { GetMonitorInfoW(monitor, &mut info.monitorInfo) }.as_bool() {
            unsafe {
                (&mut *(data.0 as *mut Vec<MonitorGeometry>)).push(MonitorGeometry {
                    gdi_name: wide(&info.szDevice),
                    left: info.monitorInfo.rcMonitor.left,
                    top: info.monitorInfo.rcMonitor.top,
                    primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
                })
            };
        }
        true.into()
    }
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(callback),
            LPARAM((&mut monitors as *mut Vec<MonitorGeometry>) as isize),
        );
    }
    monitors
}

#[cfg(windows)]
fn active_targets() -> Vec<ActiveTarget> {
    use windows::Win32::Devices::Display::*;

    query_active_paths()
        .into_iter()
        .filter_map(|path| unsafe {
            let mut source = DISPLAYCONFIG_SOURCE_DEVICE_NAME {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
                    size: std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32,
                    adapterId: path.sourceInfo.adapterId,
                    id: path.sourceInfo.id,
                },
                ..Default::default()
            };
            if DisplayConfigGetDeviceInfo(&mut source.header) != 0 {
                return None;
            }
            let mut target = DISPLAYCONFIG_TARGET_DEVICE_NAME {
                header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                    r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                    size: std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
                    adapterId: path.targetInfo.adapterId,
                    id: path.targetInfo.id,
                },
                ..Default::default()
            };
            if DisplayConfigGetDeviceInfo(&mut target.header) != 0 {
                return None;
            }
            let connector = connector_mapping(target.outputTechnology.0);
            let device_path = wide(&target.monitorDevicePath);
            let stable_key = if device_path.is_empty() {
                format!(
                    "display:{}:{}:{}",
                    path.targetInfo.adapterId.HighPart,
                    path.targetInfo.adapterId.LowPart,
                    path.targetInfo.id
                )
            } else {
                device_path
            };
            Some(ActiveTarget {
                stable_key,
                gdi_name: wide(&source.viewGdiDeviceName),
                friendly_name: wide(&target.monitorFriendlyDeviceName),
                connection_type: connector.connection_type,
                is_external: connector.is_external,
                is_virtual: connector.is_virtual,
            })
        })
        .collect()
}

#[cfg(windows)]
fn query_active_paths() -> Vec<windows::Win32::Devices::Display::DISPLAYCONFIG_PATH_INFO> {
    use windows::Win32::{Devices::Display::*, Foundation::*};

    for _ in 0..3 {
        let mut path_count = 0u32;
        let mut mode_count = 0u32;
        if unsafe {
            GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count)
        } != ERROR_SUCCESS
        {
            return Vec::new();
        }
        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];
        let result = unsafe {
            QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                None,
            )
        };
        if result == ERROR_INSUFFICIENT_BUFFER {
            continue;
        }
        if result != ERROR_SUCCESS {
            return Vec::new();
        }
        paths.truncate(path_count as usize);
        return paths;
    }
    Vec::new()
}

#[cfg(windows)]
fn fallback_device(index: usize, geometry: MonitorGeometry) -> VisualDevice {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{index}", geometry.gdi_name));
    VisualDevice {
        id: format!("{:x}", hasher.finalize())[..16].into(),
        category: "display".into(),
        display_name: None,
        manufacturer: None,
        connection_type: "Unknown".into(),
        icon_key: "display".into(),
        count: 1,
        is_external: true,
        is_virtual: false,
        present: true,
        position_hint: Some(PositionHint {
            x: geometry.left,
            y: geometry.top,
            primary: geometry.primary,
        }),
        visual_variant: Some("monitor".into()),
        midi: None,
    }
}

#[cfg(windows)]
struct ConnectorMapping {
    connection_type: &'static str,
    is_external: bool,
    is_virtual: bool,
}

#[cfg(windows)]
fn connector_mapping(technology: i32) -> ConnectorMapping {
    use windows::Win32::Devices::Display::*;

    if technology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI.0 {
        ConnectorMapping {
            connection_type: "HDMI",
            is_external: true,
            is_virtual: false,
        }
    } else if technology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EXTERNAL.0
        || technology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_USB_TUNNEL.0
    {
        ConnectorMapping {
            connection_type: "DisplayPort",
            is_external: true,
            is_virtual: false,
        }
    } else if technology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL.0
        || technology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_LVDS.0
        || technology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED.0
        || technology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EMBEDDED.0
    {
        ConnectorMapping {
            connection_type: "BuiltIn",
            is_external: false,
            is_virtual: false,
        }
    } else if technology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_WIRED.0 {
        ConnectorMapping {
            connection_type: "USB",
            is_external: true,
            is_virtual: false,
        }
    } else if technology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_MIRACAST.0 {
        ConnectorMapping {
            connection_type: "Network",
            is_external: true,
            is_virtual: false,
        }
    } else if technology == DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_VIRTUAL.0 {
        ConnectorMapping {
            connection_type: "Virtual",
            is_external: false,
            is_virtual: true,
        }
    } else {
        ConnectorMapping {
            connection_type: "Unknown",
            is_external: true,
            is_virtual: false,
        }
    }
}

#[cfg(windows)]
fn wide(value: &[u16]) -> String {
    String::from_utf16_lossy(&value[..value.iter().position(|x| *x == 0).unwrap_or(value.len())])
        .trim()
        .into()
}

#[cfg(not(windows))]
pub fn enumerate() -> Vec<VisualDevice> {
    vec![]
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use windows::Win32::Devices::Display::*;

    #[test]
    fn maps_hdmi_and_displayport_connectors() {
        let hdmi = connector_mapping(DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI.0);
        assert_eq!(hdmi.connection_type, "HDMI");
        assert!(hdmi.is_external);

        let display_port =
            connector_mapping(DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EXTERNAL.0);
        assert_eq!(display_port.connection_type, "DisplayPort");
        assert!(display_port.is_external);
    }

    #[test]
    fn maps_embedded_and_virtual_displays() {
        let embedded = connector_mapping(DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL.0);
        assert_eq!(embedded.connection_type, "BuiltIn");
        assert!(!embedded.is_external);

        let virtual_display = connector_mapping(DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_VIRTUAL.0);
        assert_eq!(virtual_display.connection_type, "Virtual");
        assert!(virtual_display.is_virtual);
    }
}
