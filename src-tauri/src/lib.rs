pub mod commands;
pub mod conversion;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![commands::convert::convert_images])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
