//! Native and legacy v3 report inspection, generation, and backend-known artifact opening.

mod inspect;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rngkit_recording::{NativeSession, open_legacy};
use rngkit_xlsx::{Overwrite, legacy_report_path, native_report_path, write_report};

use crate::coordinator::{AppCoordinator, ReportKind};
use crate::dto::{AppStateDto, CollectionState, FileJobState};
use crate::errors::SafeError;

pub use inspect::{InspectedReport, inspect_input, inspect_legacy, inspect_native};

use inspect::{map_legacy, map_legacy_xlsx, map_xlsx};

pub trait ArtifactOpener: Send + Sync {
    fn open_folder(&self, path: &Path) -> Result<(), SafeError>;
    fn open_file(&self, path: &Path) -> Result<(), SafeError>;
}

#[derive(Clone)]
pub struct ReportsHandle {
    opener: Arc<dyn ArtifactOpener>,
    opened: Arc<Mutex<Vec<PathBuf>>>,
}

impl ReportsHandle {
    #[must_use]
    pub fn live() -> Self {
        Self {
            opener: Arc::new(LiveArtifactOpener),
            opened: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[must_use]
    pub fn fake() -> Self {
        Self {
            opener: Arc::new(RecordingArtifactOpener),
            opened: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn open_known_report(&self, coordinator: &AppCoordinator) -> Result<(), SafeError> {
        ensure_artifact_open_allowed(coordinator)?;
        let path = coordinator
            .report_dest()
            .filter(|_| coordinator.report_ready())
            .ok_or_else(|| {
                SafeError::invalid_transition("Open the report after a report is generated.")
            })?;
        if !path.is_file() {
            return Err(SafeError::invalid_transition(
                "The generated report is no longer available.",
            ));
        }
        self.opener.open_file(path)?;
        self.record(path);
        Ok(())
    }

    pub fn open_known_folder(&self, coordinator: &AppCoordinator) -> Result<(), SafeError> {
        ensure_artifact_open_allowed(coordinator)?;
        let path = coordinator
            .report_directory()
            .filter(|_| coordinator.report_ready())
            .ok_or_else(|| {
                SafeError::invalid_transition("Open the containing folder after a report exists.")
            })?;
        if !path.is_dir() {
            return Err(SafeError::invalid_transition(
                "The report folder is no longer available.",
            ));
        }
        self.opener.open_folder(path)?;
        self.record(path);
        Ok(())
    }

    #[must_use]
    pub fn opened(&self) -> Vec<PathBuf> {
        self.opened
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn record(&self, path: &Path) {
        self.opened
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(path.to_path_buf());
    }
}

fn ensure_artifact_open_allowed(coordinator: &AppCoordinator) -> Result<(), SafeError> {
    let snapshot = coordinator.snapshot();
    if matches!(
        snapshot.collection.state,
        CollectionState::Collecting | CollectionState::Stopping
    ) {
        return Err(SafeError::operation_conflict(
            "Report artifacts cannot open while a session is collecting or stopping.",
        ));
    }
    if snapshot.file_job != FileJobState::Idle {
        return Err(SafeError::operation_conflict(
            "Report artifacts cannot open while a file job is running.",
        ));
    }
    Ok(())
}

pub fn inspect_picked(
    coordinator: &mut AppCoordinator,
    dir: &Path,
) -> Result<AppStateDto, SafeError> {
    coordinator.begin_file_job(FileJobState::Inspecting)?;
    let live = coordinator
        .live_recording_directory()
        .map(Path::to_path_buf);
    let result = inspect_input(dir, live.as_deref());
    let _ = coordinator.finish_file_job();
    match result {
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

pub fn generate_inspected(
    coordinator: &mut AppCoordinator,
    replace: bool,
) -> Result<AppStateDto, SafeError> {
    coordinator.begin_file_job(FileJobState::GeneratingReport)?;
    let (Some(input), Some(kind)) = (
        coordinator.report_input().map(Path::to_path_buf),
        coordinator.report_kind(),
    ) else {
        let _ = coordinator.finish_file_job();
        return Err(SafeError::invalid_configuration(
            "Inspect a session or file before generating a report.",
        ));
    };
    let result = write_inspected_report(&input, kind, replace);
    let _ = coordinator.finish_file_job();
    match result {
        Ok(_) => {
            coordinator.mark_report_written();
            Ok(coordinator.snapshot())
        }
        Err(error) if error.code == crate::dto::ErrorCode::OutputExists => {
            coordinator.mark_report_conflict();
            Err(error)
        }
        Err(error) => {
            coordinator.record_diagnostic(error.code, error.message());
            Err(error)
        }
    }
}

pub fn write_inspected_report(
    input: &Path,
    kind: ReportKind,
    replace: bool,
) -> Result<PathBuf, SafeError> {
    match kind {
        ReportKind::Native => write_native_report(input, replace),
        ReportKind::Legacy => write_legacy_report(input, replace),
    }
}

pub fn write_native_report(dir: &Path, replace: bool) -> Result<PathBuf, SafeError> {
    let native = NativeSession::open(dir).map_err(inspect::map_recording)?;
    let dest = native_report_path(native.directory(), native.session_stem()).map_err(map_xlsx)?;
    write_report(&native.normalized(), &dest, overwrite_mode(replace)).map_err(map_xlsx)
}

pub fn write_legacy_report(selected: &Path, replace: bool) -> Result<PathBuf, SafeError> {
    let session = open_legacy(selected).map_err(map_legacy)?;
    let dest = legacy_report_path(selected);
    write_report(&session, &dest, overwrite_mode(replace)).map_err(map_legacy_xlsx)
}

fn overwrite_mode(replace: bool) -> Overwrite {
    if replace {
        Overwrite::Replace
    } else {
        Overwrite::ErrorIfExists
    }
}

struct LiveArtifactOpener;

impl ArtifactOpener for LiveArtifactOpener {
    fn open_folder(&self, path: &Path) -> Result<(), SafeError> {
        spawn_open(path, true)
    }

    fn open_file(&self, path: &Path) -> Result<(), SafeError> {
        spawn_open(path, false)
    }
}

struct RecordingArtifactOpener;

impl ArtifactOpener for RecordingArtifactOpener {
    fn open_folder(&self, _path: &Path) -> Result<(), SafeError> {
        Ok(())
    }

    fn open_file(&self, _path: &Path) -> Result<(), SafeError> {
        Ok(())
    }
}

fn spawn_open(path: &Path, folder: bool) -> Result<(), SafeError> {
    let mut command = if cfg!(windows) {
        if folder {
            let mut command = std::process::Command::new("explorer");
            command.arg(path);
            command
        } else {
            let mut command = std::process::Command::new("cmd");
            command.args(["/C", "start", "", &path.to_string_lossy()]);
            command
        }
    } else {
        let mut command = std::process::Command::new("xdg-open");
        command.arg(path);
        command
    };
    command
        .spawn()
        .map(|_| ())
        .map_err(|_| SafeError::unexpected_failure())
}
