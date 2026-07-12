fn main() {
    let Some(_single_instance) = whats_on_my_desk_agent::single_instance::Guard::acquire() else {
        return;
    };
    let serve_local = !std::env::args().any(|arg| arg == "--native-messaging");
    whats_on_my_desk_agent::run(serve_local);
}
