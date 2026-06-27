export type HotkeyModifier = "control" | "alt" | "shift" | "meta";

export type HotkeyCode =
  | "KeyA"
  | "KeyB"
  | "KeyC"
  | "KeyD"
  | "KeyE"
  | "KeyF"
  | "KeyG"
  | "KeyH"
  | "KeyI"
  | "KeyJ"
  | "KeyK"
  | "KeyL"
  | "KeyM"
  | "KeyN"
  | "KeyO"
  | "KeyP"
  | "KeyQ"
  | "KeyR"
  | "KeyS"
  | "KeyT"
  | "KeyU"
  | "KeyV"
  | "KeyW"
  | "KeyX"
  | "KeyY"
  | "KeyZ"
  | "Digit0"
  | "Digit1"
  | "Digit2"
  | "Digit3"
  | "Digit4"
  | "Digit5"
  | "Digit6"
  | "Digit7"
  | "Digit8"
  | "Digit9"
  | "F1"
  | "F2"
  | "F3"
  | "F4"
  | "F5"
  | "F6"
  | "F7"
  | "F8"
  | "F9"
  | "F10"
  | "F11"
  | "F12"
  | "Escape"
  | "Enter"
  | "Tab"
  | "Space"
  | "Backspace"
  | "ArrowUp"
  | "ArrowDown"
  | "ArrowLeft"
  | "ArrowRight";

export interface Hotkey {
  modifiers: HotkeyModifier[];
  code: HotkeyCode;
}

export type MouseButton = "left" | "right" | "middle";

export type Action =
  | {
      type: "keyCombo";
      keys: Hotkey;
      delayAfterMs?: number;
    }
  | {
      type: "mouseClick";
      button: MouseButton;
      clickCount?: 1 | 2;
      delayAfterMs?: number;
    }
  | {
      type: "delay";
      durationMs: number;
    };

export interface MacroConfig {
  id: string;
  name: string;
  enabled: boolean;
  triggerHotkey: Hotkey;
  toggleHotkey?: Hotkey | null;
  actions: Action[];
  createdAt: string;
  updatedAt: string;
}

export interface MacroConfigInput {
  name: string;
  enabled: boolean;
  triggerHotkey: Hotkey;
  toggleHotkey?: Hotkey | null;
  actions: Action[];
}

export interface AppConfig {
  version: number;
  globalEnabled: boolean;
  globalToggleHotkey?: Hotkey | null;
  configs: MacroConfig[];
}

export interface RuntimeStatus {
  configErrors: Record<string, string>;
  globalError?: string | null;
  configPath: string;
}

export interface CommandError {
  code: string;
  message: string;
  configId?: string | null;
}

export type ImportMode = "replace" | "append";

export function defaultHotkey(): Hotkey {
  return { modifiers: ["alt"], code: "KeyA" };
}

export function defaultAction(): Action {
  return { type: "keyCombo", keys: { modifiers: ["meta"], code: "KeyA" }, delayAfterMs: 0 };
}
