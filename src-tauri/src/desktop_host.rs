#[cfg(windows)]
pub fn run() {
    use std::sync::{
        atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering},
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
        HealthCheck,
    }
    static INTERACTIVE: AtomicBool = AtomicBool::new(false);
    static EVENT_PROXY: OnceLock<EventLoopProxy<HostEvent>> = OnceLock::new();
    static ORIGINAL_WNDPROC: AtomicIsize = AtomicIsize::new(0);
    static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);
    unsafe extern "system" fn host_wndproc(
        hwnd: windows_sys::Win32::Foundation::HWND,
        message: u32,
        wparam: windows_sys::Win32::Foundation::WPARAM,
        lparam: windows_sys::Win32::Foundation::LPARAM,
    ) -> windows_sys::Win32::Foundation::LRESULT {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CallWindowProcW, HTTRANSPARENT, MA_NOACTIVATE, WM_HOTKEY, WM_KEYDOWN, WM_MOUSEACTIVATE,
            WM_NCHITTEST, WNDPROC,
        };
        if message == WM_HOTKEY && wparam == HOTKEY_INTERACTION_TOGGLE as usize {
            if let Some(proxy) = EVENT_PROXY.get() {
                let _ = proxy.send_event(HostEvent::ToggleInteraction);
            }
            return 0;
        }
        if message == TASKBAR_CREATED.load(Ordering::Relaxed) {
            if let Some(proxy) = EVENT_PROXY.get() {
                let _ = proxy.send_event(HostEvent::HealthCheck);
            }
            return 0;
        }
        if message == WM_KEYDOWN && wparam == 0x1b && INTERACTIVE.load(Ordering::Relaxed) {
            if let Some(proxy) = EVENT_PROXY.get() {
                let _ = proxy.send_event(HostEvent::Wallpaper);
            }
            return 0;
        }
        if message == WM_MOUSEACTIVATE && !INTERACTIVE.load(Ordering::Relaxed) {
            return MA_NOACTIVATE as isize;
        }
        if message == WM_NCHITTEST && !INTERACTIVE.load(Ordering::Relaxed) {
            return HTTRANSPARENT as isize;
        }
        let original = ORIGINAL_WNDPROC.load(Ordering::Relaxed);
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
    let health_proxy = EVENT_PROXY.get().expect("event proxy").clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(4));
        if health_proxy.send_event(HostEvent::HealthCheck).is_err() {
            break;
        }
    });
    let window = WindowBuilder::new()
        .with_title("What’s on My Desk?")
        .with_decorations(false)
        .with_maximized(true)
        .with_always_on_bottom(true)
        .with_skip_taskbar(true)
        .build(&event_loop)
        .expect("desktop window");
    let mut windows = vec![window];
    let mut host = windows[0].hwnd() as windows_sys::Win32::Foundation::HWND;
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::RegisterWindowMessageW;
        let taskbar_created: Vec<u16> = "TaskbarCreated".encode_utf16().chain(Some(0)).collect();
        TASKBAR_CREATED.store(
            RegisterWindowMessageW(taskbar_created.as_ptr()),
            Ordering::Relaxed,
        );
    }
    let ipc_proxy = EVENT_PROXY.get().expect("event proxy").clone();
    let web_url = if std::env::var_os("WOMD_DEBUG_INTERACTION").is_some() {
        "http://127.0.0.1:47831?debugInteraction=1"
    } else {
        "http://127.0.0.1:47831"
    };
    let mut webview = WebViewBuilder::new()
        .with_url(web_url)
        .with_ipc_handler(move |request| {
            if request.body() == "set-interaction-mode:wallpaper" {
                let _ = ipc_proxy.send_event(HostEvent::Wallpaper);
            }
        })
        .build(&windows[0])
        .expect("WebView2");
    let mut wallpaper_parent = match crate::windows::workerw::find_wallpaper_parent() {
        Ok(found) => {
            let _strategy = found.strategy;
            match crate::windows::workerw::attach(host, found.wallpaper_parent) {
                Ok(()) => {
                    crate::windows::workerw::log_shell(host, &found);
                    let _ = webview.evaluate_script(&format!("window.dispatchEvent(new CustomEvent('womd-host-debug', {{ detail: {{ strategy: '{}' }} }}));", found.strategy.label()));
                    eprintln!("[workerw] attach_success=true");
                    found.wallpaper_parent
                }
                Err(error) => {
                    eprintln!("[workerw] attach failed, fallback=desktop-overlay error={error}");
                    eprintln!("[workerw] attach_success=false");
                    std::ptr::null_mut()
                }
            }
        }
        Err(error) => {
            eprintln!("[workerw] attach failed, fallback=desktop-overlay error={error}");
            eprintln!("[workerw] attach_success=false");
            std::ptr::null_mut()
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
        ORIGINAL_WNDPROC.store(old, Ordering::Relaxed);
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
    event_loop.run(move |event, target, flow| {
        *flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(HostEvent::ToggleInteraction) => {
                let next = if INTERACTIVE.load(Ordering::Relaxed) { crate::windows::interaction::InteractionMode::Wallpaper } else { crate::windows::interaction::InteractionMode::Interactive };
                INTERACTIVE.store(next == crate::windows::interaction::InteractionMode::Interactive, Ordering::Relaxed);
                crate::windows::interaction::set_mode(host, next);
                if next == crate::windows::interaction::InteractionMode::Interactive {
                    unsafe {
                        use windows_sys::Win32::UI::{Input::KeyboardAndMouse::{SetActiveWindow, SetFocus}, WindowsAndMessaging::SetForegroundWindow};
                        let _ = SetForegroundWindow(host); let _ = SetActiveWindow(host); let _ = SetFocus(host);
                    }
                    let _ = webview.focus();
                } else { let _ = webview.focus_parent(); }
                let _ = webview.evaluate_script(&format!("window.dispatchEvent(new CustomEvent('womd-interaction-mode', {{ detail: {{ mode: '{}' }} }}));", next.label()));
            }
            Event::UserEvent(HostEvent::Wallpaper) if INTERACTIVE.load(Ordering::Relaxed) => {
                INTERACTIVE.store(false, Ordering::Relaxed);
                crate::windows::interaction::set_mode(host, crate::windows::interaction::InteractionMode::Wallpaper);
                let _ = webview.focus_parent();
                let _ = webview.evaluate_script("window.dispatchEvent(new CustomEvent('womd-interaction-mode', { detail: { mode: 'wallpaper' } }));");
            }
            Event::UserEvent(HostEvent::HealthCheck) => unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::IsWindow;
                if wallpaper_parent.is_null() || IsWindow(wallpaper_parent) == 0 {
                    eprintln!("[shell] parent invalid");
                    eprintln!("[shell] explorer restart detected");
                    if let Ok(found) = crate::windows::workerw::find_wallpaper_parent() {
                        let strategy = found.strategy.label();
                        match crate::windows::workerw::attach(host, found.wallpaper_parent) {
                            Ok(()) => { wallpaper_parent = found.wallpaper_parent; crate::windows::workerw::log_shell(host, &found); eprintln!("[shell] reattach strategy={strategy}"); eprintln!("[shell] reattach success=true"); eprintln!("[shell] interaction mode restored={}", if INTERACTIVE.load(Ordering::Relaxed) { "interactive" } else { "wallpaper" }); }
                            Err(error) => {
                                eprintln!("[shell] reattach failed error={error}; recreating host");
                                let _ = windows_sys::Win32::UI::Input::KeyboardAndMouse::UnregisterHotKey(host, HOTKEY_INTERACTION_TOGGLE);
                                let next_window = WindowBuilder::new().with_title("What’s on My Desk?").with_decorations(false).with_maximized(true).with_always_on_bottom(true).with_skip_taskbar(true).build(target).expect("recovery desktop window");
                                let next_host = next_window.hwnd() as windows_sys::Win32::Foundation::HWND;
                                let next_proxy = EVENT_PROXY.get().expect("event proxy").clone();
                                let next_webview = WebViewBuilder::new().with_url(web_url).with_ipc_handler(move |request| { if request.body() == "set-interaction-mode:wallpaper" { let _ = next_proxy.send_event(HostEvent::Wallpaper); } }).build(&next_window).expect("recovery WebView2");
                                let old = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(next_host, windows_sys::Win32::UI::WindowsAndMessaging::GWLP_WNDPROC);
                                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(next_host, windows_sys::Win32::UI::WindowsAndMessaging::GWLP_WNDPROC, host_wndproc as *const () as usize as isize);
                                ORIGINAL_WNDPROC.store(old, Ordering::Relaxed);
                                let registered = windows_sys::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(next_host, HOTKEY_INTERACTION_TOGGLE, windows_sys::Win32::UI::Input::KeyboardAndMouse::MOD_CONTROL | windows_sys::Win32::UI::Input::KeyboardAndMouse::MOD_ALT, b'D' as u32) != 0;
                                host = next_host;
                                windows.push(next_window);
                                webview = next_webview;
                                match crate::windows::workerw::attach(host, found.wallpaper_parent) {
                                    Ok(()) => { wallpaper_parent = found.wallpaper_parent; crate::windows::interaction::set_mode(host, if INTERACTIVE.load(Ordering::Relaxed) { crate::windows::interaction::InteractionMode::Interactive } else { crate::windows::interaction::InteractionMode::Wallpaper }); crate::windows::workerw::log_shell(host, &found); eprintln!("[hotkey] recovery register success={registered}"); eprintln!("[shell] reattach strategy={strategy}"); eprintln!("[shell] reattach success=true"); }
                                    Err(error) => eprintln!("[shell] reattach success=false error={error}"),
                                }
                            }
                        }
                    }
                }
            },
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
