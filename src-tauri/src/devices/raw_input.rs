#[derive(Default)]
pub struct InputPresence {
    keyboards: Vec<String>,
    mice: Vec<String>,
}

impl InputPresence {
    pub fn contains(&self, category: &str, instance_id: &str) -> bool {
        let target = normalize(instance_id);
        let devices = if category == "keyboard" {
            &self.keyboards
        } else {
            &self.mice
        };
        devices
            .iter()
            .any(|name| normalize(name).contains(&target) || target.contains(&normalize(name)))
    }

    #[cfg(test)]
    pub fn with_keyboard(instance_id: &str) -> Self {
        Self {
            keyboards: vec![instance_id.into()],
            mice: Vec::new(),
        }
    }
}

#[cfg(windows)]
pub fn enumerate() -> InputPresence {
    use windows::Win32::UI::Input::{
        GetRawInputDeviceInfoW, GetRawInputDeviceList, RAWINPUTDEVICELIST, RIDI_DEVICENAME,
        RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
    };

    let mut count = 0u32;
    unsafe {
        if GetRawInputDeviceList(
            None,
            &mut count,
            std::mem::size_of::<RAWINPUTDEVICELIST>() as u32,
        ) == u32::MAX
        {
            return InputPresence::default();
        }
        let mut list = vec![RAWINPUTDEVICELIST::default(); count as usize];
        if GetRawInputDeviceList(
            Some(list.as_mut_ptr()),
            &mut count,
            std::mem::size_of::<RAWINPUTDEVICELIST>() as u32,
        ) == u32::MAX
        {
            return InputPresence::default();
        }
        let mut presence = InputPresence::default();
        for device in list.into_iter().take(count as usize) {
            if device.dwType != RIM_TYPEKEYBOARD && device.dwType != RIM_TYPEMOUSE {
                continue;
            }
            let mut chars = 0u32;
            if GetRawInputDeviceInfoW(Some(device.hDevice), RIDI_DEVICENAME, None, &mut chars)
                == u32::MAX
                || chars == 0
            {
                continue;
            }
            let mut name = vec![0u16; chars as usize + 1];
            if GetRawInputDeviceInfoW(
                Some(device.hDevice),
                RIDI_DEVICENAME,
                Some(name.as_mut_ptr().cast()),
                &mut chars,
            ) == u32::MAX
            {
                continue;
            }
            let name = String::from_utf16_lossy(&name[..chars as usize])
                .trim_end_matches('\0')
                .to_string();
            if device.dwType == RIM_TYPEKEYBOARD {
                presence.keyboards.push(name);
            } else {
                presence.mice.push(name);
            }
        }
        presence
    }
}

#[cfg(not(windows))]
pub fn enumerate() -> InputPresence {
    InputPresence::default()
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .replace("\\\\?\\", "")
        .replace('#', "\\")
        .replace("&col", "")
}
