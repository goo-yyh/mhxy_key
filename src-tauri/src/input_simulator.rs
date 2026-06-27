use std::{thread, time::Duration};

use enigo::{Button as EnigoButton, Direction, Enigo, Key as EnigoKey, Keyboard, Mouse, Settings};

use crate::{
    errors::{AppError, AppResult},
    models::{Hotkey, HotkeyCode, HotkeyModifier, MouseButton},
};

pub struct EnigoInputSimulator {
    enigo: Enigo,
}

impl EnigoInputSimulator {
    pub fn new() -> AppResult<Self> {
        let enigo =
            Enigo::new(&Settings::default()).map_err(|err| AppError::Action(err.to_string()))?;
        Ok(Self { enigo })
    }

    pub fn key_combo(&mut self, hotkey: &Hotkey) -> AppResult<()> {
        let normalized = hotkey.normalized();
        let mut pressed = Vec::new();

        for modifier in &normalized.modifiers {
            let key = modifier_to_enigo(*modifier);
            if let Err(err) = self.enigo.key(key, Direction::Press) {
                self.release_pressed(&pressed);
                return Err(AppError::Action(err.to_string()));
            }
            pressed.push(key);
        }

        let key = code_to_enigo(&normalized.code);
        if let Err(err) = self.enigo.key(key, Direction::Click) {
            self.release_pressed(&pressed);
            return Err(AppError::Action(err.to_string()));
        }

        self.release_pressed(&pressed);
        Ok(())
    }

    pub fn mouse_click(&mut self, button: MouseButton, click_count: u8) -> AppResult<()> {
        let button = button_to_enigo(button);
        for index in 0..click_count {
            self.enigo
                .button(button, Direction::Click)
                .map_err(|err| AppError::Action(err.to_string()))?;
            if index + 1 < click_count {
                thread::sleep(Duration::from_millis(40));
            }
        }
        Ok(())
    }

    pub fn release_all(&mut self) {
        let modifiers = [
            EnigoKey::Control,
            EnigoKey::Alt,
            EnigoKey::Shift,
            EnigoKey::Meta,
        ];
        self.release_pressed(&modifiers);
    }

    fn release_pressed(&mut self, pressed: &[EnigoKey]) {
        for key in pressed.iter().rev() {
            let _ = self.enigo.key(*key, Direction::Release);
        }
    }
}

fn modifier_to_enigo(modifier: HotkeyModifier) -> EnigoKey {
    match modifier {
        HotkeyModifier::Control => EnigoKey::Control,
        HotkeyModifier::Alt => EnigoKey::Alt,
        HotkeyModifier::Shift => EnigoKey::Shift,
        HotkeyModifier::Meta => EnigoKey::Meta,
    }
}

fn code_to_enigo(code: &HotkeyCode) -> EnigoKey {
    match code {
        HotkeyCode::KeyA => EnigoKey::Unicode('a'),
        HotkeyCode::KeyB => EnigoKey::Unicode('b'),
        HotkeyCode::KeyC => EnigoKey::Unicode('c'),
        HotkeyCode::KeyD => EnigoKey::Unicode('d'),
        HotkeyCode::KeyE => EnigoKey::Unicode('e'),
        HotkeyCode::KeyF => EnigoKey::Unicode('f'),
        HotkeyCode::KeyG => EnigoKey::Unicode('g'),
        HotkeyCode::KeyH => EnigoKey::Unicode('h'),
        HotkeyCode::KeyI => EnigoKey::Unicode('i'),
        HotkeyCode::KeyJ => EnigoKey::Unicode('j'),
        HotkeyCode::KeyK => EnigoKey::Unicode('k'),
        HotkeyCode::KeyL => EnigoKey::Unicode('l'),
        HotkeyCode::KeyM => EnigoKey::Unicode('m'),
        HotkeyCode::KeyN => EnigoKey::Unicode('n'),
        HotkeyCode::KeyO => EnigoKey::Unicode('o'),
        HotkeyCode::KeyP => EnigoKey::Unicode('p'),
        HotkeyCode::KeyQ => EnigoKey::Unicode('q'),
        HotkeyCode::KeyR => EnigoKey::Unicode('r'),
        HotkeyCode::KeyS => EnigoKey::Unicode('s'),
        HotkeyCode::KeyT => EnigoKey::Unicode('t'),
        HotkeyCode::KeyU => EnigoKey::Unicode('u'),
        HotkeyCode::KeyV => EnigoKey::Unicode('v'),
        HotkeyCode::KeyW => EnigoKey::Unicode('w'),
        HotkeyCode::KeyX => EnigoKey::Unicode('x'),
        HotkeyCode::KeyY => EnigoKey::Unicode('y'),
        HotkeyCode::KeyZ => EnigoKey::Unicode('z'),
        HotkeyCode::Digit0 => EnigoKey::Unicode('0'),
        HotkeyCode::Digit1 => EnigoKey::Unicode('1'),
        HotkeyCode::Digit2 => EnigoKey::Unicode('2'),
        HotkeyCode::Digit3 => EnigoKey::Unicode('3'),
        HotkeyCode::Digit4 => EnigoKey::Unicode('4'),
        HotkeyCode::Digit5 => EnigoKey::Unicode('5'),
        HotkeyCode::Digit6 => EnigoKey::Unicode('6'),
        HotkeyCode::Digit7 => EnigoKey::Unicode('7'),
        HotkeyCode::Digit8 => EnigoKey::Unicode('8'),
        HotkeyCode::Digit9 => EnigoKey::Unicode('9'),
        HotkeyCode::F1 => EnigoKey::F1,
        HotkeyCode::F2 => EnigoKey::F2,
        HotkeyCode::F3 => EnigoKey::F3,
        HotkeyCode::F4 => EnigoKey::F4,
        HotkeyCode::F5 => EnigoKey::F5,
        HotkeyCode::F6 => EnigoKey::F6,
        HotkeyCode::F7 => EnigoKey::F7,
        HotkeyCode::F8 => EnigoKey::F8,
        HotkeyCode::F9 => EnigoKey::F9,
        HotkeyCode::F10 => EnigoKey::F10,
        HotkeyCode::F11 => EnigoKey::F11,
        HotkeyCode::F12 => EnigoKey::F12,
        HotkeyCode::Escape => EnigoKey::Escape,
        HotkeyCode::Enter => EnigoKey::Return,
        HotkeyCode::Tab => EnigoKey::Tab,
        HotkeyCode::Space => EnigoKey::Space,
        HotkeyCode::Backspace => EnigoKey::Backspace,
        HotkeyCode::ArrowUp => EnigoKey::UpArrow,
        HotkeyCode::ArrowDown => EnigoKey::DownArrow,
        HotkeyCode::ArrowLeft => EnigoKey::LeftArrow,
        HotkeyCode::ArrowRight => EnigoKey::RightArrow,
    }
}

fn button_to_enigo(button: MouseButton) -> EnigoButton {
    match button {
        MouseButton::Left => EnigoButton::Left,
        MouseButton::Right => EnigoButton::Right,
        MouseButton::Middle => EnigoButton::Middle,
    }
}
