fn main() {
    let serve_local = std::env::args().any(|arg| arg == "--serve-local");
    whats_on_my_desk_agent::run(serve_local);
}
