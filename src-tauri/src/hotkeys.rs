use tauri::AppHandle;
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent, ShortcutState,
};

use crate::{
    app_logic::handle_shortcut,
    errors::{AppError, AppResult},
    models::{Hotkey, HotkeyCode, HotkeyModifier},
    state::HotkeyRole,
};

pub fn register_hotkey(
    app: &AppHandle<tauri::Wry>,
    hotkey: &Hotkey,
    role: HotkeyRole,
) -> AppResult<()> {
    let shortcut = shortcut_from_hotkey(hotkey);
    app.global_shortcut()
        .on_shortcut(
            shortcut,
            move |app, _shortcut: &Shortcut, event: ShortcutEvent| {
                if event.state == ShortcutState::Pressed {
                    handle_shortcut(app.clone(), role.clone());
                }
            },
        )
        .map_err(|err| AppError::Hotkey(err.to_string()))
}

pub fn unregister_all(app: &AppHandle<tauri::Wry>) {
    let _ = app.global_shortcut().unregister_all();
}

pub fn shortcut_from_hotkey(hotkey: &Hotkey) -> Shortcut {
    let normalized = hotkey.normalized();
    let mut modifiers = Modifiers::empty();
    for modifier in normalized.modifiers {
        match modifier {
            HotkeyModifier::Control => modifiers |= Modifiers::CONTROL,
            HotkeyModifier::Alt => modifiers |= Modifiers::ALT,
            HotkeyModifier::Shift => modifiers |= Modifiers::SHIFT,
            HotkeyModifier::Meta => modifiers |= Modifiers::SUPER,
        }
    }
    Shortcut::new(Some(modifiers), code_from_hotkey(&normalized.code))
}

fn code_from_hotkey(code: &HotkeyCode) -> Code {
    match code {
        HotkeyCode::KeyA => Code::KeyA,
        HotkeyCode::KeyB => Code::KeyB,
        HotkeyCode::KeyC => Code::KeyC,
        HotkeyCode::KeyD => Code::KeyD,
        HotkeyCode::KeyE => Code::KeyE,
        HotkeyCode::KeyF => Code::KeyF,
        HotkeyCode::KeyG => Code::KeyG,
        HotkeyCode::KeyH => Code::KeyH,
        HotkeyCode::KeyI => Code::KeyI,
        HotkeyCode::KeyJ => Code::KeyJ,
        HotkeyCode::KeyK => Code::KeyK,
        HotkeyCode::KeyL => Code::KeyL,
        HotkeyCode::KeyM => Code::KeyM,
        HotkeyCode::KeyN => Code::KeyN,
        HotkeyCode::KeyO => Code::KeyO,
        HotkeyCode::KeyP => Code::KeyP,
        HotkeyCode::KeyQ => Code::KeyQ,
        HotkeyCode::KeyR => Code::KeyR,
        HotkeyCode::KeyS => Code::KeyS,
        HotkeyCode::KeyT => Code::KeyT,
        HotkeyCode::KeyU => Code::KeyU,
        HotkeyCode::KeyV => Code::KeyV,
        HotkeyCode::KeyW => Code::KeyW,
        HotkeyCode::KeyX => Code::KeyX,
        HotkeyCode::KeyY => Code::KeyY,
        HotkeyCode::KeyZ => Code::KeyZ,
        HotkeyCode::Digit0 => Code::Digit0,
        HotkeyCode::Digit1 => Code::Digit1,
        HotkeyCode::Digit2 => Code::Digit2,
        HotkeyCode::Digit3 => Code::Digit3,
        HotkeyCode::Digit4 => Code::Digit4,
        HotkeyCode::Digit5 => Code::Digit5,
        HotkeyCode::Digit6 => Code::Digit6,
        HotkeyCode::Digit7 => Code::Digit7,
        HotkeyCode::Digit8 => Code::Digit8,
        HotkeyCode::Digit9 => Code::Digit9,
        HotkeyCode::F1 => Code::F1,
        HotkeyCode::F2 => Code::F2,
        HotkeyCode::F3 => Code::F3,
        HotkeyCode::F4 => Code::F4,
        HotkeyCode::F5 => Code::F5,
        HotkeyCode::F6 => Code::F6,
        HotkeyCode::F7 => Code::F7,
        HotkeyCode::F8 => Code::F8,
        HotkeyCode::F9 => Code::F9,
        HotkeyCode::F10 => Code::F10,
        HotkeyCode::F11 => Code::F11,
        HotkeyCode::F12 => Code::F12,
        HotkeyCode::Escape => Code::Escape,
        HotkeyCode::Enter => Code::Enter,
        HotkeyCode::Tab => Code::Tab,
        HotkeyCode::Space => Code::Space,
        HotkeyCode::Backspace => Code::Backspace,
        HotkeyCode::ArrowUp => Code::ArrowUp,
        HotkeyCode::ArrowDown => Code::ArrowDown,
        HotkeyCode::ArrowLeft => Code::ArrowLeft,
        HotkeyCode::ArrowRight => Code::ArrowRight,
    }
}
