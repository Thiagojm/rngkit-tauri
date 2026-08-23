//! Session-draft and theme IPC. Never persist candidate tokens.

use std::sync::Mutex;

use tauri::State;

use crate::coordinator::AppCoordinator;
use crate::dto::{AppStateDto, ThemePreference};
use crate::errors::SafeError;
use crate::preferences::PreferencesHandle;

use super::update_and_persist_session_draft;

#[tauri::command]
pub fn set_sample_bits(
    bits: u32,
    coordinator: State<'_, Mutex<AppCoordinator>>,
    prefs: State<'_, PreferencesHandle>,
) -> Result<AppStateDto, SafeError> {
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    update_and_persist_session_draft(&prefs, &mut coordinator, |coordinator| {
        coordinator.set_sample_bits(bits)
    })
}

#[tauri::command]
pub fn set_interval_seconds(
    seconds: u32,
    coordinator: State<'_, Mutex<AppCoordinator>>,
    prefs: State<'_, PreferencesHandle>,
) -> Result<AppStateDto, SafeError> {
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    update_and_persist_session_draft(&prefs, &mut coordinator, |coordinator| {
        coordinator.set_interval_seconds(seconds)
    })
}

#[tauri::command]
pub fn set_fold(
    fold: u32,
    coordinator: State<'_, Mutex<AppCoordinator>>,
    prefs: State<'_, PreferencesHandle>,
) -> Result<AppStateDto, SafeError> {
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    update_and_persist_session_draft(&prefs, &mut coordinator, |coordinator| {
        coordinator.set_fold(Some(fold))
    })
}

#[tauri::command]
pub fn set_theme(
    theme: String,
    coordinator: State<'_, Mutex<AppCoordinator>>,
    prefs: State<'_, PreferencesHandle>,
) -> Result<AppStateDto, SafeError> {
    let parsed = ThemePreference::parse(&theme)
        .ok_or_else(|| SafeError::invalid_configuration("Theme must be system, light, or dark."))?;
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    update_and_persist_session_draft(&prefs, &mut coordinator, |coordinator| {
        coordinator.set_theme(parsed);
        Ok(())
    })
}
