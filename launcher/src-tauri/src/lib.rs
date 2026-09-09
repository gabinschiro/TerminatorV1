mod commands;

use commands::auth::AuthState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AuthState::default())
        .invoke_handler(tauri::generate_handler![
            commands::system::get_system_info,
            commands::launch::launch_game,
            commands::download::download_assets,
            commands::auth::get_auth_state,
            commands::auth::logout,
            commands::msauth::begin_ms_login,
            commands::msauth::complete_ms_login,
            commands::skin::get_player_skin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}