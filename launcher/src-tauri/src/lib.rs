mod commands;

use commands::auth::AuthState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AuthState::default())
        .invoke_handler(tauri::generate_handler![
            commands::system::get_system_info,
            commands::launch::launch_game,
            commands::download::download_assets,
            commands::auth::get_auth_state,
            commands::auth::login_microsoft,
            commands::auth::logout
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}