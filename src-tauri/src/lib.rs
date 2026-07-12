mod desktop_host;
mod devices;
mod native_messaging;
mod server;
pub mod single_instance;
mod windows;

pub fn run(serve_local: bool) {
    if serve_local {
        desktop_host::run();
    } else {
        native_messaging::run();
    }
}
