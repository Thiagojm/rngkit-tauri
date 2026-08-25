//! Native, legacy v3, and derived report inspection, generation, and opening.

mod inspect;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rngkit_recording::{
    ConcatenationStem, NativeSession, open_concatenation, open_flat_legacy_concatenation,
    open_legacy, open_standalone,
};
use rngkit_xlsx::{
    Overwrite, ReportOptions, XlsxError, derived_report_path, legacy_report_path,
    native_report_path, write_report_with_options,
};

use crate::coordinator::{AppCoordinator, ReportKind};
use crate::dto::{AppStateDto, CollectionState, FileJobState};
use crate::errors::SafeError;

pub use inspect::{
    InspectedReport, inspect_derived, inspect_input, inspect_legacy, inspect_native,
    inspect_standalone,
};

use inspect::{
    map_derived, map_derived_xlsx, map_legacy, map_legacy_xlsx, map_standalone, map_xlsx,
};

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

    pub fn open_existing_folder(&self, path: &Path) -> Result<(), SafeError> {
        if !path.is_dir() {
            return Err(SafeError::invalid_transition(
                "The derived folder is no longer available.",
            ));
        }
        self.opener.open_folder(path)?;
        self.record(path);
        Ok(())
    }
}

pub(crate) fn ensure_artifact_open_allowed(coordinator: &AppCoordinator) -> Result<(), SafeError> {
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
                inspected.options,
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
    let (Some(input), Some(kind), Some(options)) = (
        coordinator.report_input().map(Path::to_path_buf),
        coordinator.report_kind(),
        coordinator.report_options().cloned(),
    ) else {
        let _ = coordinator.finish_file_job();
        return Err(SafeError::invalid_configuration(
            "Inspect a session, file, or derived bundle before generating a report.",
        ));
    };
    let result = write_inspected_report(&input, kind, replace, &options);
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
    options: &ReportOptions,
) -> Result<PathBuf, SafeError> {
    match kind {
        ReportKind::Native => write_native_report_with_options(input, replace, options),
        ReportKind::Legacy => write_legacy_report_with_options(input, replace, options),
        ReportKind::Derived => write_derived_report_with_options(input, replace, options),
        ReportKind::Standalone => write_standalone_report_with_options(input, replace, options),
        ReportKind::FlatLegacyConcatenation => {
            write_flat_legacy_report_with_options(input, replace, options)
        }
    }
}

pub fn write_native_report(dir: &Path, replace: bool) -> Result<PathBuf, SafeError> {
    let native = NativeSession::open(dir).map_err(inspect::map_recording)?;
    let options = ReportOptions::for_session(&native.normalized()).map_err(map_xlsx)?;
    write_native_report_with_options(dir, replace, &options)
}

fn write_native_report_with_options(
    dir: &Path,
    replace: bool,
    options: &ReportOptions,
) -> Result<PathBuf, SafeError> {
    let native = NativeSession::open(dir).map_err(inspect::map_recording)?;
    let session = native.normalized();
    ensure_report_options(&session, options)?;
    let dest = native_report_path(native.directory(), native.session_stem()).map_err(map_xlsx)?;
    write_report_with_options(&session, &dest, overwrite_mode(replace), options).map_err(map_xlsx)
}

pub fn write_legacy_report(selected: &Path, replace: bool) -> Result<PathBuf, SafeError> {
    let session = open_legacy(selected).map_err(map_legacy)?;
    let options = ReportOptions::for_session(&session).map_err(map_legacy_xlsx)?;
    write_legacy_report_with_options(selected, replace, &options)
}

fn write_legacy_report_with_options(
    selected: &Path,
    replace: bool,
    options: &ReportOptions,
) -> Result<PathBuf, SafeError> {
    let session = open_legacy(selected).map_err(map_legacy)?;
    ensure_report_options(&session, options)?;
    let dest = legacy_report_path(selected);
    write_report_with_options(&session, &dest, overwrite_mode(replace), options)
        .map_err(map_legacy_xlsx)
}

pub fn write_standalone_report(selected: &Path, replace: bool) -> Result<PathBuf, SafeError> {
    let session = open_standalone(selected).map_err(map_standalone)?;
    let options = ReportOptions::for_session(&session).map_err(map_standalone_xlsx)?;
    write_standalone_report_with_options(selected, replace, &options)
}

fn write_standalone_report_with_options(
    selected: &Path,
    replace: bool,
    options: &ReportOptions,
) -> Result<PathBuf, SafeError> {
    let session = open_standalone(selected).map_err(map_standalone)?;
    ensure_report_options(&session, options)?;
    let dest = legacy_report_path(selected);
    write_report_with_options(&session, &dest, overwrite_mode(replace), options)
        .map_err(map_standalone_xlsx)
}

pub fn write_derived_report(dir: &Path, replace: bool) -> Result<PathBuf, SafeError> {
    let session = open_concatenation(dir).map_err(map_derived)?;
    let options = ReportOptions::for_session(&session).map_err(map_derived_xlsx)?;
    write_derived_report_with_options(dir, replace, &options)
}

fn write_derived_report_with_options(
    dir: &Path,
    replace: bool,
    options: &ReportOptions,
) -> Result<PathBuf, SafeError> {
    let session = open_concatenation(dir).map_err(map_derived)?;
    ensure_report_options(&session, options)?;
    let stem = ConcatenationStem::parse(&session.meta().stem).map_err(map_derived)?;
    let dest = derived_report_path(dir, &stem).map_err(map_xlsx)?;
    write_report_with_options(&session, &dest, overwrite_mode(replace), options)
        .map_err(map_derived_xlsx)
}

fn write_flat_legacy_report_with_options(
    selected: &Path,
    replace: bool,
    options: &ReportOptions,
) -> Result<PathBuf, SafeError> {
    let session = open_flat_legacy_concatenation(selected).map_err(map_standalone)?;
    ensure_report_options(&session, options)?;
    let dest = legacy_report_path(selected);
    write_report_with_options(&session, &dest, overwrite_mode(replace), options)
        .map_err(map_standalone_xlsx)
}

fn ensure_report_options(
    session: &rngkit_recording::NormalizedSession,
    expected: &ReportOptions,
) -> Result<(), SafeError> {
    let actual = ReportOptions::for_session(session).map_err(map_xlsx)?;
    if actual == *expected {
        Ok(())
    } else {
        Err(SafeError::corrupt_input(
            "The selected report input changed; inspect it again.",
        ))
    }
}

fn map_standalone_xlsx(error: XlsxError) -> SafeError {
    match error {
        XlsxError::Recording(error) => map_standalone(error),
        other => map_xlsx(other),
    }
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
        spawn_open(path)
    }

    fn open_file(&self, path: &Path) -> Result<(), SafeError> {
        spawn_open(path)
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

pub(crate) fn spawn_open(path: &Path) -> Result<(), SafeError> {
    let mut command = if cfg!(windows) {
        let mut command = std::process::Command::new("explorer");
        command.arg(path);
        command
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
