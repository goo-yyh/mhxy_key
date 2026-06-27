import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  CommandError,
  Hotkey,
  ImportMode,
  MacroConfigInput,
  RuntimeStatus,
} from "../types/config";

function normalizeError(error: unknown): Error {
  if (error && typeof error === "object" && "message" in error) {
    const commandError = error as CommandError;
    return new Error(commandError.message);
  }
  return new Error(String(error));
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeError(error);
  }
}

export function getConfig(): Promise<AppConfig> {
  return call("get_config");
}

export function getRuntimeStatus(): Promise<RuntimeStatus> {
  return call("get_runtime_status");
}

export function saveConfig(config: AppConfig): Promise<AppConfig> {
  return call("save_config", { config });
}

export function createConfig(input: MacroConfigInput): Promise<AppConfig> {
  return call("create_config", { input });
}

export function updateConfig(id: string, input: MacroConfigInput): Promise<AppConfig> {
  return call("update_config", { id, input });
}

export function deleteConfig(id: string): Promise<AppConfig> {
  return call("delete_config", { id });
}

export function setGlobalEnabled(enabled: boolean): Promise<AppConfig> {
  return call("set_global_enabled", { enabled });
}

export function setGlobalToggleHotkey(hotkey: Hotkey | null): Promise<AppConfig> {
  return call("set_global_toggle_hotkey", { hotkey });
}

export function setConfigEnabled(id: string, enabled: boolean): Promise<AppConfig> {
  return call("set_config_enabled", { id, enabled });
}

export function importConfig(path: string, mode: ImportMode): Promise<AppConfig> {
  return call("import_config", { path, mode });
}

export function exportConfig(path: string): Promise<void> {
  return call("export_config", { path });
}

export function testHotkey(hotkey: Hotkey): Promise<void> {
  return call("test_hotkey", { hotkey });
}

export function hideMainWindow(): Promise<void> {
  return call("hide_main_window");
}

export function showMainWindow(): Promise<void> {
  return call("show_main_window");
}

export function exitApp(): Promise<void> {
  return call("exit_app");
}
