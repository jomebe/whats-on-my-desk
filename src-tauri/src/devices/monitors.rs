use super::models::{PositionHint, VisualDevice};
use sha2::{Digest, Sha256};

#[cfg(windows)]
pub fn enumerate() -> Vec<VisualDevice> {
    use windows::{
        core::BOOL,
        Win32::{
            Foundation::{LPARAM, RECT},
            Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR},
        },
    };
    let mut monitors: Vec<RECT> = Vec::new();
    unsafe extern "system" fn callback(_: HMONITOR, _: HDC, rect: *mut RECT, data: LPARAM) -> BOOL {
        if let Some(rect) = unsafe { rect.as_ref() } {
            unsafe { (&mut *(data.0 as *mut Vec<RECT>)).push(*rect) };
        }
        true.into()
    }
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(callback),
            LPARAM((&mut monitors as *mut Vec<RECT>) as isize),
        );
    }
    monitors
        .into_iter()
        .take(4)
        .enumerate()
        .map(|(index, rect)| {
            let mut hasher = Sha256::new();
            hasher.update(format!("{}:{}:{}", rect.left, rect.top, index));
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
                    x: rect.left,
                    y: rect.top,
                    primary: rect.left == 0 && rect.top == 0,
                }),
                visual_variant: Some("monitor".into()),
                midi: None,
            }
        })
        .collect()
}

#[cfg(not(windows))]
pub fn enumerate() -> Vec<VisualDevice> {
    vec![]
}
