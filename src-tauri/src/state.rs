use std::{collections::HashMap, path::PathBuf, sync::Mutex, time::Instant};

use crate::{
    config_store::ConfigStore,
    executor::ActionExecutor,
    models::{AppConfig, Hotkey},
};

pub const TRAY_ID: &str = "main-tray";
pub const MAIN_WINDOW_LABEL: &str = "main";
pub const TRIGGER_DEBOUNCE_MS: u128 = 300;

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub store: ConfigStore,
    pub runtime: Mutex<RuntimeState>,
    pub executor: ActionExecutor,
}

impl AppState {
    pub fn new(config: AppConfig, store: ConfigStore, executor: ActionExecutor) -> Self {
        Self {
            config: Mutex::new(config),
            store,
            runtime: Mutex::new(RuntimeState::default()),
            executor,
        }
    }
}

#[derive(Debug, Default)]
pub struct RuntimeState {
    pub last_trigger_at: HashMap<String, Instant>,
    pub config_errors: HashMap<String, String>,
    pub global_error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum HotkeyRole {
    Trigger { config_id: String },
    ToggleConfig { config_id: String },
    ToggleGlobal,
}

impl HotkeyRole {
    pub fn config_id(&self) -> Option<&str> {
        match self {
            Self::Trigger { config_id } | Self::ToggleConfig { config_id } => Some(config_id),
            Self::ToggleGlobal => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub config_errors: HashMap<String, String>,
    pub global_error: Option<String>,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct HotkeyRegistration {
    pub hotkey: Hotkey,
    pub role: HotkeyRole,
}
