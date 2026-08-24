//! Native session inspection. Legacy and derived inputs stay unsupported here.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use rngkit_core::SessionStatus;
use rngkit_recording::{NativeSession, RecordingError};
use rngkit_xlsx::{XlsxError, native_report_path};

use crate::dto::ReportPreview;
use crate::errors::SafeError;

const KIND_LABEL: &str = "Native session";
const ORIGIN: &str = "Collected session";
const TAIL_WARNING: &str =
    "The session has an uncommitted binary tail. Report rows use the committed CSV prefix.";

#[derive(Debug)]
pub struct InspectedNative {
    pub preview: ReportPreview,
    pub directory: PathBuf,
    pub dest: PathBuf,
}

pub fn inspect_native(
    dir: &Path,
    live_recording: Option<&Path>,
) -> Result<InspectedNative, SafeError> {
    if is_same_dir(dir, live_recording) {
        return Err(SafeError::operation_conflict(
            "The selected session is currently recording.",
        ));
    }
    let native = NativeSession::open(dir).map_err(map_recording)?;
    let dest = native_report_path(native.directory(), native.session_stem()).map_err(map_xlsx)?;
    let conflict = dest.exists();
    let warning = if native.consistency().warnings.is_empty() {
        None
    } else {
        Some(TAIL_WARNING.to_owned())
    };
    Ok(InspectedNative {
        preview: ReportPreview {
            kind_label: KIND_LABEL.into(),
            origin: ORIGIN.into(),
            source: native.manifest().source_label().to_owned(),
            sample_bits: native.manifest().sample_bits().get(),
            interval_seconds: native.manifest().interval().get(),
            fold: native.manifest().fold().map(|fold| u32::from(fold.get())),
            status: status_label(native.manifest().reader_status()).into(),
            row_count: native.records().len() as u64,
            warning,
            conflict,
        },
        directory: native.directory().to_path_buf(),
        dest,
    })
}

pub(super) fn map_recording(error: RecordingError) -> SafeError {
    match error {
        RecordingError::Corrupt { .. } | RecordingError::Json(_) | RecordingError::Csv(_) => {
            SafeError::corrupt_input("The selected session is corrupt.")
        }
        RecordingError::Io(io) if io.kind() == ErrorKind::PermissionDenied => {
            SafeError::permission_denied("The selected session could not be read.")
        }
        RecordingError::UnsupportedVersion { .. }
        | RecordingError::UnsupportedSchema { .. }
        | RecordingError::InvalidName { .. }
        | RecordingError::Io(_) => {
            SafeError::unsupported_input("That input is not a supported native session.")
        }
        _ => SafeError::unsupported_input("That input is not a supported native session."),
    }
}

pub(super) fn map_xlsx(error: XlsxError) -> SafeError {
    match error {
        XlsxError::AlreadyExists { .. } => SafeError::report_exists(),
        XlsxError::RowLimit { .. } => {
            SafeError::invalid_configuration("The session has too many rows for an Excel report.")
        }
        XlsxError::Recording(error) => map_recording(error),
        XlsxError::Io(io) if io.kind() == ErrorKind::PermissionDenied => {
            SafeError::permission_denied("The report could not be written.")
        }
        _ => SafeError::unexpected_failure(),
    }
}

fn status_label(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Completed => "Completed",
        SessionStatus::Failed => "Failed",
        SessionStatus::Interrupted | SessionStatus::Recording => "Interrupted",
    }
}

fn is_same_dir(dir: &Path, live: Option<&Path>) -> bool {
    let Some(live) = live else {
        return false;
    };
    let left = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let right = fs::canonicalize(live).unwrap_or_else(|_| live.to_path_buf());
    left == right
}
