use super::models::RawDevice;

#[cfg(windows)]
pub fn enumerate() -> Vec<RawDevice> {
    use windows::{
        core::PCWSTR,
        Win32::{Devices::DeviceAndDriverInstallation::*, Foundation::GetLastError},
    };
    let mut result = Vec::new();
    unsafe {
        let set = match SetupDiGetClassDevsW(
            None,
            PCWSTR::null(),
            None,
            DIGCF_ALLCLASSES | DIGCF_PRESENT,
        ) {
            Ok(value) => value,
            Err(_) => return result,
        };
        let mut index = 0;
        loop {
            let mut info = SP_DEVINFO_DATA {
                cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                ..Default::default()
            };
            if SetupDiEnumDeviceInfo(set, index, &mut info).is_err() {
                let _ = GetLastError();
                break;
            }
            let name = property(set, &mut info, SPDRP_FRIENDLYNAME)
                .or_else(|| property(set, &mut info, SPDRP_DEVICEDESC))
                .unwrap_or_default();
            let manufacturer = property(set, &mut info, SPDRP_MFG).unwrap_or_default();
            let class_name = property(set, &mut info, SPDRP_CLASS).unwrap_or_default();
            let mut id = vec![0u16; 1024];
            let instance_id = SetupDiGetDeviceInstanceIdW(set, &info, Some(&mut id), None)
                .ok()
                .map(|_| wide(&id))
                .unwrap_or_default();
            if !name.is_empty() {
                result.push(RawDevice {
                    stable_key: instance_id.clone(),
                    name,
                    manufacturer,
                    class_name,
                    instance_id,
                });
            }
            index += 1;
        }
        let _ = SetupDiDestroyDeviceInfoList(set);
    }
    result
}

#[cfg(windows)]
unsafe fn property(
    set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    info: &mut windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
    property: windows::Win32::Devices::DeviceAndDriverInstallation::SETUP_DI_REGISTRY_PROPERTY,
) -> Option<String> {
    use windows::Win32::Devices::DeviceAndDriverInstallation::SetupDiGetDeviceRegistryPropertyW;
    let mut kind = 0u32;
    let mut bytes = vec![0u8; 2048];
    SetupDiGetDeviceRegistryPropertyW(set, info, property, Some(&mut kind), Some(&mut bytes), None)
        .ok()?;
    let wide = std::slice::from_raw_parts(bytes.as_ptr() as *const u16, bytes.len() / 2);
    Some(self::wide(wide))
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
