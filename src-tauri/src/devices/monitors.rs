use super::models::VisualDevice;
use sha2::{Digest, Sha256};

#[cfg(windows)]
pub fn enumerate() -> Vec<VisualDevice> {
    use windows::{core::BOOL, Win32::{Foundation::{LPARAM, RECT}, Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR}}};
    unsafe extern "system" fn callback(_: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL { *(data.0 as *mut u32) += 1; true.into() }
    let mut count = 0u32;
    unsafe { let _ = EnumDisplayMonitors(None, None, Some(callback), LPARAM((&mut count as *mut u32) as isize)); }
    if count == 0 { return vec![]; }
    let mut hasher = Sha256::new(); hasher.update(b"active-displays");
    vec![VisualDevice { id: format!("{:x}", hasher.finalize())[..16].into(), category: "display".into(), display_name: Some(if count == 1 { "Active display".into() } else { format!("{count} active displays") }), manufacturer: None, connection_type: "Unknown".into(), icon_key: "display".into(), count, is_external: None, is_virtual: Some(false), present: true }]
}

#[cfg(not(windows))]
pub fn enumerate() -> Vec<VisualDevice> { vec![] }
