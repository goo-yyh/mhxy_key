use std::collections::{HashMap, HashSet};

use crate::{
    errors::{AppError, AppResult},
    models::{Action, AppConfig, Hotkey, MouseButton, CONFIG_VERSION},
};

const MAX_DELAY_MS: u64 = 60_000;

pub fn validate_app_config(config: &AppConfig) -> AppResult<()> {
    if config.version != CONFIG_VERSION {
        return Err(AppError::Validation(format!(
            "不支持的配置版本：{}",
            config.version
        )));
    }

    let mut ids = HashSet::new();
    let mut enabled_triggers: HashMap<Hotkey, String> = HashMap::new();
    let mut toggles: HashMap<Hotkey, String> = HashMap::new();

    if let Some(hotkey) = &config.global_toggle_hotkey {
        validate_hotkey(hotkey)?;
    }

    for item in &config.configs {
        if !ids.insert(item.id.clone()) {
            return Err(AppError::Validation(format!("配置 ID 重复：{}", item.id)));
        }

        if item.name.trim().is_empty() {
            return Err(AppError::Validation("配置名称不能为空".to_string()));
        }

        validate_hotkey(&item.trigger_hotkey)?;

        if item.actions.is_empty() {
            return Err(AppError::Validation(format!(
                "配置「{}」至少需要一个动作",
                item.name
            )));
        }

        for action in &item.actions {
            validate_action(action)?;
        }

        let trigger = item.trigger_hotkey.normalized();

        if item.enabled {
            if let Some(existing_name) = enabled_triggers.insert(trigger.clone(), item.name.clone())
            {
                return Err(AppError::Validation(format!(
                    "触发快捷键「{}」与配置「{}」冲突",
                    trigger.display_label(),
                    existing_name
                )));
            }
        }

        if let Some(toggle_hotkey) = &item.toggle_hotkey {
            validate_hotkey(toggle_hotkey)?;
            let toggle = toggle_hotkey.normalized();
            if trigger == toggle {
                return Err(AppError::Validation(format!(
                    "配置「{}」的触发快捷键和启停快捷键不能相同",
                    item.name
                )));
            }

            if let Some(existing_name) = toggles.insert(toggle.clone(), item.name.clone()) {
                return Err(AppError::Validation(format!(
                    "启停快捷键「{}」与配置「{}」冲突",
                    toggle.display_label(),
                    existing_name
                )));
            }
        }
    }

    if let Some(global_toggle) = &config.global_toggle_hotkey {
        let normalized = global_toggle.normalized();
        if let Some(name) = enabled_triggers
            .get(&normalized)
            .or_else(|| toggles.get(&normalized))
        {
            return Err(AppError::Validation(format!(
                "全局启停快捷键「{}」与配置「{}」冲突",
                normalized.display_label(),
                name
            )));
        }
    }

    Ok(())
}

pub fn validate_hotkey(hotkey: &Hotkey) -> AppResult<()> {
    let normalized = hotkey.normalized();
    if normalized.modifiers.len() > 4 {
        return Err(AppError::Validation("快捷键修饰键过多".to_string()));
    }
    Ok(())
}

pub fn validate_action(action: &Action) -> AppResult<()> {
    match action {
        Action::KeyCombo {
            keys,
            delay_after_ms,
        } => {
            validate_hotkey(keys)?;
            validate_delay_after(*delay_after_ms)?;
        }
        Action::MouseClick {
            button,
            click_count,
            delay_after_ms,
        } => {
            validate_mouse_button(*button)?;
            if !matches!(click_count, 1 | 2) {
                return Err(AppError::Validation(
                    "鼠标点击次数只能是 1 或 2".to_string(),
                ));
            }
            validate_delay_after(*delay_after_ms)?;
        }
        Action::Delay { duration_ms } => {
            if *duration_ms == 0 || *duration_ms > MAX_DELAY_MS {
                return Err(AppError::Validation(format!(
                    "等待时间需要在 1 到 {} 毫秒之间",
                    MAX_DELAY_MS
                )));
            }
        }
    }
    Ok(())
}

fn validate_delay_after(delay_after_ms: u64) -> AppResult<()> {
    if delay_after_ms > MAX_DELAY_MS {
        return Err(AppError::Validation(format!(
            "动作后等待时间不能超过 {} 毫秒",
            MAX_DELAY_MS
        )));
    }
    Ok(())
}

fn validate_mouse_button(button: MouseButton) -> AppResult<()> {
    match button {
        MouseButton::Left | MouseButton::Right | MouseButton::Middle => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Hotkey, HotkeyCode, HotkeyModifier};

    use super::*;

    #[test]
    fn normalizes_hotkey_modifiers() {
        let hotkey = Hotkey {
            modifiers: vec![
                HotkeyModifier::Meta,
                HotkeyModifier::Control,
                HotkeyModifier::Meta,
            ],
            code: HotkeyCode::KeyA,
        };

        assert_eq!(
            hotkey.normalized().modifiers,
            vec![HotkeyModifier::Control, HotkeyModifier::Meta]
        );
    }

    #[test]
    fn rejects_invalid_click_count() {
        let action = Action::MouseClick {
            button: MouseButton::Left,
            click_count: 3,
            delay_after_ms: 0,
        };

        assert!(validate_action(&action).is_err());
    }
}
