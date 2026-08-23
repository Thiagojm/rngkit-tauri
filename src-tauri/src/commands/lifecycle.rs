//! Stop-and-exit and sanitized copy-diagnostics IPC.

use std::sync::Mutex;

use tauri::{AppHandle, Runtime, State};

use crate::coordinator::AppCoordinator;
use crate::diagnostics::format_copy;
use crate::dto::AppStateDto;
use crate::errors::SafeError;
use crate::lifecycle::begin_exit_after_stop;

#[tauri::command]
pub fn copy_diagnostics(
    coordinator: State<'_, Mutex<AppCoordinator>>,
) -> Result<String, SafeError> {
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(format_copy(&coordinator.diagnostics()))
}

#[tauri::command]
pub fn stop_and_exit<R: Runtime>(
    app: AppHandle<R>,
    coordinator: State<'_, Mutex<AppCoordinator>>,
) -> Result<AppStateDto, SafeError> {
    begin_exit_after_stop(&app);
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(coordinator.snapshot())
}
