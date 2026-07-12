use std::ptr::{null, null_mut};
use windows_sys::Win32::{
    Foundation::{GetLastError, SetLastError, HWND, LPARAM, RECT},
    UI::WindowsAndMessaging::{
        EnumWindows, FindWindowExW, FindWindowW, GetClientRect, SendMessageTimeoutW, SetParent,
        SetWindowPos, SMTO_NORMAL, SWP_FRAMECHANGED, SWP_NOZORDER, SWP_SHOWWINDOW,
    },
};

pub enum Strategy {
    SiblingWorkerW,
    ProgmanDirect,
}
pub struct Result {
    pub wallpaper_parent: HWND,
    pub strategy: Strategy,
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
pub fn find_wallpaper_parent() -> std::result::Result<Result, String> {
    unsafe {
        let progman = FindWindowW(wide("Progman").as_ptr(), null());
        if progman.is_null() {
            return Err("ProgmanNotFound".into());
        }
        let mut output = 0usize;
        let _ = SendMessageTimeoutW(progman, 0x052C, 0xD, 1, SMTO_NORMAL, 1000, &mut output);
        let _ = SendMessageTimeoutW(progman, 0x052C, 0, 0, SMTO_NORMAL, 1000, &mut output);
        let mut context = Box::new(Context {
            defview_parent: null_mut(),
            wallpaper_worker: null_mut(),
        });
        EnumWindows(Some(enum_proc), (&mut *context as *mut Context) as LPARAM);
        let parent = if context.wallpaper_worker.is_null() {
            progman
        } else {
            context.wallpaper_worker
        };
        let strategy = if context.wallpaper_worker.is_null() {
            Strategy::ProgmanDirect
        } else {
            Strategy::SiblingWorkerW
        };
        eprintln!(
            "[workerw] progman={progman:p} defview_parent={:p} wallpaper_workerw={parent:p}",
            context.defview_parent
        );
        Ok(Result {
            wallpaper_parent: parent,
            strategy,
        })
    }
}
struct Context {
    defview_parent: HWND,
    wallpaper_worker: HWND,
}
unsafe extern "system" fn enum_proc(hwnd: HWND, data: LPARAM) -> i32 {
    let ctx = unsafe { &mut *(data as *mut Context) };
    let defview =
        unsafe { FindWindowExW(hwnd, null_mut(), wide("SHELLDLL_DefView").as_ptr(), null()) };
    if !defview.is_null() {
        ctx.defview_parent = hwnd;
        let sibling = unsafe { FindWindowExW(null_mut(), hwnd, wide("WorkerW").as_ptr(), null()) };
        if !sibling.is_null() {
            ctx.wallpaper_worker = sibling;
            return 0;
        }
    }
    1
}
pub fn attach(host: HWND, parent: HWND) -> std::result::Result<(), String> {
    unsafe {
        SetLastError(0);
        let previous = SetParent(host, parent);
        if previous.is_null() && GetLastError() != 0 {
            return Err(format!("SetParent error={}", GetLastError()));
        }
        let mut rect = RECT::default();
        if GetClientRect(parent, &mut rect) == 0 {
            return Err("GetClientRect failed".into());
        }
        if SetWindowPos(
            host,
            null_mut(),
            0,
            0,
            rect.right - rect.left,
            rect.bottom - rect.top,
            SWP_NOZORDER | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
        ) == 0
        {
            return Err(format!("SetWindowPos error={}", GetLastError()));
        }
        eprintln!(
            "[workerw] set_parent success=true parent_client={}x{}",
            rect.right, rect.bottom
        );
        Ok(())
    }
}
