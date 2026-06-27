use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use atomic_write_file::AtomicWriteFile;
use chrono::Utc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::{AppConfig, ImportMode},
    validation::validate_app_config,
};

pub const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(app_data_dir: PathBuf) -> AppResult<Self> {
        fs::create_dir_all(&app_data_dir).map_err(|err| AppError::ConfigWrite(err.to_string()))?;
        Ok(Self {
            path: app_data_dir.join(CONFIG_FILE_NAME),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load_or_create(&self) -> AppResult<AppConfig> {
        if !self.path.exists() {
            let config = AppConfig::default();
            self.save(&config)?;
            return Ok(config);
        }

        match self.load() {
            Ok(config) => Ok(config),
            Err(err) => {
                let backup = self.backup_corrupt_file()?;
                let config = AppConfig::default();
                self.save(&config)?;
                Err(AppError::ConfigRead(format!(
                    "{}。已备份损坏配置到 {}",
                    err,
                    backup.display()
                )))
            }
        }
    }

    pub fn load(&self) -> AppResult<AppConfig> {
        let raw =
            fs::read_to_string(&self.path).map_err(|err| AppError::ConfigRead(err.to_string()))?;
        let config: AppConfig =
            serde_json::from_str(&raw).map_err(|err| AppError::ConfigRead(err.to_string()))?;
        validate_app_config(&config)?;
        Ok(config)
    }

    pub fn save(&self, config: &AppConfig) -> AppResult<()> {
        validate_app_config(config)?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|err| AppError::ConfigWrite(err.to_string()))?;
        }

        let raw = serde_json::to_string_pretty(config)
            .map_err(|err| AppError::ConfigWrite(err.to_string()))?;
        let mut file = AtomicWriteFile::open(&self.path)
            .map_err(|err| AppError::ConfigWrite(err.to_string()))?;
        file.write_all(raw.as_bytes())
            .map_err(|err| AppError::ConfigWrite(err.to_string()))?;
        file.commit()
            .map_err(|err| AppError::ConfigWrite(err.to_string()))?;
        Ok(())
    }

    pub fn export_to(&self, config: &AppConfig, path: impl AsRef<Path>) -> AppResult<()> {
        validate_app_config(config)?;
        let raw = serde_json::to_string_pretty(config)
            .map_err(|err| AppError::ConfigWrite(err.to_string()))?;
        fs::write(path, raw).map_err(|err| AppError::ConfigWrite(err.to_string()))
    }

    pub fn read_import_file(path: impl AsRef<Path>) -> AppResult<AppConfig> {
        let raw = fs::read_to_string(path).map_err(|err| AppError::ConfigRead(err.to_string()))?;
        let config: AppConfig =
            serde_json::from_str(&raw).map_err(|err| AppError::ConfigRead(err.to_string()))?;
        validate_app_config(&config)?;
        Ok(config)
    }

    pub fn merge_import(
        current: &AppConfig,
        imported: AppConfig,
        mode: ImportMode,
    ) -> AppResult<AppConfig> {
        match mode {
            ImportMode::Replace => {
                validate_app_config(&imported)?;
                Ok(imported)
            }
            ImportMode::Append => {
                let mut next = current.clone();
                let mut known_ids = next
                    .configs
                    .iter()
                    .map(|config| config.id.clone())
                    .collect::<std::collections::HashSet<_>>();

                for mut item in imported.configs {
                    if known_ids.contains(&item.id) {
                        item.id = Uuid::new_v4().to_string();
                    }
                    known_ids.insert(item.id.clone());
                    next.configs.push(item);
                }

                validate_app_config(&next)?;
                Ok(next)
            }
        }
    }

    fn backup_corrupt_file(&self) -> AppResult<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let backup_path = self
            .path
            .with_file_name(format!("config.corrupt.{timestamp}.json"));
        fs::rename(&self.path, &backup_path)
            .map_err(|err| AppError::ConfigRead(err.to_string()))?;
        Ok(backup_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Action, Hotkey, HotkeyCode, MacroConfig, MouseButton};

    use super::*;

    #[test]
    fn append_import_rewrites_duplicate_ids() {
        let hotkey = Hotkey {
            modifiers: vec![],
            code: HotkeyCode::KeyA,
        };
        let config = MacroConfig {
            id: "same".to_string(),
            name: "one".to_string(),
            enabled: false,
            trigger_hotkey: hotkey.clone(),
            toggle_hotkey: None,
            actions: vec![Action::MouseClick {
                button: MouseButton::Left,
                click_count: 1,
                delay_after_ms: 0,
            }],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let current = AppConfig {
            configs: vec![config.clone()],
            ..AppConfig::default()
        };
        let imported = AppConfig {
            configs: vec![config],
            ..AppConfig::default()
        };

        let merged = ConfigStore::merge_import(&current, imported, ImportMode::Append).unwrap();
        assert_eq!(merged.configs.len(), 2);
        assert_ne!(merged.configs[0].id, merged.configs[1].id);
    }
}
