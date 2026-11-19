mod auth;
mod client;
mod commands;
mod db;
mod discovery;
mod identity;
mod protocol;
mod server;
mod state;
mod transfer;
mod util;

#[cfg(test)]
mod tests;

use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};

#[cfg(desktop)]
fn show_main(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[cfg(desktop)]
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
    let show = MenuItem::with_id(app, "show", "Open LocalSendStudio", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    let mut b = TrayIconBuilder::new()
        .tooltip("LocalSendStudio")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, ev| match ev.id.as_ref() {
            "show" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        b = b.icon(icon.clone());
    }
    b.build(app)?;
    Ok(())
}

/// First run: create an "Inbox" share (receive-only) so sending files works out of the box.
fn ensure_inbox(state: &AppState) {
    if state.db.get_setting("inbox_created").is_some() {
        return;
    }
    let dir = state
        .db
        .get_setting("download_dir")
        .filter(|d| !d.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(util::default_download_dir);
    if std::fs::create_dir_all(&dir).is_ok() {
        let f = db::SharedFolder {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Inbox".into(),
            path: dir.to_string_lossy().to_string(),
            enabled: true,
            read: false,
            upload: true,
            manage: false,
        };
        state.db.upsert_shared_folder(&f);
        state.shared_folders.lock().push(f);
    }
    state.db.set_setting("inbox_created", "1");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::try_init();
    let builder = tauri::Builder::default();
    // LSS_ALLOW_MULTIPLE=1 lets developers run two instances on one machine to test discovery.
    #[cfg(desktop)]
    let builder = if std::env::var_os("LSS_ALLOW_MULTIPLE").is_some() {
        builder
    } else {
        builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_main(app)))
    };
    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let emit: state::Emitter = Arc::new(move |event: &str, payload: serde_json::Value| {
                let _ = handle.emit(event, payload);
            });
            let dir = if let Some(d) = std::env::var_os("LSS_DATA_DIR") {
                std::path::PathBuf::from(d)
            } else if cfg!(target_os = "android") || cfg!(target_os = "ios") {
                app.path().app_data_dir()?
            } else {
                util::app_data_dir()
            };
            let state = AppState::new(&dir, emit);
            ensure_inbox(&state);

            match server::bind(protocol::DEFAULT_PORT) {
                Ok((listener, port)) => {
                    state.port.store(port, std::sync::atomic::Ordering::Relaxed);
                    let st = state.clone();
                    tauri::async_runtime::spawn(async move { server::run_server(st, listener).await });
                }
                Err(e) => log::error!("cannot start file server: {}", e),
            }
            let st = state.clone();
            tauri::async_runtime::spawn(async move { discovery::start(st) });

            app.manage(state);
            app.manage(transfer::new_cancel_map());
            #[cfg(desktop)]
            setup_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            #[cfg(desktop)]
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let to_tray = window
                    .app_handle()
                    .try_state::<AppState>()
                    .map(|s| s.db.get_setting("close_to_tray").map(|v| v != "false").unwrap_or(true))
                    .unwrap_or(false);
                if to_tray {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
            #[cfg(not(desktop))]
            let _ = (window, event);
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_identity,
            commands::set_device_name,
            commands::get_settings,
            commands::set_settings,
            commands::get_download_dir,
            commands::get_devices,
            commands::add_device_by_ip,
            commands::forget_device,
            commands::get_interfaces,
            commands::respond_approval,
            commands::list_trusted,
            commands::untrust_device,
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
            commands::start_download,
            commands::start_upload,
            commands::cancel_transfer,
            commands::get_transfer_history,
            commands::clear_transfer_history,
            commands::get_groups,
            commands::add_group,
            commands::delete_group,
            commands::network_diagnostics,
            commands::speed_test,
            commands::get_local_storage,
            commands::sync_local_to_remote,
            commands::sync_remote_to_local,
            commands::list_local_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running LocalSendStudio");
}
