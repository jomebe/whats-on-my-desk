mod devices;
mod server;

pub fn run() {
    tokio::runtime::Runtime::new()
        .expect("create runtime")
        .block_on(server::serve());
}
