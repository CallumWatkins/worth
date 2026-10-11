fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().plugin(
        "window-snap",
        tauri_build::InlinedPlugin::new().commands(&["set_bounds", "show_system_menu"]),
    ))
    .expect("failed to build Tauri application");
}
