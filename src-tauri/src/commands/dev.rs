//! Debug-only transition command. Omitted from release builds.

use std::sync::Mutex;

use tauri::State;

use crate::coordinator::{AppCoordinator, DevScenario};
use crate::dto::AppStateDto;
use crate::errors::SafeError;

#[tauri::command]
pub fn apply_dev_scenario(
    scenario_id: String,
    coordinator: State<'_, Mutex<AppCoordinator>>,
) -> Result<AppStateDto, SafeError> {
    let scenario = DevScenario::parse(&scenario_id)?;
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    coordinator.load_dev_fixture(scenario);
    Ok(coordinator.snapshot())
}
