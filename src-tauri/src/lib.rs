mod devices;
mod native_messaging;
mod server;

pub fn run(serve_local: bool) {
    if serve_local {
        tokio::runtime::Runtime::new()
            .expect("create runtime")
            .block_on(server::serve());
    } else {
        native_messaging::run();
    }
}
