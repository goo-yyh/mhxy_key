use std::{
    collections::HashSet,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Wry};

use crate::{
    input_simulator::EnigoInputSimulator,
    models::{Action, MouseButton},
};

#[derive(Debug, Clone)]
pub struct ActionExecutor {
    tx: Sender<ExecutorMessage>,
    state: Arc<Mutex<ExecutorState>>,
}

#[derive(Debug, Default)]
struct ExecutorState {
    running: HashSet<String>,
    cancelled: HashSet<String>,
    generation: u64,
    shutdown: bool,
}

#[derive(Debug)]
enum ExecutorMessage {
    Run(ExecutionRequest),
    CancelConfig(String),
    CancelAll,
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    pub config_id: String,
    pub actions: Vec<Action>,
    pub generation: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionEvent {
    config_id: String,
    message: Option<String>,
}

impl ActionExecutor {
    pub fn new(app: AppHandle<Wry>) -> Self {
        let (tx, rx) = mpsc::channel();
        let state = Arc::new(Mutex::new(ExecutorState::default()));
        let worker_state = state.clone();

        thread::spawn(move || {
            let mut simulator = EnigoInputSimulator::new().ok();

            while let Ok(message) = rx.recv() {
                match message {
                    ExecutorMessage::Run(request) => {
                        let config_id = request.config_id.clone();
                        let _ = app.emit(
                            "action://started",
                            ActionEvent {
                                config_id: config_id.clone(),
                                message: None,
                            },
                        );

                        let result =
                            execute_request(&mut simulator, &worker_state, request.clone());

                        {
                            let mut state = worker_state.lock().expect("executor state poisoned");
                            state.running.remove(&config_id);
                            state.cancelled.remove(&config_id);
                        }

                        match result {
                            Ok(()) => {
                                let _ = app.emit(
                                    "action://finished",
                                    ActionEvent {
                                        config_id,
                                        message: None,
                                    },
                                );
                            }
                            Err(message) => {
                                if let Some(simulator) = simulator.as_mut() {
                                    simulator.release_all();
                                }
                                let _ = app.emit(
                                    "action://failed",
                                    ActionEvent {
                                        config_id,
                                        message: Some(message),
                                    },
                                );
                            }
                        }
                    }
                    ExecutorMessage::CancelConfig(config_id) => {
                        let mut state = worker_state.lock().expect("executor state poisoned");
                        state.cancelled.insert(config_id);
                    }
                    ExecutorMessage::CancelAll => {
                        let mut state = worker_state.lock().expect("executor state poisoned");
                        state.generation = state.generation.saturating_add(1);
                        let running = state.running.clone();
                        state.cancelled.extend(running);
                    }
                    ExecutorMessage::Shutdown => {
                        let mut state = worker_state.lock().expect("executor state poisoned");
                        state.shutdown = true;
                        state.generation = state.generation.saturating_add(1);
                        let running = state.running.clone();
                        state.cancelled.extend(running);
                        if let Some(simulator) = simulator.as_mut() {
                            simulator.release_all();
                        }
                        break;
                    }
                }
            }
        });

        Self { tx, state }
    }

    pub fn enqueue(&self, config_id: String, actions: Vec<Action>) -> bool {
        let generation = {
            let mut state = self.state.lock().expect("executor state poisoned");
            if state.shutdown || state.running.contains(&config_id) {
                return false;
            }
            state.cancelled.remove(&config_id);
            state.running.insert(config_id.clone());
            state.generation
        };

        self.tx
            .send(ExecutorMessage::Run(ExecutionRequest {
                config_id,
                actions,
                generation,
            }))
            .is_ok()
    }

    pub fn cancel_config(&self, config_id: &str) {
        let _ = self
            .tx
            .send(ExecutorMessage::CancelConfig(config_id.to_string()));
    }

    pub fn cancel_all(&self) {
        let _ = self.tx.send(ExecutorMessage::CancelAll);
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(ExecutorMessage::Shutdown);
    }

    pub fn next_generation(&self) {
        let mut state = self.state.lock().expect("executor state poisoned");
        state.generation = state.generation.saturating_add(1);
    }
}

fn execute_request(
    simulator: &mut Option<EnigoInputSimulator>,
    state: &Arc<Mutex<ExecutorState>>,
    request: ExecutionRequest,
) -> Result<(), String> {
    let simulator = simulator
        .as_mut()
        .ok_or_else(|| "输入模拟器初始化失败，请检查系统权限".to_string())?;

    for action in &request.actions {
        ensure_not_cancelled(state, &request)?;

        match action {
            Action::KeyCombo {
                keys,
                delay_after_ms,
            } => {
                simulator.key_combo(keys).map_err(|err| err.to_string())?;
                interruptible_sleep(state, &request, *delay_after_ms)?;
            }
            Action::MouseClick {
                button,
                click_count,
                delay_after_ms,
            } => {
                execute_mouse_click(simulator, *button, *click_count)?;
                interruptible_sleep(state, &request, *delay_after_ms)?;
            }
            Action::Delay { duration_ms } => {
                interruptible_sleep(state, &request, *duration_ms)?;
            }
        }
    }

    thread::sleep(Duration::from_millis(50));
    Ok(())
}

fn execute_mouse_click(
    simulator: &mut EnigoInputSimulator,
    button: MouseButton,
    click_count: u8,
) -> Result<(), String> {
    simulator
        .mouse_click(button, click_count)
        .map_err(|err| err.to_string())
}

fn ensure_not_cancelled(
    state: &Arc<Mutex<ExecutorState>>,
    request: &ExecutionRequest,
) -> Result<(), String> {
    let state = state.lock().expect("executor state poisoned");
    if state.shutdown
        || state.generation != request.generation
        || state.cancelled.contains(&request.config_id)
    {
        return Err("动作已取消".to_string());
    }
    Ok(())
}

fn interruptible_sleep(
    state: &Arc<Mutex<ExecutorState>>,
    request: &ExecutionRequest,
    duration_ms: u64,
) -> Result<(), String> {
    let mut remaining = duration_ms;
    while remaining > 0 {
        ensure_not_cancelled(state, request)?;
        let chunk = remaining.min(20);
        thread::sleep(Duration::from_millis(chunk));
        remaining -= chunk;
    }
    Ok(())
}
