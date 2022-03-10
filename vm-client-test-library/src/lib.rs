#[no_mangle]
pub fn print_current_thread_name() {
    let current_thread = std::thread::current();
    println!(
        "Current thread name: {:?}, id: {:?}",
        current_thread.name(),
        current_thread.id()
    );
}
