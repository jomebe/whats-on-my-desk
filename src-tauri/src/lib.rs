mod devices;
mod desktop_host;
mod native_messaging;
mod server;

pub fn run(serve_local: bool) {
    if serve_local {
        desktop_host::run();
    } else {
        native_messaging::run();
    }
}
