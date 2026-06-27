use tauri::{AppHandle, Wry};

use crate::{
    app_logic,
    errors::{CommandError, CommandResult},
    models::{AppConfig, Hotkey, ImportMode, MacroConfigInput},
    state::RuntimeStatus,
};

#[tauri::command]
pub fn get_config(app: AppHandle<Wry>) -> AppConfig {
    app_logic::current_config(&app)
}

#[tauri::command]
pub fn get_runtime_status(app: AppHandle<Wry>) -> RuntimeStatus {
    app_logic::runtime_status(&app)
}

#[tauri::command]
pub fn save_config(app: AppHandle<Wry>, config: AppConfig) -> CommandResult<AppConfig> {
    app_logic::save_config(&app, config).map_err(CommandError::from)
}

#[tauri::command]
pub fn create_config(app: AppHandle<Wry>, input: MacroConfigInput) -> CommandResult<AppConfig> {
    app_logic::create_config(&app, input).map_err(CommandError::from)
}

#[tauri::command]
pub fn update_config(
    app: AppHandle<Wry>,
    id: String,
    input: MacroConfigInput,
) -> CommandResult<AppConfig> {
    app_logic::update_config(&app, id, input).map_err(CommandError::from)
}

#[tauri::command]
pub fn delete_config(app: AppHandle<Wry>, id: String) -> CommandResult<AppConfig> {
    app_logic::delete_config(&app, id).map_err(CommandError::from)
}

#[tauri::command]
pub fn set_global_enabled(app: AppHandle<Wry>, enabled: bool) -> CommandResult<AppConfig> {
    app_logic::set_global_enabled(&app, enabled).map_err(CommandError::from)
}

#[tauri::command]
pub fn set_global_toggle_hotkey(
    app: AppHandle<Wry>,
    hotkey: Option<Hotkey>,
) -> CommandResult<AppConfig> {
    app_logic::set_global_toggle_hotkey(&app, hotkey).map_err(CommandError::from)
}

#[tauri::command]
pub fn set_config_enabled(
    app: AppHandle<Wry>,
    id: String,
    enabled: bool,
) -> CommandResult<AppConfig> {
    app_logic::set_config_enabled(&app, id, enabled).map_err(CommandError::from)
}

#[tauri::command]
pub fn import_config(
    app: AppHandle<Wry>,
    path: String,
    mode: ImportMode,
) -> CommandResult<AppConfig> {
    app_logic::import_config(&app, path, mode).map_err(CommandError::from)
}

#[tauri::command]
pub fn export_config(app: AppHandle<Wry>, path: String) -> CommandResult<()> {
    app_logic::export_config(&app, path).map_err(CommandError::from)
}

#[tauri::command]
pub fn test_hotkey(app: AppHandle<Wry>, hotkey: Hotkey) -> CommandResult<()> {
    app_logic::test_hotkey(&app, hotkey).map_err(CommandError::from)
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle<Wry>) -> CommandResult<()> {
    app_logic::hide_main_window(&app).map_err(CommandError::from)
}

#[tauri::command]
pub fn show_main_window(app: AppHandle<Wry>) -> CommandResult<()> {
    app_logic::show_main_window(&app).map_err(CommandError::from)
}

#[tauri::command]
pub fn exit_app(app: AppHandle<Wry>) -> CommandResult<()> {
    app_logic::exit_app(&app).map_err(CommandError::from)
}
