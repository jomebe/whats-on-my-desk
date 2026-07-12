use std::ptr::{null, null_mut};
use windows_sys::Win32::{
    Foundation::{GetLastError, SetLastError, HWND, LPARAM, RECT},
    UI::WindowsAndMessaging::{
        EnumWindows, FindWindowExW, FindWindowW, GetClientRect, GetParent, GetWindow,
        GetWindowLongPtrW, IsWindowVisible, SendMessageTimeoutW, SetParent, SetWindowLongPtrW,
        SetWindowPos, GWL_EXSTYLE, GWL_STYLE, GW_HWNDPREV, SMTO_NORMAL, SWP_FRAMECHANGED,
        SWP_NOZORDER, SWP_SHOWWINDOW, WS_CHILD, WS_EX_APPWINDOW, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
    },
};

#[derive(Clone, Copy)]
pub enum Strategy {
    SiblingWorkerW,
    ProgmanDirect,
}
impl Strategy {
    pub fn label(self) -> &'static str {
        match self {
            Self::SiblingWorkerW => "SiblingWorkerW",
            Self::ProgmanDirect => "ProgmanDirect",
        }
    }
}
pub struct Result {
    pub progman: HWND,
    pub defview_parent: HWND,
    pub defview: HWND,
    pub listview: HWND,
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
        let _ = SendMessageTimeoutW(progman, 0x052C, 0xD, 0, SMTO_NORMAL, 1000, &mut output);
        let _ = SendMessageTimeoutW(progman, 0x052C, 0, 0, SMTO_NORMAL, 1000, &mut output);
        let mut worker = FindWindowExW(null_mut(), null_mut(), wide("WorkerW").as_ptr(), null());
        while !worker.is_null() {
            debug(|| eprintln!("[workerw] top_level_workerw={worker:p}"));
            worker = FindWindowExW(null_mut(), worker, wide("WorkerW").as_ptr(), null());
        }
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
        debug(|| eprintln!("[workerw] progman={progman:p}"));
        debug(|| eprintln!("[workerw] defview_parent={:p}", context.defview_parent));
        debug(|| eprintln!("[workerw] wallpaper_parent={parent:p}"));
        debug(|| eprintln!("[workerw] strategy={}", strategy.label()));
        let defview = FindWindowExW(
            context.defview_parent,
            null_mut(),
            wide("SHELLDLL_DefView").as_ptr(),
            null(),
        );
        let listview = FindWindowExW(defview, null_mut(), wide("SysListView32").as_ptr(), null());
        Ok(Result {
            progman,
            defview_parent: context.defview_parent,
            defview,
            listview,
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
        debug(|| eprintln!("[workerw] host_before_parent={:p}", GetParent(host)));
        SetLastError(0);
        let previous = SetParent(host, parent);
        let last_error = GetLastError();
        debug(|| eprintln!("[workerw] set_parent_previous={previous:p}"));
        debug(|| eprintln!("[workerw] set_parent_last_error={last_error}"));
        if previous.is_null() && last_error != 0 {
            return Err(format!("SetParent error={last_error}"));
        }
        let style = GetWindowLongPtrW(host, GWL_STYLE) as u32;
        SetWindowLongPtrW(
            host,
            GWL_STYLE,
            ((style & !WS_POPUP) | WS_CHILD | WS_VISIBLE) as isize,
        );
        let ex_style = GetWindowLongPtrW(host, GWL_EXSTYLE) as u32;
        SetWindowLongPtrW(
            host,
            GWL_EXSTYLE,
            ((ex_style & !WS_EX_APPWINDOW) | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) as isize,
        );
        if GetParent(host) != parent {
            return Err(format!(
                "SetParent verification failed parent={:p}",
                GetParent(host)
            ));
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
        debug(|| {
            eprintln!(
                "[workerw] parent_client={}x{}",
                rect.right - rect.left,
                rect.bottom - rect.top
            )
        });
        debug(|| eprintln!("[workerw] host_after_parent={:p}", GetParent(host)));
        Ok(())
    }
}

pub fn log_shell(host: HWND, found: &Result) {
    unsafe {
        let mut above = GetWindow(host, GW_HWNDPREV);
        let mut host_behind_icons = false;
        for _ in 0..64 {
            if above.is_null() {
                break;
            }
            if above == found.defview || above == found.defview_parent {
                host_behind_icons = true;
                break;
            }
            above = GetWindow(above, GW_HWNDPREV);
        }
        debug(|| eprintln!("[shell] progman={:p}", found.progman));
        debug(|| eprintln!("[shell] defview={:p}", found.defview));
        debug(|| eprintln!("[shell] defview_parent={:p}", found.defview_parent));
        debug(|| eprintln!("[shell] listview={:p}", found.listview));
        debug(|| eprintln!("[shell] wallpaper_host={host:p}"));
        debug(|| eprintln!("[shell] wallpaper_parent={:p}", found.wallpaper_parent));
        debug(|| eprintln!("[shell] host_visible={}", IsWindowVisible(host) != 0));
        debug(|| eprintln!("[shell] host_behind_icons={host_behind_icons}"));
    }
}

fn debug(log: impl FnOnce()) {
    if std::env::var_os("WOMD_DEBUG_WORKERW").as_deref() == Some(std::ffi::OsStr::new("1")) {
        log();
    }
}
