fn main() {
    // In debug builds, this will write `app/bindings.ts` via tauri-specta.
    // The actual app also exports on startup (see `worth_lib::api::invoke_handler`).
    let _handler = worth_lib::api::invoke_handler();
    println!("Bindings export complete.");
}
