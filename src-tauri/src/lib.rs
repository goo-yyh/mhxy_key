mod app_logic;
mod commands;
mod config_store;
mod errors;
mod executor;
mod hotkeys;
mod input_simulator;
mod models;
mod state;
mod tray;
mod validation;

use config_store::ConfigStore;
use executor::ActionExecutor;
use models::AppConfig;
use state::{AppState, MAIN_WINDOW_LABEL};
use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_data_dir = app.path().app_data_dir()?;
            let store = ConfigStore::new(app_data_dir)?;
            let (config, load_error) = match store.load_or_create() {
                Ok(config) => (config, None),
                Err(err) => (AppConfig::default(), Some(err.to_string())),
            };
            let executor = ActionExecutor::new(app_handle.clone());

            app.manage(AppState::new(config, store, executor));

            tray::setup_tray(&app_handle)?;
            app_logic::refresh_runtime(&app_handle);

            if let Some(message) = load_error {
                app_logic::emit_load_failed(&app_handle, message);
            }

            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let handle = app_handle.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = app_logic::exit_app(&handle);
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::get_runtime_status,
            commands::save_config,
            commands::create_config,
            commands::update_config,
            commands::delete_config,
            commands::set_global_enabled,
            commands::set_global_toggle_hotkey,
            commands::set_config_enabled,
            commands::import_config,
            commands::export_config,
            commands::test_hotkey,
            commands::hide_main_window,
            commands::show_main_window,
            commands::exit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
