use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};

use crate::{
    app_logic,
    errors::{AppError, AppResult},
    state::{AppState, TRAY_ID},
};

const MENU_OPEN: &str = "open";
const MENU_TOGGLE_GLOBAL: &str = "toggle-global";
const MENU_EXIT: &str = "exit";
const MENU_CONFIG_PREFIX: &str = "config:";

pub fn setup_tray(app: &AppHandle<Wry>) -> AppResult<()> {
    let menu = build_menu(app)?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| AppError::Tray("缺少应用图标".to_string()))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("无敌小铃铛")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            handle_menu_event(app, event.id().as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    let _ = app_logic::show_main_window(tray.app_handle());
                }
            }
        })
        .build(app)
        .map_err(|err| AppError::Tray(err.to_string()))?;

    Ok(())
}

pub fn refresh_tray_menu(app: &AppHandle<Wry>) -> AppResult<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let menu = build_menu(app)?;
        tray.set_menu(Some(menu))
            .map_err(|err| AppError::Tray(err.to_string()))?;
    }
    Ok(())
}

fn build_menu(app: &AppHandle<Wry>) -> AppResult<tauri::menu::Menu<Wry>> {
    let config = app
        .state::<AppState>()
        .config
        .lock()
        .expect("config state poisoned")
        .clone();

    let global_label = if config.global_enabled {
        "全局总开关：关闭"
    } else {
        "全局总开关：开启"
    };

    let mut builder = MenuBuilder::new(app)
        .text(MENU_OPEN, "打开主窗口")
        .text(MENU_TOGGLE_GLOBAL, global_label)
        .separator();

    for item in &config.configs {
        let status = if item.enabled { "关闭" } else { "开启" };
        builder = builder.text(
            format!("{MENU_CONFIG_PREFIX}{}", item.id),
            format!("{}：{}", item.name, status),
        );
    }

    builder
        .separator()
        .text(MENU_EXIT, "退出")
        .build()
        .map_err(|err| AppError::Tray(err.to_string()))
}

fn handle_menu_event(app: &AppHandle<Wry>, id: &str) {
    match id {
        MENU_OPEN => {
            let _ = app_logic::show_main_window(app);
        }
        MENU_TOGGLE_GLOBAL => {
            let next = !app_logic::current_config(app).global_enabled;
            let _ = app_logic::set_global_enabled(app, next);
        }
        MENU_EXIT => {
            let _ = app_logic::exit_app(app);
        }
        value if value.starts_with(MENU_CONFIG_PREFIX) => {
            let config_id = value.trim_start_matches(MENU_CONFIG_PREFIX).to_string();
            let config = app_logic::current_config(app);
            if let Some(item) = config.configs.iter().find(|item| item.id == config_id) {
                let _ = app_logic::set_config_enabled(app, config_id, !item.enabled);
            }
        }
        _ => {}
    }
}
