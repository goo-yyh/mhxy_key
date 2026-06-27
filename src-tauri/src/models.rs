use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub version: u32,
    pub global_enabled: bool,
    pub global_toggle_hotkey: Option<Hotkey>,
    pub configs: Vec<MacroConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            global_enabled: true,
            global_toggle_hotkey: None,
            configs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub trigger_hotkey: Hotkey,
    pub toggle_hotkey: Option<Hotkey>,
    pub actions: Vec<Action>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MacroConfig {
    pub fn from_input(input: MacroConfigInput) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: input.name,
            enabled: input.enabled,
            trigger_hotkey: input.trigger_hotkey,
            toggle_hotkey: input.toggle_hotkey,
            actions: input.actions,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_from_input(&mut self, input: MacroConfigInput) {
        self.name = input.name;
        self.enabled = input.enabled;
        self.trigger_hotkey = input.trigger_hotkey;
        self.toggle_hotkey = input.toggle_hotkey;
        self.actions = input.actions;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroConfigInput {
    pub name: String,
    pub enabled: bool,
    pub trigger_hotkey: Hotkey,
    pub toggle_hotkey: Option<Hotkey>,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Hotkey {
    #[serde(default)]
    pub modifiers: Vec<HotkeyModifier>,
    pub code: HotkeyCode,
}

impl Hotkey {
    pub fn normalized(&self) -> Self {
        let mut modifiers = self.modifiers.clone();
        modifiers.sort_by_key(|modifier| modifier.sort_order());
        modifiers.dedup();
        Self {
            modifiers,
            code: self.code.clone(),
        }
    }

    pub fn display_label(&self) -> String {
        let normalized = self.normalized();
        let mut parts: Vec<String> = normalized
            .modifiers
            .iter()
            .map(|modifier| modifier.display_label().to_string())
            .collect();
        parts.push(normalized.code.display_label().to_string());
        parts.join(" + ")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub enum HotkeyModifier {
    Control,
    Alt,
    Shift,
    Meta,
}

impl HotkeyModifier {
    pub fn sort_order(self) -> u8 {
        match self {
            Self::Control => 0,
            Self::Alt => 1,
            Self::Shift => 2,
            Self::Meta => 3,
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Control => {
                if cfg!(target_os = "macos") {
                    "Control"
                } else {
                    "Ctrl"
                }
            }
            Self::Alt => {
                if cfg!(target_os = "macos") {
                    "Option"
                } else {
                    "Alt"
                }
            }
            Self::Shift => "Shift",
            Self::Meta => {
                if cfg!(target_os = "macos") {
                    "Command"
                } else {
                    "Win"
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HotkeyCode {
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Escape,
    Enter,
    Tab,
    Space,
    Backspace,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

impl HotkeyCode {
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::KeyA => "A",
            Self::KeyB => "B",
            Self::KeyC => "C",
            Self::KeyD => "D",
            Self::KeyE => "E",
            Self::KeyF => "F",
            Self::KeyG => "G",
            Self::KeyH => "H",
            Self::KeyI => "I",
            Self::KeyJ => "J",
            Self::KeyK => "K",
            Self::KeyL => "L",
            Self::KeyM => "M",
            Self::KeyN => "N",
            Self::KeyO => "O",
            Self::KeyP => "P",
            Self::KeyQ => "Q",
            Self::KeyR => "R",
            Self::KeyS => "S",
            Self::KeyT => "T",
            Self::KeyU => "U",
            Self::KeyV => "V",
            Self::KeyW => "W",
            Self::KeyX => "X",
            Self::KeyY => "Y",
            Self::KeyZ => "Z",
            Self::Digit0 => "0",
            Self::Digit1 => "1",
            Self::Digit2 => "2",
            Self::Digit3 => "3",
            Self::Digit4 => "4",
            Self::Digit5 => "5",
            Self::Digit6 => "6",
            Self::Digit7 => "7",
            Self::Digit8 => "8",
            Self::Digit9 => "9",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::Escape => "Esc",
            Self::Enter => "Enter",
            Self::Tab => "Tab",
            Self::Space => "Space",
            Self::Backspace => "Backspace",
            Self::ArrowUp => "Up",
            Self::ArrowDown => "Down",
            Self::ArrowLeft => "Left",
            Self::ArrowRight => "Right",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    KeyCombo {
        keys: Hotkey,
        #[serde(default)]
        delay_after_ms: u64,
    },
    MouseClick {
        button: MouseButton,
        #[serde(default = "default_click_count")]
        click_count: u8,
        #[serde(default)]
        delay_after_ms: u64,
    },
    Delay {
        duration_ms: u64,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

fn default_click_count() -> u8 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportMode {
    Replace,
    Append,
}
