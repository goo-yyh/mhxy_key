use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::{
    config_store::ConfigStore,
    errors::{AppError, AppResult},
    hotkeys,
    models::{AppConfig, ImportMode, MacroConfig, MacroConfigInput},
    state::{
        AppState, HotkeyRegistration, HotkeyRole, RuntimeStatus, MAIN_WINDOW_LABEL,
        TRIGGER_DEBOUNCE_MS,
    },
    tray,
    validation::{validate_app_config, validate_hotkey},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyFailureEvent {
    pub config_id: Option<String>,
    pub hotkey: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigLoadFailedEvent {
    pub message: String,
}

pub fn emit_config(app: &AppHandle<Wry>, config: &AppConfig) {
    let _ = app.emit("config://changed", config.clone());
    let _ = app.emit("runtime://changed", runtime_status(app));
}

pub fn runtime_status(app: &AppHandle<Wry>) -> RuntimeStatus {
    let state = app.state::<AppState>();
    let runtime = state.runtime.lock().expect("runtime state poisoned");
    RuntimeStatus {
        config_errors: runtime.config_errors.clone(),
        global_error: runtime.global_error.clone(),
        config_path: state.store.path().to_path_buf(),
    }
}

pub fn current_config(app: &AppHandle<Wry>) -> AppConfig {
    let state = app.state::<AppState>();
    let config = state.config.lock().expect("config state poisoned").clone();
    config
}

pub fn create_config(app: &AppHandle<Wry>, input: MacroConfigInput) -> AppResult<AppConfig> {
    let mut config = current_config(app);
    config.configs.push(MacroConfig::from_input(input));
    persist_and_refresh(app, config)
}

pub fn update_config(
    app: &AppHandle<Wry>,
    id: String,
    input: MacroConfigInput,
) -> AppResult<AppConfig> {
    let mut config = current_config(app);
    let item = config
        .configs
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| AppError::Validation("配置不存在".to_string()))?;
    item.update_from_input(input);
    persist_and_refresh(app, config)
}

pub fn delete_config(app: &AppHandle<Wry>, id: String) -> AppResult<AppConfig> {
    let mut config = current_config(app);
    let original_len = config.configs.len();
    config.configs.retain(|item| item.id != id);
    if config.configs.len() == original_len {
        return Err(AppError::Validation("配置不存在".to_string()));
    }
    app.state::<AppState>().executor.cancel_config(&id);
    persist_and_refresh(app, config)
}

pub fn save_config(app: &AppHandle<Wry>, config: AppConfig) -> AppResult<AppConfig> {
    persist_and_refresh(app, config)
}

pub fn set_global_enabled(app: &AppHandle<Wry>, enabled: bool) -> AppResult<AppConfig> {
    let mut config = current_config(app);
    config.global_enabled = enabled;
    if !enabled {
        app.state::<AppState>().executor.cancel_all();
    }
    persist_and_refresh(app, config)
}

pub fn set_global_toggle_hotkey(
    app: &AppHandle<Wry>,
    hotkey: Option<crate::models::Hotkey>,
) -> AppResult<AppConfig> {
    if let Some(hotkey) = &hotkey {
        validate_hotkey(hotkey)?;
    }
    let mut config = current_config(app);
    config.global_toggle_hotkey = hotkey;
    persist_and_refresh(app, config)
}

pub fn set_config_enabled(app: &AppHandle<Wry>, id: String, enabled: bool) -> AppResult<AppConfig> {
    let mut config = current_config(app);
    let item = config
        .configs
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| AppError::Validation("配置不存在".to_string()))?;
    item.enabled = enabled;
    item.updated_at = chrono::Utc::now();
    if !enabled {
        app.state::<AppState>().executor.cancel_config(&id);
    }
    persist_and_refresh(app, config)
}

pub fn import_config(app: &AppHandle<Wry>, path: String, mode: ImportMode) -> AppResult<AppConfig> {
    let imported = ConfigStore::read_import_file(path)?;
    let current = current_config(app);
    let next = ConfigStore::merge_import(&current, imported, mode)?;
    persist_and_refresh(app, next)
}

pub fn export_config(app: &AppHandle<Wry>, path: String) -> AppResult<()> {
    let state = app.state::<AppState>();
    let config = current_config(app);
    state.store.export_to(&config, path)
}

pub fn test_hotkey(app: &AppHandle<Wry>, hotkey: crate::models::Hotkey) -> AppResult<()> {
    validate_hotkey(&hotkey)?;
    let mut config = current_config(app);
    config.global_toggle_hotkey = Some(hotkey);
    validate_app_config(&config)
}

pub fn persist_and_refresh(app: &AppHandle<Wry>, config: AppConfig) -> AppResult<AppConfig> {
    validate_app_config(&config)?;
    {
        let state = app.state::<AppState>();
        state.store.save(&config)?;
        *state.config.lock().expect("config state poisoned") = config.clone();
        state.executor.next_generation();
    }

    refresh_runtime(app);
    let _ = tray::refresh_tray_menu(app);
    emit_config(app, &config);
    Ok(config)
}

pub fn refresh_runtime(app: &AppHandle<Wry>) {
    let config = current_config(app);
    hotkeys::unregister_all(app);

    {
        let state = app.state::<AppState>();
        let mut runtime = state.runtime.lock().expect("runtime state poisoned");
        runtime.config_errors.clear();
        runtime.global_error = None;
    }

    let mut registrations = Vec::new();
    if let Some(hotkey) = &config.global_toggle_hotkey {
        registrations.push(HotkeyRegistration {
            hotkey: hotkey.clone(),
            role: HotkeyRole::ToggleGlobal,
        });
    }

    for item in &config.configs {
        if let Some(hotkey) = &item.toggle_hotkey {
            registrations.push(HotkeyRegistration {
                hotkey: hotkey.clone(),
                role: HotkeyRole::ToggleConfig {
                    config_id: item.id.clone(),
                },
            });
        }

        if config.global_enabled && item.enabled {
            registrations.push(HotkeyRegistration {
                hotkey: item.trigger_hotkey.clone(),
                role: HotkeyRole::Trigger {
                    config_id: item.id.clone(),
                },
            });
        }
    }

    for registration in registrations {
        let label = registration.hotkey.display_label();
        let role = registration.role.clone();
        if let Err(err) = hotkeys::register_hotkey(app, &registration.hotkey, registration.role) {
            let message = err.to_string();
            {
                let state = app.state::<AppState>();
                let mut runtime = state.runtime.lock().expect("runtime state poisoned");
                if let Some(config_id) = role.config_id() {
                    runtime
                        .config_errors
                        .insert(config_id.to_string(), message.clone());
                } else {
                    runtime.global_error = Some(message.clone());
                }
            }

            let _ = app.emit(
                "hotkey://register_failed",
                HotkeyFailureEvent {
                    config_id: role.config_id().map(ToOwned::to_owned),
                    hotkey: label,
                    message,
                },
            );
        }
    }
}

pub fn handle_shortcut(app: AppHandle<Wry>, role: HotkeyRole) {
    let event_config_id = role.config_id().map(ToOwned::to_owned);
    let result = match role {
        HotkeyRole::ToggleGlobal => {
            let next = !current_config(&app).global_enabled;
            set_global_enabled(&app, next).map(|_| ())
        }
        HotkeyRole::ToggleConfig { config_id } => {
            let config = current_config(&app);
            let enabled = config
                .configs
                .iter()
                .find(|item| item.id == config_id)
                .map(|item| !item.enabled)
                .unwrap_or(false);
            set_config_enabled(&app, config_id, enabled).map(|_| ())
        }
        HotkeyRole::Trigger { config_id } => trigger_config(&app, config_id),
    };

    if let Err(err) = result {
        let _ = app.emit(
            "action://failed",
            serde_json::json!({
                "configId": event_config_id.unwrap_or_default(),
                "message": err.to_string()
            }),
        );
    }
}

pub fn trigger_config(app: &AppHandle<Wry>, config_id: String) -> AppResult<()> {
    let config = current_config(app);
    if !config.global_enabled {
        return Ok(());
    }

    let item = match config
        .configs
        .iter()
        .find(|item| item.id == config_id && item.enabled)
    {
        Some(item) => item,
        None => return Ok(()),
    };

    {
        let state = app.state::<AppState>();
        let mut runtime = state.runtime.lock().expect("runtime state poisoned");
        let now = Instant::now();
        if let Some(last) = runtime.last_trigger_at.get(&config_id) {
            if now.duration_since(*last) < Duration::from_millis(TRIGGER_DEBOUNCE_MS as u64) {
                return Ok(());
            }
        }
        runtime.last_trigger_at.insert(config_id.clone(), now);
    }

    app.state::<AppState>()
        .executor
        .enqueue(config_id, item.actions.clone());

    Ok(())
}

pub fn hide_main_window(app: &AppHandle<Wry>) -> AppResult<()> {
    let window = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| AppError::Window("主窗口不存在".to_string()))?;
    window
        .hide()
        .map_err(|err| AppError::Window(err.to_string()))
}

pub fn show_main_window(app: &AppHandle<Wry>) -> AppResult<()> {
    let window = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| AppError::Window("主窗口不存在".to_string()))?;
    window
        .show()
        .map_err(|err| AppError::Window(err.to_string()))?;
    window
        .set_focus()
        .map_err(|err| AppError::Window(err.to_string()))
}

pub fn exit_app(app: &AppHandle<Wry>) -> AppResult<()> {
    let state = app.state::<AppState>();
    state.executor.shutdown();
    hotkeys::unregister_all(app);
    let config = current_config(app);
    state.store.save(&config)?;
    let _ = app.emit("app://exiting", serde_json::json!({}));
    app.exit(0);
    Ok(())
}

pub fn emit_load_failed(app: &AppHandle<Wry>, message: String) {
    let _ = app.emit("config://load_failed", ConfigLoadFailedEvent { message });
}
