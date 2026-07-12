use windows::Win32::{
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

pub fn set_mode(hwnd: HWND, mode: InteractionMode) {
    unsafe {
        let mut style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        match mode {
            InteractionMode::Wallpaper => style |= WS_EX_TRANSPARENT.0 | WS_EX_NOACTIVATE.0,
            InteractionMode::Interactive => style &= !(WS_EX_TRANSPARENT.0 | WS_EX_NOACTIVATE.0),
        }
        let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style as isize);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
        );
        eprintln!(
            "[interaction] mode={}",
            if mode == InteractionMode::Wallpaper {
                "wallpaper"
            } else {
                "interactive"
            }
        );
    }
}
