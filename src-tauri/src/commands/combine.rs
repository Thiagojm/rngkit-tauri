//! Combine preview, derived creation, and derived XLSX IPC.

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use tauri::State;

use crate::combine::{apply_preview, finish_created, map_combine};
use crate::commands::dialogs::DialogHandle;
use crate::coordinator::AppCoordinator;
use crate::dto::{AppStateDto, FileJobState};
use crate::errors::SafeError;
use crate::reports::{ReportsHandle, ensure_artifact_open_allowed};

#[tauri::command]
pub async fn choose_csv_inputs(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    dialogs: State<'_, DialogHandle>,
) -> Result<AppStateDto, SafeError> {
    let current = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.begin_file_job(FileJobState::Inspecting)?;
        coordinator
            .combine_last_directory()
            .or_else(|| coordinator.output_root())
            .map(Path::to_path_buf)
    };
    let handle = (*dialogs).clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        handle.pick_files("Add CSV files", current.as_deref())
    })
    .await;
    let picked = match picked {
        Ok(paths) => paths,
        Err(_) => {
            finish_job(&coordinator);
            return Err(SafeError::unexpected_failure());
        }
    };
    let Some(paths) = picked else {
        finish_job(&coordinator);
        let coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        return Ok(coordinator.snapshot());
    };
    let canonical = paths
        .iter()
        .map(|path| {
            fs::canonicalize(path)
                .map_err(|_| SafeError::corrupt_input("A selected CSV file could not be read."))
        })
        .collect::<Result<Vec<_>, _>>();
    let canonical = match canonical {
        Ok(paths) => paths,
        Err(error) => {
            finish_job(&coordinator);
            return Err(error);
        }
    };
    let selected = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(directory) = paths
            .first()
            .and_then(|path| path.parent())
            .map(Path::to_path_buf)
        {
            coordinator.remember_combine_directory(directory);
        }
        coordinator.add_combine_inputs(canonical);
        coordinator.combine_inputs().to_vec()
    };
    let inspected_paths = selected.clone();
    let inspected = tauri::async_runtime::spawn_blocking(move || {
        rngkit_recording::inspect_csv_inputs(&selected)
    })
    .await;
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _ = coordinator.finish_file_job();
    match inspected {
        Ok(result) => apply_preview(&mut coordinator, &inspected_paths, result),
        Err(_) => Err(SafeError::unexpected_failure()),
    }
}

#[tauri::command]
pub async fn remove_combine_input(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    input_id: String,
) -> Result<AppStateDto, SafeError> {
    let selected = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.begin_file_job(FileJobState::Inspecting)?;
        if let Err(error) = coordinator.remove_combine_input(&input_id) {
            let _ = coordinator.finish_file_job();
            return Err(error);
        }
        coordinator.combine_inputs().to_vec()
    };
    if selected.is_empty() {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _ = coordinator.finish_file_job();
        coordinator.clear_combine_inputs();
        return Ok(coordinator.snapshot());
    }
    let inspected = tauri::async_runtime::spawn_blocking(move || {
        rngkit_recording::inspect_csv_inputs(&selected)
    })
    .await;
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _ = coordinator.finish_file_job();
    match inspected {
        Ok(result) => {
            let paths = coordinator.combine_inputs().to_vec();
            apply_preview(&mut coordinator, &paths, result)
        }
        Err(_) => Err(SafeError::unexpected_failure()),
    }
}

#[tauri::command]
pub fn clear_combine_inputs(
    coordinator: State<'_, Mutex<AppCoordinator>>,
) -> Result<AppStateDto, SafeError> {
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    coordinator.begin_file_job(FileJobState::Inspecting)?;
    coordinator.clear_combine_inputs();
    let _ = coordinator.finish_file_job();
    Ok(coordinator.snapshot())
}

#[tauri::command]
pub async fn create_derived(
    coordinator: State<'_, Mutex<AppCoordinator>>,
) -> Result<AppStateDto, SafeError> {
    let (paths, root) = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.begin_file_job(FileJobState::Combining)?;
        (
            coordinator.combine_inputs().to_vec(),
            coordinator.output_root().map(Path::to_path_buf),
        )
    };
    let Some(root) = root else {
        finish_job(&coordinator);
        return Err(SafeError::invalid_configuration(
            "Choose an output folder before creating a derived bundle.",
        ));
    };
    if paths.is_empty() {
        finish_job(&coordinator);
        return Err(SafeError::invalid_configuration(
            "Select compatible CSV files before creating a bundle.",
        ));
    }
    let created = tauri::async_runtime::spawn_blocking(move || {
        rngkit_recording::create_csv_concatenation(&paths, &root)
    })
    .await;
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _ = coordinator.finish_file_job();
    match created {
        Ok(Ok(directory)) => finish_created(&mut coordinator, directory),
        Ok(Err(error)) => {
            let mapped = map_combine(error);
            coordinator.record_diagnostic(mapped.code, mapped.message());
            Err(mapped)
        }
        Err(_) => Err(SafeError::unexpected_failure()),
    }
}

#[tauri::command]
pub async fn generate_derived(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    replace: Option<bool>,
) -> Result<AppStateDto, SafeError> {
    let directory = {
        let mut coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        coordinator.begin_file_job(FileJobState::GeneratingReport)?;
        match coordinator.combine_directory().map(Path::to_path_buf) {
            Some(directory) => directory,
            None => {
                let _ = coordinator.finish_file_job();
                return Err(SafeError::invalid_configuration(
                    "Create a derived bundle before generating XLSX.",
                ));
            }
        }
    };
    let replace = replace.unwrap_or(false);
    let written = tauri::async_runtime::spawn_blocking(move || {
        crate::reports::write_inspected_report(
            &directory,
            crate::coordinator::ReportKind::Derived,
            replace,
        )
    })
    .await;
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
        Err(_) => Err(SafeError::unexpected_failure()),
    }
}

#[tauri::command]
pub fn open_derived_folder(
    coordinator: State<'_, Mutex<AppCoordinator>>,
    reports: State<'_, ReportsHandle>,
) -> Result<AppStateDto, SafeError> {
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    ensure_artifact_open_allowed(&coordinator)?;
    let path = coordinator.combine_directory().ok_or_else(|| {
        SafeError::invalid_transition("Create a derived bundle before opening its folder.")
    })?;
    reports.open_existing_folder(path)?;
    Ok(coordinator.snapshot())
}

fn finish_job(coordinator: &Mutex<AppCoordinator>) {
    let mut coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _ = coordinator.finish_file_job();
}
