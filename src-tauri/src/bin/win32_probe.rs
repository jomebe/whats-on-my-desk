#[cfg(windows)]
fn main() {
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::{
            Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL},
            WindowsAndMessaging::{
                EnumWindows, FindWindowExW, FindWindowW, GetClientRect, SendMessageTimeoutW,
                SetParent, SetWindowPos, SMTO_NORMAL,
            },
        },
    };
    let _: unsafe extern "system" fn(
        Option<unsafe extern "system" fn(HWND, LPARAM) -> i32>,
        LPARAM,
    ) -> i32 = EnumWindows;
    let _: unsafe extern "system" fn(*const u16, *const u16) -> HWND = FindWindowW;
    let _: unsafe extern "system" fn(HWND, HWND, *const u16, *const u16) -> HWND = FindWindowExW;
    let _: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM, u32, u32, *mut usize) -> isize =
        SendMessageTimeoutW;
    let _: unsafe extern "system" fn(HWND, HWND) -> HWND = SetParent;
    let _: unsafe extern "system" fn(HWND, *mut windows_sys::Win32::Foundation::RECT) -> i32 =
        GetClientRect;
    let _: unsafe extern "system" fn(HWND, HWND, i32, i32, i32, i32, u32) -> i32 = SetWindowPos;
    let _: unsafe extern "system" fn(HWND, i32, u32, u32) -> i32 = RegisterHotKey;
    let _: unsafe extern "system" fn(HWND, i32) -> i32 = UnregisterHotKey;
    let _ = (MOD_ALT, MOD_CONTROL, SMTO_NORMAL);
    println!("win32-sys probe ok");
}
#[cfg(not(windows))]
fn main() {}
