#[cfg(windows)]
pub fn run() {
    use std::sync::{
        atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering},
        OnceLock,
    };
    let (server_ready_tx, server_ready_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .expect("runtime")
            .block_on(crate::server::serve(false, Some(server_ready_tx)));
    });
    if let Err(error) = server_ready_rx.recv_timeout(std::time::Duration::from_secs(30)) {
        eprintln!("[agent] loopback startup failed: {error}");
        return;
    }
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
        TrayEnter,
        TrayWallpaper,
        TrayRefresh,
        TrayExit,
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
    let tray = {
        use tray_icon::{
            menu::{Menu, MenuEvent, MenuItem},
            Icon, TrayIconBuilder, TrayIconEvent,
        };
        let menu = Menu::new();
        let enter = MenuItem::new("Enter Interaction Mode", true, None);
        let wallpaper = MenuItem::new("Return to Wallpaper Mode", true, None);
        let refresh = MenuItem::new("Refresh Devices", true, None);
        let exit = MenuItem::new("Exit", true, None);
        let _ = menu.append(&enter);
        let _ = menu.append(&wallpaper);
        let _ = menu.append(&refresh);
        let _ = menu.append(&exit);
        let proxy = EVENT_PROXY.get().expect("event proxy").clone();
        let enter_id = enter.id().clone();
        let wallpaper_id = wallpaper.id().clone();
        let refresh_id = refresh.id().clone();
        let exit_id = exit.id().clone();
        MenuEvent::set_event_handler(Some(move |event: tray_icon::menu::MenuEvent| {
            let target = if event.id() == &enter_id {
                HostEvent::TrayEnter
            } else if event.id() == &wallpaper_id {
                HostEvent::TrayWallpaper
            } else if event.id() == &refresh_id {
                HostEvent::TrayRefresh
            } else if event.id() == &exit_id {
                HostEvent::TrayExit
            } else {
                return;
            };
            let _ = proxy.send_event(target);
        }));
        let proxy = EVENT_PROXY.get().expect("event proxy").clone();
        TrayIconEvent::set_event_handler(Some(move |event| {
            let is_left_click = match event {
                TrayIconEvent::Click { button, .. } | TrayIconEvent::DoubleClick { button, .. } => {
                    button == tray_icon::MouseButton::Left
                }
                _ => false,
            };
            if is_left_click {
                let _ = proxy.send_event(HostEvent::TrayEnter);
            }
        }));
        let icon =
            Icon::from_rgba([0xd1, 0x9a, 0x68, 0xff].repeat(16 * 16), 16, 16).expect("tray icon");
        TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("What’s on My Desk?")
            .with_icon(icon)
            .build()
            .expect("tray icon")
    };
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
    let mut wallpaper_target = match crate::windows::workerw::find_wallpaper_parent() {
        Ok(found) => match crate::windows::workerw::attach(host, &found) {
            Ok(()) => {
                crate::windows::workerw::log_shell(host, &found);
                let _ = webview.evaluate_script(&format!("window.dispatchEvent(new CustomEvent('womd-host-debug', {{ detail: {{ strategy: '{}' }} }}));", found.strategy.label()));
                eprintln!("[workerw] attach_success=true");
                Some(found)
            }
            Err(error) => {
                eprintln!("[workerw] attach failed, fallback=desktop-overlay error={error}");
                eprintln!("[workerw] attach_success=false");
                None
            }
        },
        Err(error) => {
            eprintln!("[workerw] attach failed, fallback=desktop-overlay error={error}");
            eprintln!("[workerw] attach_success=false");
            None
        }
    };
    fit_webview(&webview, windows[0].inner_size());
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
            Event::UserEvent(HostEvent::TrayEnter) => { if !INTERACTIVE.load(Ordering::Relaxed) { let _ = EVENT_PROXY.get().expect("event proxy").send_event(HostEvent::ToggleInteraction); } }
            Event::UserEvent(HostEvent::TrayWallpaper) => { if INTERACTIVE.load(Ordering::Relaxed) { let _ = EVENT_PROXY.get().expect("event proxy").send_event(HostEvent::ToggleInteraction); } }
            Event::UserEvent(HostEvent::TrayRefresh) => { let _ = webview.evaluate_script("window.location.reload()"); }
            Event::UserEvent(HostEvent::TrayExit) => { let _ = &tray; *flow = ControlFlow::Exit; }
            Event::UserEvent(HostEvent::Wallpaper) if INTERACTIVE.load(Ordering::Relaxed) => {
                INTERACTIVE.store(false, Ordering::Relaxed);
                crate::windows::interaction::set_mode(host, crate::windows::interaction::InteractionMode::Wallpaper);
                let _ = webview.focus_parent();
                let _ = webview.evaluate_script("window.dispatchEvent(new CustomEvent('womd-interaction-mode', { detail: { mode: 'wallpaper' } }));");
            }
            Event::WindowEvent { window_id, event: tao::event::WindowEvent::Resized(size), .. }
                if windows.last().is_some_and(|window| window.id() == window_id) =>
            {
                fit_webview(&webview, size);
            }
            Event::UserEvent(HostEvent::HealthCheck) => unsafe {
                let attachment_error = wallpaper_target
                    .as_ref()
                    .map_or_else(
                        || Some("wallpaper target missing".to_string()),
                        |found| crate::windows::workerw::validate_attachment(host, found).err(),
                    );
                if let Some(reason) = attachment_error {
                    eprintln!("[shell] attachment unhealthy error={reason}");
                    if let Ok(found) = crate::windows::workerw::find_wallpaper_parent() {
                        let strategy = found.strategy.label();
                        match crate::windows::workerw::attach(host, &found) {
                            Ok(()) => {
                                wallpaper_target = Some(found);
                                fit_webview(
                                    &webview,
                                    windows.last().expect("desktop window").inner_size(),
                                );
                                let mode = if INTERACTIVE.load(Ordering::Relaxed) { crate::windows::interaction::InteractionMode::Interactive } else { crate::windows::interaction::InteractionMode::Wallpaper };
                                crate::windows::interaction::set_mode(host, mode);
                                crate::windows::workerw::log_shell(host, &found);
                                eprintln!("[shell] reattach strategy={strategy}");
                                eprintln!("[shell] reattach success=true");
                                eprintln!("[shell] interaction mode restored={}", mode.label());
                            }
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
                                let previous_webview = std::mem::replace(&mut webview, next_webview);
                                let previous_window = windows.pop().expect("previous desktop window");
                                windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                                    host,
                                    windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE,
                                );
                                drop(previous_webview);
                                drop(previous_window);
                                host = next_host;
                                windows.push(next_window);
                                match crate::windows::workerw::attach(host, &found) {
                                    Ok(()) => {
                                        wallpaper_target = Some(found);
                                        fit_webview(&webview, windows.last().expect("recovery window").inner_size());
                                        crate::windows::interaction::set_mode(host, if INTERACTIVE.load(Ordering::Relaxed) { crate::windows::interaction::InteractionMode::Interactive } else { crate::windows::interaction::InteractionMode::Wallpaper });
                                        crate::windows::workerw::log_shell(host, &found);
                                        eprintln!("[hotkey] recovery register success={registered}");
                                        eprintln!("[shell] reattach strategy={strategy}");
                                        eprintln!("[shell] reattach success=true");
                                    }
                                    Err(error) => {
                                        wallpaper_target = None;
                                        eprintln!("[shell] reattach success=false error={error}");
                                    }
                                }
                            }
                        }
                    } else {
                        wallpaper_target = None;
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

#[cfg(windows)]
fn fit_webview(webview: &wry::WebView, size: tao::dpi::PhysicalSize<u32>) {
    if size.width == 0 || size.height == 0 {
        return;
    }
    if let Err(error) = webview.set_bounds(wry::Rect {
        position: wry::dpi::LogicalPosition::new(0, 0).into(),
        size: wry::dpi::PhysicalSize::new(size.width, size.height).into(),
    }) {
        eprintln!("[webview] resize failed error={error}");
    }
}

#[cfg(not(windows))]
pub fn run() {
    tokio::runtime::Runtime::new()
        .expect("runtime")
        .block_on(crate::server::serve(true, None));
}
