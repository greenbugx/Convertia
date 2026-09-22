use tauri::Manager;

pub mod commands;
pub mod conversion;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::preview::PreviewGrants::default())
        .register_uri_scheme_protocol("preview", |context, request| {
            let grants = context
                .app_handle()
                .state::<commands::preview::PreviewGrants>();
            commands::preview::respond(request, &grants)
        })
        .invoke_handler(tauri::generate_handler![
            commands::convert::convert_images,
            commands::preview::set_preview_paths
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
