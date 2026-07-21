use std::ptr::{null, null_mut};
use windows_sys::Win32::{
    Foundation::{GetLastError, SetLastError, HWND, LPARAM, POINT, RECT},
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTOPRIMARY},
    UI::WindowsAndMessaging::{
        EnumWindows, FindWindowExW, FindWindowW, GetClientRect, GetParent, GetWindow,
        GetWindowLongPtrW, GetWindowRect, IsWindow, IsWindowVisible, SendMessageTimeoutW,
        SetLayeredWindowAttributes, SetParent, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE,
        GWL_STYLE, GW_HWNDPREV, HWND_BOTTOM, HWND_TOP, LWA_ALPHA, SMTO_NORMAL, SWP_FRAMECHANGED,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, WS_BORDER, WS_CAPTION, WS_CHILD,
        WS_DLGFRAME, WS_EX_APPWINDOW, WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME, WS_EX_LAYERED,
        WS_EX_NOACTIVATE, WS_EX_STATICEDGE, WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE, WS_MAXIMIZE,
        WS_MAXIMIZEBOX, WS_MINIMIZE, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
        WS_VISIBLE,
    },
};

#[derive(Clone, Copy)]
pub enum Strategy {
    SiblingWorkerW,
    DefViewParent,
    ProgmanDirect,
}
impl Strategy {
    pub fn label(self) -> &'static str {
        match self {
            Self::SiblingWorkerW => "SiblingWorkerW",
            Self::DefViewParent => "DefViewParent",
            Self::ProgmanDirect => "ProgmanDirect",
        }
    }
}
#[derive(Clone, Copy)]
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
        let _ = SendMessageTimeoutW(progman, 0x052C, 0xD, 1, SMTO_NORMAL, 1000, &mut output);
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
        let (parent, strategy) = if !context.wallpaper_worker.is_null() {
            (context.wallpaper_worker, Strategy::SiblingWorkerW)
        } else if !context.defview_parent.is_null() {
            (context.defview_parent, Strategy::DefViewParent)
        } else {
            (progman, Strategy::ProgmanDirect)
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
pub fn attach(host: HWND, found: &Result) -> std::result::Result<(), String> {
    unsafe {
        validate_target(found)?;
        let parent = found.wallpaper_parent;
        debug(|| eprintln!("[workerw] host_before_parent={:p}", GetParent(host)));
        let style = GetWindowLongPtrW(host, GWL_STYLE) as u32;
        let window_frame = WS_POPUP
            | WS_CAPTION
            | WS_BORDER
            | WS_DLGFRAME
            | WS_THICKFRAME
            | WS_SYSMENU
            | WS_MINIMIZE
            | WS_MINIMIZEBOX
            | WS_MAXIMIZE
            | WS_MAXIMIZEBOX;
        SetWindowLongPtrW(
            host,
            GWL_STYLE,
            ((style & !window_frame) | WS_CHILD | WS_VISIBLE) as isize,
        );
        let ex_style = GetWindowLongPtrW(host, GWL_EXSTYLE) as u32;
        let extended_frame = WS_EX_APPWINDOW
            | WS_EX_WINDOWEDGE
            | WS_EX_CLIENTEDGE
            | WS_EX_DLGMODALFRAME
            | WS_EX_STATICEDGE;
        let layered = if matches!(found.strategy, Strategy::DefViewParent) {
            WS_EX_LAYERED
        } else {
            0
        };
        SetWindowLongPtrW(
            host,
            GWL_EXSTYLE,
            ((ex_style & !extended_frame) | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | layered) as isize,
        );
        if layered != 0 && SetLayeredWindowAttributes(host, 0, 255, LWA_ALPHA) == 0 {
            return Err(format!(
                "SetLayeredWindowAttributes error={}",
                GetLastError()
            ));
        }
        SetLastError(0);
        let previous = SetParent(host, parent);
        let last_error = GetLastError();
        debug(|| eprintln!("[workerw] set_parent_previous={previous:p}"));
        debug(|| eprintln!("[workerw] set_parent_last_error={last_error}"));
        if previous.is_null() && last_error != 0 {
            return Err(format!("SetParent error={last_error}"));
        }
        if GetParent(host) != parent {
            return Err(format!(
                "SetParent verification failed parent={:p}",
                GetParent(host)
            ));
        }
        let target_rect = get_primary_target_rect(parent);
        let parent_size = (target_rect.2, target_rect.3);
        if !usable_size(parent_size) {
            return Err(format!(
                "wallpaper parent has invalid target size={}x{}",
                parent_size.0, parent_size.1
            ));
        }
        let insert_after = match found.strategy {
            Strategy::SiblingWorkerW => HWND_TOP,
            Strategy::DefViewParent => found.defview,
            Strategy::ProgmanDirect if GetParent(found.defview) == parent => found.defview,
            Strategy::ProgmanDirect => HWND_TOP,
        };
        let mut requested_size = parent_size;
        for _ in 0..3 {
            if SetWindowPos(
                host,
                insert_after,
                target_rect.0,
                target_rect.1,
                requested_size.0,
                requested_size.1,
                SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            ) == 0
            {
                return Err(format!("SetWindowPos error={}", GetLastError()));
            }
            let mut host_rect = RECT::default();
            if GetClientRect(host, &mut host_rect) == 0 {
                return Err("wallpaper host client rect unavailable".into());
            }
            let host_size = dimensions(host_rect);
            if same_size(parent_size, host_size) {
                break;
            }
            requested_size.0 += parent_size.0 - host_size.0;
            requested_size.1 += parent_size.1 - host_size.1;
        }
        if matches!(found.strategy, Strategy::DefViewParent) {
            let stock_wallpaper = FindWindowExW(
                found.wallpaper_parent,
                null_mut(),
                wide("WorkerW").as_ptr(),
                null(),
            );
            if !stock_wallpaper.is_null()
                && SetWindowPos(
                    stock_wallpaper,
                    HWND_BOTTOM,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
                ) == 0
            {
                return Err(format!(
                    "stock wallpaper SetWindowPos error={}",
                    GetLastError()
                ));
            }
        }
        debug(|| {
            let mut host_window_rect = RECT::default();
            let mut host_client_rect = RECT::default();
            let _ = GetWindowRect(host, &mut host_window_rect);
            let _ = GetClientRect(host, &mut host_client_rect);
            eprintln!(
                "[workerw] style before={style:#010x} after={:#010x} ex_before={ex_style:#010x} ex_after={:#010x} window={}x{} client={}x{}",
                GetWindowLongPtrW(host, GWL_STYLE) as u32,
                GetWindowLongPtrW(host, GWL_EXSTYLE) as u32,
                dimensions(host_window_rect).0,
                dimensions(host_window_rect).1,
                dimensions(host_client_rect).0,
                dimensions(host_client_rect).1,
            );
        });
        debug(|| {
            eprintln!(
                "[workerw] parent_client={}x{}",
                parent_size.0, parent_size.1
            )
        });
        debug(|| eprintln!("[workerw] host_after_parent={:p}", GetParent(host)));
        validate_attachment(host, found)
    }
}

pub fn validate_attachment(host: HWND, found: &Result) -> std::result::Result<(), String> {
    unsafe {
        validate_target(found)?;
        if host.is_null() || IsWindow(host) == 0 {
            return Err("wallpaper host is invalid".into());
        }
        if GetParent(host) != found.wallpaper_parent {
            return Err(format!(
                "wallpaper host parent changed parent={:p}",
                GetParent(host)
            ));
        }
        if IsWindowVisible(host) == 0 {
            return Err("wallpaper host is not visible".into());
        }
        let target_rect = get_primary_target_rect(found.wallpaper_parent);
        let parent_size = (target_rect.2, target_rect.3);
        let mut host_rect = RECT::default();
        if GetClientRect(host, &mut host_rect) == 0 {
            return Err("wallpaper client rect unavailable".into());
        }
        let host_size = dimensions(host_rect);
        if !usable_size(parent_size) || !usable_size(host_size) {
            return Err(format!(
                "wallpaper client size invalid target={}x{} host={}x{}",
                parent_size.0, parent_size.1, host_size.0, host_size.1
            ));
        }
        if !same_size(parent_size, host_size) {
            return Err(format!(
                "wallpaper client size mismatch target={}x{} host={}x{}",
                parent_size.0, parent_size.1, host_size.0, host_size.1
            ));
        }
        match found.strategy {
            Strategy::SiblingWorkerW => {
                if !is_behind(found.wallpaper_parent, found.defview_parent) {
                    return Err("wallpaper WorkerW moved above desktop icons".into());
                }
            }
            Strategy::DefViewParent => {
                if !is_behind(host, found.defview) {
                    return Err("wallpaper host moved above desktop icons".into());
                }
                let mut worker = FindWindowExW(
                    found.wallpaper_parent,
                    null_mut(),
                    wide("WorkerW").as_ptr(),
                    null(),
                );
                while !worker.is_null() {
                    if IsWindowVisible(worker) != 0 && !is_behind(worker, host) {
                        return Err(format!(
                            "stock wallpaper WorkerW is not behind host worker={worker:p}"
                        ));
                    }
                    worker = FindWindowExW(
                        found.wallpaper_parent,
                        worker,
                        wide("WorkerW").as_ptr(),
                        null(),
                    );
                }
                if GetWindowLongPtrW(host, GWL_EXSTYLE) as u32 & WS_EX_LAYERED == 0 {
                    return Err("wallpaper host is not layered on raised desktop".into());
                }
            }
            Strategy::ProgmanDirect => {}
        }
        Ok(())
    }
}

unsafe fn validate_target(found: &Result) -> std::result::Result<(), String> {
    if found.wallpaper_parent.is_null() || IsWindow(found.wallpaper_parent) == 0 {
        return Err("wallpaper parent is invalid".into());
    }
    match found.strategy {
        Strategy::SiblingWorkerW => {
            if found.defview_parent.is_null()
                || IsWindow(found.defview_parent) == 0
                || found.wallpaper_parent == found.defview_parent
            {
                return Err("wallpaper WorkerW topology is invalid".into());
            }
            let nested_defview = FindWindowExW(
                found.wallpaper_parent,
                null_mut(),
                wide("SHELLDLL_DefView").as_ptr(),
                null(),
            );
            if !nested_defview.is_null() {
                return Err("wallpaper WorkerW contains desktop icons".into());
            }
        }
        Strategy::DefViewParent => {
            if found.defview.is_null()
                || IsWindow(found.defview) == 0
                || GetParent(found.defview) != found.wallpaper_parent
            {
                return Err("desktop icon parent topology is invalid".into());
            }
        }
        Strategy::ProgmanDirect => {
            if found.wallpaper_parent != found.progman {
                return Err("Progman wallpaper parent changed".into());
            }
        }
    }
    Ok(())
}

unsafe fn is_behind(window: HWND, expected_above: HWND) -> bool {
    if window.is_null() || expected_above.is_null() {
        return false;
    }
    let mut above = GetWindow(window, GW_HWNDPREV);
    for _ in 0..128 {
        if above.is_null() {
            return false;
        }
        if above == expected_above {
            return true;
        }
        above = GetWindow(above, GW_HWNDPREV);
    }
    false
}

pub fn get_primary_target_rect(parent: HWND) -> (i32, i32, i32, i32) {
    unsafe {
        let hmon = MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        let mut parent_rect = RECT::default();
        if GetMonitorInfoW(hmon, &mut info) != 0 && GetWindowRect(parent, &mut parent_rect) != 0 {
            let left = info.rcMonitor.left - parent_rect.left;
            let top = info.rcMonitor.top - parent_rect.top;
            let width = info.rcMonitor.right - info.rcMonitor.left;
            let height = info.rcMonitor.bottom - info.rcMonitor.top;
            if width > 0 && height > 0 {
                return (left, top, width, height);
            }
        }

        let mut rect = RECT::default();
        if GetClientRect(parent, &mut rect) != 0 {
            let (w, h) = dimensions(rect);
            return (0, 0, w, h);
        }
        (0, 0, 1920, 1080)
    }
}

fn dimensions(rect: RECT) -> (i32, i32) {
    (rect.right - rect.left, rect.bottom - rect.top)
}

fn usable_size(size: (i32, i32)) -> bool {
    size.0 > 0 && size.1 > 0
}

fn same_size(left: (i32, i32), right: (i32, i32)) -> bool {
    (left.0 - right.0).abs() <= 2 && (left.1 - right.1).abs() <= 2
}

pub fn log_shell(host: HWND, found: &Result) {
    unsafe {
        let host_behind_icons = match found.strategy {
            Strategy::SiblingWorkerW => is_behind(found.wallpaper_parent, found.defview_parent),
            Strategy::DefViewParent => is_behind(host, found.defview),
            Strategy::ProgmanDirect => true,
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachment_sizes_must_be_positive() {
        assert!(usable_size((1920, 1080)));
        assert!(!usable_size((0, 1080)));
        assert!(!usable_size((1920, -1)));
    }

    #[test]
    fn attachment_sizes_allow_rounding_only() {
        assert!(same_size((1920, 1080), (1918, 1082)));
        assert!(!same_size((1920, 1080), (1917, 1080)));
    }
}
