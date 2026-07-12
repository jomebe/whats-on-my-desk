#[cfg(windows)]
pub fn run() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        OnceLock,
    };
    std::thread::spawn(|| {
        tokio::runtime::Runtime::new()
            .expect("runtime")
            .block_on(crate::server::serve(false));
    });
    std::thread::sleep(std::time::Duration::from_millis(700));
    use tao::{
        event::Event,
        event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
        platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
        window::WindowBuilder,
    };
    use wry::WebViewBuilder;
    const HOTKEY_INTERACTION_TOGGLE: i32 = 0x574F4D44;
    enum HostEvent {
        ToggleInteraction,
        Wallpaper,
    }
    static INTERACTIVE: AtomicBool = AtomicBool::new(false);
    static EVENT_PROXY: OnceLock<EventLoopProxy<HostEvent>> = OnceLock::new();
    static ORIGINAL_WNDPROC: OnceLock<isize> = OnceLock::new();
    unsafe extern "system" fn host_wndproc(
        hwnd: windows_sys::Win32::Foundation::HWND,
        message: u32,
        wparam: windows_sys::Win32::Foundation::WPARAM,
        lparam: windows_sys::Win32::Foundation::LPARAM,
    ) -> windows_sys::Win32::Foundation::LRESULT {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CallWindowProcW, HTTRANSPARENT, WM_HOTKEY, WM_NCHITTEST, WNDPROC,
        };
        if message == WM_HOTKEY && wparam == HOTKEY_INTERACTION_TOGGLE as usize {
            if let Some(proxy) = EVENT_PROXY.get() {
                let _ = proxy.send_event(HostEvent::ToggleInteraction);
            }
            return 0;
        }
        if message == WM_NCHITTEST && !INTERACTIVE.load(Ordering::Relaxed) {
            return HTTRANSPARENT as isize;
        }
        let original = ORIGINAL_WNDPROC.get().copied().unwrap_or_default();
        let proc: WNDPROC = Some(std::mem::transmute::<
            isize,
            unsafe extern "system" fn(
                windows_sys::Win32::Foundation::HWND,
                u32,
                windows_sys::Win32::Foundation::WPARAM,
                windows_sys::Win32::Foundation::LPARAM,
            ) -> windows_sys::Win32::Foundation::LRESULT,
        >(original));
        CallWindowProcW(proc, hwnd, message, wparam, lparam)
    }
    let event_loop = EventLoopBuilder::<HostEvent>::with_user_event().build();
    let _ = EVENT_PROXY.set(event_loop.create_proxy());
    let window = WindowBuilder::new()
        .with_title("What’s on My Desk?")
        .with_decorations(false)
        .with_maximized(true)
        .with_always_on_bottom(true)
        .with_skip_taskbar(true)
        .build(&event_loop)
        .expect("desktop window");
    let host = window.hwnd() as windows_sys::Win32::Foundation::HWND;
    let ipc_proxy = EVENT_PROXY.get().expect("event proxy").clone();
    let webview = WebViewBuilder::new()
        .with_url("http://127.0.0.1:47831")
        .with_ipc_handler(move |request| {
            if request.body() == "set-interaction-mode:wallpaper" {
                let _ = ipc_proxy.send_event(HostEvent::Wallpaper);
            }
        })
        .build(&window)
        .expect("WebView2");
    match crate::windows::workerw::find_wallpaper_parent() {
        Ok(found) => {
            let _strategy = found.strategy;
            match crate::windows::workerw::attach(host, found.wallpaper_parent) {
                Ok(()) => {
                    eprintln!("[workerw] attach_success=true");
                    true
                }
                Err(error) => {
                    eprintln!("[workerw] attach failed, fallback=desktop-overlay error={error}");
                    eprintln!("[workerw] attach_success=false");
                    false
                }
            }
        }
        Err(error) => {
            eprintln!("[workerw] attach failed, fallback=desktop-overlay error={error}");
            eprintln!("[workerw] attach_success=false");
            false
        }
    };
    crate::windows::interaction::set_mode(
        host,
        crate::windows::interaction::InteractionMode::Wallpaper,
    );
    unsafe {
        use windows_sys::Win32::{
            Foundation::{GetLastError, SetLastError},
            UI::{
                Input::KeyboardAndMouse::{RegisterHotKey, MOD_ALT, MOD_CONTROL},
                WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWLP_WNDPROC},
            },
        };
        let old = GetWindowLongPtrW(host, GWLP_WNDPROC);
        SetWindowLongPtrW(
            host,
            GWLP_WNDPROC,
            host_wndproc as *const () as usize as isize,
        );
        let _ = ORIGINAL_WNDPROC.set(old);
        SetLastError(0);
        let registered = RegisterHotKey(
            host,
            HOTKEY_INTERACTION_TOGGLE,
            MOD_CONTROL | MOD_ALT,
            b'D' as u32,
        ) != 0;
        eprintln!(
            "[hotkey] register id={HOTKEY_INTERACTION_TOGGLE} success={registered} error={}",
            GetLastError()
        );
    }
    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(HostEvent::ToggleInteraction) => {
                let next = if INTERACTIVE.load(Ordering::Relaxed) { crate::windows::interaction::InteractionMode::Wallpaper } else { crate::windows::interaction::InteractionMode::Interactive };
                INTERACTIVE.store(next == crate::windows::interaction::InteractionMode::Interactive, Ordering::Relaxed);
                crate::windows::interaction::set_mode(host, next);
                let _ = webview.evaluate_script(&format!("window.dispatchEvent(new CustomEvent('womd-interaction-mode', {{ detail: {{ mode: '{}' }} }}));", next.label()));
            }
            Event::UserEvent(HostEvent::Wallpaper) if INTERACTIVE.load(Ordering::Relaxed) => {
                INTERACTIVE.store(false, Ordering::Relaxed);
                crate::windows::interaction::set_mode(host, crate::windows::interaction::InteractionMode::Wallpaper);
                let _ = webview.evaluate_script("window.dispatchEvent(new CustomEvent('womd-interaction-mode', { detail: { mode: 'wallpaper' } }));");
            }
            Event::WindowEvent { event: tao::event::WindowEvent::CloseRequested, .. } => {
                unsafe { use windows_sys::Win32::UI::Input::KeyboardAndMouse::UnregisterHotKey; let _ = UnregisterHotKey(host, HOTKEY_INTERACTION_TOGGLE); }
                *flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

#[cfg(not(windows))]
pub fn run() {
    tokio::runtime::Runtime::new()
        .expect("runtime")
        .block_on(crate::server::serve(true));
}
