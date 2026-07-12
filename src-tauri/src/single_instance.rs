#[cfg(windows)]
pub struct Guard(windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl Guard {
    pub fn acquire() -> Option<Self> {
        use windows::{
            core::w,
            Win32::{
                Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
                System::Threading::CreateMutexW,
            },
        };
        unsafe {
            let handle =
                CreateMutexW(None, false, w!("Local\\WhatsOnMyDesk.SingleInstance")).ok()?;
            if GetLastError() == ERROR_ALREADY_EXISTS {
                let _ = windows::Win32::Foundation::CloseHandle(handle);
                eprintln!("[instance] existing instance found; exiting");
                None
            } else {
                eprintln!("[instance] primary instance acquired");
                Some(Self(handle))
            }
        }
    }
}

#[cfg(windows)]
impl Drop for Guard {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(not(windows))]
pub struct Guard;
#[cfg(not(windows))]
impl Guard {
    pub fn acquire() -> Option<Self> {
        Some(Self)
    }
}
