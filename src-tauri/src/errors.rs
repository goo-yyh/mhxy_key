use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("配置读取失败：{0}")]
    ConfigRead(String),
    #[error("配置写入失败：{0}")]
    ConfigWrite(String),
    #[error("配置校验失败：{0}")]
    Validation(String),
    #[error("快捷键注册失败：{0}")]
    Hotkey(String),
    #[error("动作执行失败：{0}")]
    Action(String),
    #[error("窗口操作失败：{0}")]
    Window(String),
    #[error("托盘操作失败：{0}")]
    Tray(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub config_id: Option<String>,
}

impl CommandError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            config_id: None,
        }
    }
}

impl From<AppError> for CommandError {
    fn from(error: AppError) -> Self {
        match &error {
            AppError::ConfigRead(_) => Self::new("configRead", error.to_string()),
            AppError::ConfigWrite(_) => Self::new("configWrite", error.to_string()),
            AppError::Validation(_) => Self::new("validation", error.to_string()),
            AppError::Hotkey(_) => Self::new("hotkey", error.to_string()),
            AppError::Action(_) => Self::new("action", error.to_string()),
            AppError::Window(_) => Self::new("window", error.to_string()),
            AppError::Tray(_) => Self::new("tray", error.to_string()),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
pub type CommandResult<T> = Result<T, CommandError>;
