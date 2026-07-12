#[cfg(windows)]
pub fn run() {
    std::thread::spawn(|| { tokio::runtime::Runtime::new().expect("runtime").block_on(crate::server::serve(false)); });
    std::thread::sleep(std::time::Duration::from_millis(700));
    use tao::{event::Event, event_loop::{ControlFlow, EventLoop}, platform::windows::WindowBuilderExtWindows, window::WindowBuilder};
    use wry::WebViewBuilder;
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("What’s on My Desk?").with_decorations(false).with_maximized(true).with_always_on_bottom(true).with_skip_taskbar(true).build(&event_loop).expect("desktop window");
    let _webview = WebViewBuilder::new().with_url("http://127.0.0.1:47831").build(&window).expect("WebView2");
    event_loop.run(move |event, _, flow| { *flow = ControlFlow::Wait; if let Event::WindowEvent { event: tao::event::WindowEvent::CloseRequested, .. } = event { *flow = ControlFlow::Exit; } });
}

#[cfg(not(windows))]
pub fn run() { tokio::runtime::Runtime::new().expect("runtime").block_on(crate::server::serve(true)); }
