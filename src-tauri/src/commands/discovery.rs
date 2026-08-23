//! Production discovery and selection IPC. Does not open a source.

use std::sync::Mutex;

use tauri::State;

use crate::coordinator::AppCoordinator;
use crate::discovery::DiscoveryHandle;
use crate::dto::AppStateDto;
use crate::errors::SafeError;

#[tauri::command]
pub async fn refresh_sources(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    discovery: State<'_, DiscoveryHandle>,
) -> Result<AppStateDto, SafeError> {
    let generation = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.begin_discover()?
    };

    let handle = (*discovery).clone();
    match tauri::async_runtime::spawn_blocking(move || handle.discover()).await {
        Ok(outcome) => {
            let mut coordinator = coordinator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            coordinator.apply_discovery(generation, outcome)?;
            Ok(coordinator.snapshot())
        }
        Err(_) => {
            let mut coordinator = coordinator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            coordinator.fail_discover(generation)?;
            Err(SafeError::unexpected_failure())
        }
    }
}

#[tauri::command]
pub fn select_source(
    token: String,
    coordinator: State<'_, Mutex<AppCoordinator>>,
) -> Result<AppStateDto, SafeError> {
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    coordinator.select_token(&token)?;
    Ok(coordinator.snapshot())
}
