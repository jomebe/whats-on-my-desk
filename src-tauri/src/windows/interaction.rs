use std::ptr::null_mut;
use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, SWP_FRAMECHANGED,
        SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT,
    },
};
#[derive(Clone, Copy, PartialEq)]
pub enum InteractionMode {
    Wallpaper,
    Interactive,
}
impl InteractionMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Wallpaper => "wallpaper",
            Self::Interactive => "interactive",
        }
    }
}
pub fn set_mode(hwnd: HWND, mode: InteractionMode) {
    unsafe {
        let mut style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        match mode {
            InteractionMode::Wallpaper => style |= WS_EX_TRANSPARENT | WS_EX_NOACTIVATE,
            InteractionMode::Interactive => style &= !(WS_EX_TRANSPARENT | WS_EX_NOACTIVATE),
        };
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style as isize);
        SetWindowPos(
            hwnd,
            null_mut(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
        );
        eprintln!("[interaction] mode={}", mode.label());
    }
}
