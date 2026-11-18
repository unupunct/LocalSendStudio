mod client;
mod commands;
mod db;
mod discovery;
mod identity;
mod protocol;
mod server;
mod shared_folders;
mod state;
mod transfer;
mod util;

use state::AppState;
use tauri::Manager;
use transfer::new_cancel_map;

pub fn run() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = AppState::new();
            let cancel = new_cancel_map();
            // start server
            let st = state.clone();
            tauri::async_runtime::spawn(async move {
                server::run_server(st, 47821).await;
            });
            // start discovery
            let st2 = state.clone();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                discovery::start(st2, handle).await;
            });
            app.manage(state);
            app.manage(cancel);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_identity,
            commands::set_device_name,
            commands::get_devices,
            commands::get_interfaces,
            commands::get_shared_folders,
            commands::add_shared_folder,
            commands::remove_shared_folder,
            commands::toggle_shared_folder,
            commands::set_folder_perms,
            commands::list_remote_dir,
            commands::remote_info,
            commands::make_remote_dir,
            commands::delete_remote_path,
            commands::rename_remote_path,
            commands::upload_to_remote,
            commands::download_from_remote,
            commands::cancel_transfer,
            commands::get_transfer_history,
            commands::get_settings,
            commands::set_settings,
            commands::get_groups,
            commands::add_group,
            commands::delete_group,
            commands::network_diagnostics,
            commands::speed_test,
            commands::get_local_shared_storage,
            commands::sync_local_to_remote,
            commands::sync_remote_to_local,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
