//! Native and legacy v3 report inspection, generation, and open IPC.

use std::path::Path;
use std::sync::Mutex;

use tauri::State;

use crate::commands::dialogs::DialogHandle;
use crate::coordinator::AppCoordinator;
use crate::dto::{AppStateDto, FileJobState};
use crate::errors::SafeError;
use crate::reports::{ReportsHandle, inspect_input, write_inspected_report};

#[tauri::command]
pub async fn choose_report_input(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    dialogs: State<'_, DialogHandle>,
    pick_file: Option<bool>,
) -> Result<AppStateDto, SafeError> {
    let current = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.begin_file_job(FileJobState::Inspecting)?;
        coordinator.output_root().map(Path::to_path_buf)
    };
    let handle = (*dialogs).clone();
    let pick_file = pick_file.unwrap_or(false);
    let picked = tauri::async_runtime::spawn_blocking(move || {
        if pick_file {
            handle.pick_file("Choose BIN or CSV", current.as_deref())
        } else {
            handle.pick_folder("Choose session folder", current.as_deref())
        }
    })
    .await;
    let picked = match picked {
        Ok(path) => path,
        Err(_) => {
            finish_job(&coordinator);
            return Err(SafeError::unexpected_failure());
        }
    };
    let Some(path) = picked else {
        finish_job(&coordinator);
        let coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        return Ok(coordinator.snapshot());
    };
    let live = {
        let coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator
            .live_recording_directory()
            .map(Path::to_path_buf)
    };
    let inspected =
        match tauri::async_runtime::spawn_blocking(move || inspect_input(&path, live.as_deref()))
            .await
        {
            Ok(result) => result,
            Err(_) => {
                finish_job(&coordinator);
                return Err(SafeError::unexpected_failure());
            }
        };
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _ = coordinator.finish_file_job();
    match inspected {
        Ok(inspected) => {
            coordinator.set_inspected_report(
                inspected.preview,
                inspected.directory,
                inspected.dest,
                inspected.input,
                inspected.kind,
            );
            Ok(coordinator.snapshot())
        }
        Err(error) => {
            coordinator.record_diagnostic(error.code, error.message());
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn generate_report(
    coordinator: State<'_, Mutex<AppCoordinator>>,
) -> Result<AppStateDto, SafeError> {
    run_generate(&coordinator, false).await
}

#[tauri::command]
pub async fn replace_report(
    coordinator: State<'_, Mutex<AppCoordinator>>,
) -> Result<AppStateDto, SafeError> {
    run_generate(&coordinator, true).await
}

#[tauri::command]
pub fn open_report(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    reports: State<'_, ReportsHandle>,
) -> Result<AppStateDto, SafeError> {
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reports.open_known_report(&coordinator)?;
    Ok(coordinator.snapshot())
}

#[tauri::command]
pub fn open_report_folder(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    reports: State<'_, ReportsHandle>,
) -> Result<AppStateDto, SafeError> {
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reports.open_known_folder(&coordinator)?;
    Ok(coordinator.snapshot())
}

async fn run_generate(
    coordinator: &Mutex<AppCoordinator>,
    replace: bool,
) -> Result<AppStateDto, SafeError> {
    let (input, kind) = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.begin_file_job(FileJobState::GeneratingReport)?;
        match (
            coordinator.report_input().map(Path::to_path_buf),
            coordinator.report_kind(),
        ) {
            (Some(input), Some(kind)) => (input, kind),
            _ => {
                let _ = coordinator.finish_file_job();
                return Err(SafeError::invalid_configuration(
                    "Inspect a session or file before generating a report.",
                ));
            }
        }
    };
    let written =
        tauri::async_runtime::spawn_blocking(move || write_inspected_report(&input, kind, replace))
            .await
            .map_err(|_| SafeError::unexpected_failure());
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _ = coordinator.finish_file_job();
    match written {
        Ok(Ok(_)) => {
            coordinator.mark_report_written();
            Ok(coordinator.snapshot())
        }
        Ok(Err(error)) if error.code == crate::dto::ErrorCode::OutputExists => {
            coordinator.mark_report_conflict();
            Err(error)
        }
        Ok(Err(error)) => {
            coordinator.record_diagnostic(error.code, error.message());
            Err(error)
        }
        Err(error) => Err(error),
    }
}

fn finish_job(coordinator: &Mutex<AppCoordinator>) {
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _ = coordinator.finish_file_job();
}
