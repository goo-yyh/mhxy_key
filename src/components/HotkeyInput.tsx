import { Button, Space, Tag, Tooltip } from "antd";
import { CloseOutlined, KeyOutlined } from "@ant-design/icons";
import type { Hotkey, HotkeyCode, HotkeyModifier } from "../types/config";

const modifierOrder: HotkeyModifier[] = ["control", "alt", "shift", "meta"];

const codeLabels: Record<HotkeyCode, string> = {
  KeyA: "A",
  KeyB: "B",
  KeyC: "C",
  KeyD: "D",
  KeyE: "E",
  KeyF: "F",
  KeyG: "G",
  KeyH: "H",
  KeyI: "I",
  KeyJ: "J",
  KeyK: "K",
  KeyL: "L",
  KeyM: "M",
  KeyN: "N",
  KeyO: "O",
  KeyP: "P",
  KeyQ: "Q",
  KeyR: "R",
  KeyS: "S",
  KeyT: "T",
  KeyU: "U",
  KeyV: "V",
  KeyW: "W",
  KeyX: "X",
  KeyY: "Y",
  KeyZ: "Z",
  Digit0: "0",
  Digit1: "1",
  Digit2: "2",
  Digit3: "3",
  Digit4: "4",
  Digit5: "5",
  Digit6: "6",
  Digit7: "7",
  Digit8: "8",
  Digit9: "9",
  F1: "F1",
  F2: "F2",
  F3: "F3",
  F4: "F4",
  F5: "F5",
  F6: "F6",
  F7: "F7",
  F8: "F8",
  F9: "F9",
  F10: "F10",
  F11: "F11",
  F12: "F12",
  Escape: "Esc",
  Enter: "Enter",
  Tab: "Tab",
  Space: "Space",
  Backspace: "Backspace",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
};

const supportedCodes = new Set<HotkeyCode>(Object.keys(codeLabels) as HotkeyCode[]);

export interface HotkeyInputProps {
  value?: Hotkey | null;
  onChange?: (value: Hotkey | null) => void;
  allowClear?: boolean;
  placeholder?: string;
}

export function HotkeyInput({
  value,
  onChange,
  allowClear = false,
  placeholder = "点击后按快捷键",
}: HotkeyInputProps) {
  const label = value ? formatHotkey(value) : placeholder;

  return (
    <Space.Compact className="hotkey-input">
      <Tooltip title="点击后按下键盘组合键">
        <Button
          icon={<KeyOutlined />}
          onKeyDown={(event) => {
            event.preventDefault();
            event.stopPropagation();
            const hotkey = eventToHotkey(event);
            if (hotkey) {
              onChange?.(hotkey);
            }
          }}
        >
          <span className={value ? "hotkey-value" : "hotkey-placeholder"}>{label}</span>
        </Button>
      </Tooltip>
      {allowClear ? (
        <Button
          aria-label="清空快捷键"
          icon={<CloseOutlined />}
          disabled={!value}
          onClick={() => onChange?.(null)}
        />
      ) : null}
    </Space.Compact>
  );
}

export function HotkeyTag({ hotkey }: { hotkey?: Hotkey | null }) {
  if (!hotkey) {
    return <Tag>未设置</Tag>;
  }
  return <Tag color="blue">{formatHotkey(hotkey)}</Tag>;
}

export function formatHotkey(hotkey: Hotkey): string {
  const platform = navigator.platform.toLowerCase();
  const isMac = platform.includes("mac");
  const modifiers = normalizeModifiers(hotkey.modifiers).map((modifier) => {
    if (modifier === "control") return isMac ? "Control" : "Ctrl";
    if (modifier === "alt") return isMac ? "Option" : "Alt";
    if (modifier === "shift") return "Shift";
    return isMac ? "Command" : "Win";
  });
  return [...modifiers, codeLabels[hotkey.code]].join(" + ");
}

function eventToHotkey(event: React.KeyboardEvent<HTMLElement>): Hotkey | null {
  const code = event.code as HotkeyCode;
  if (!supportedCodes.has(code)) {
    return null;
  }

  const modifiers: HotkeyModifier[] = [];
  if (event.ctrlKey) modifiers.push("control");
  if (event.altKey) modifiers.push("alt");
  if (event.shiftKey) modifiers.push("shift");
  if (event.metaKey) modifiers.push("meta");

  return {
    modifiers: normalizeModifiers(modifiers),
    code,
  };
}

function normalizeModifiers(modifiers: HotkeyModifier[]): HotkeyModifier[] {
  return modifierOrder.filter((modifier) => modifiers.includes(modifier));
}
