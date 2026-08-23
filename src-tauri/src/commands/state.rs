//! Production IPC commands for coordinator snapshots.

use std::sync::Mutex;

use tauri::State;

use crate::coordinator::AppCoordinator;
use crate::dto::AppStateDto;
use crate::errors::SafeError;

#[tauri::command]
pub fn get_app_state(
    coordinator: State<'_, Mutex<AppCoordinator>>,
) -> Result<AppStateDto, SafeError> {
    Ok(coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .snapshot())
}
