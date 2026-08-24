//! Native, legacy v3, and derived concatenation inspection.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use rngkit_core::{SessionStatus, TimestampProvenance};
use rngkit_recording::{
    ConcatenationStem, NativeSession, RecordingError, open_concatenation, open_legacy,
};
use rngkit_xlsx::{XlsxError, derived_report_path, legacy_report_path, native_report_path};

use crate::coordinator::ReportKind;
use crate::dto::ReportPreview;
use crate::errors::SafeError;

const KIND_LABEL: &str = "Native session";
const ORIGIN: &str = "Collected session";
const LEGACY_KIND: &str = "Legacy v3";
const DERIVED_KIND: &str = "Derived bundle";
const DERIVED_ORIGIN: &str = "Concatenated legacy v3 CSV";
const DERIVED_NOTE: &str = "Timestamps are copied from the concatenated inputs.";
const ESTIMATED_WARNING: &str = "Timestamps are estimated from the filename start and interval.";
const RECORDED_NOTE: &str = "Timestamps are recorded in the CSV input.";
const TAIL_WARNING: &str =
    "The session has an uncommitted binary tail. Report rows use the committed CSV prefix.";

#[derive(Debug)]
pub struct InspectedReport {
    pub preview: ReportPreview,
    pub directory: PathBuf,
    pub dest: PathBuf,
    pub input: PathBuf,
    pub kind: ReportKind,
}

pub fn inspect_input(
    path: &Path,
    live_recording: Option<&Path>,
) -> Result<InspectedReport, SafeError> {
    if path.is_dir() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if ConcatenationStem::parse(name).is_ok() {
            inspect_derived(path)
        } else {
            inspect_native(path, live_recording)
        }
    } else {
        inspect_legacy(path)
    }
}

pub fn inspect_native(
    dir: &Path,
    live_recording: Option<&Path>,
) -> Result<InspectedReport, SafeError> {
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
    let directory = native.directory().to_path_buf();
    Ok(InspectedReport {
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
        dest,
        input: directory.clone(),
        directory,
        kind: ReportKind::Native,
    })
}

pub fn inspect_legacy(path: &Path) -> Result<InspectedReport, SafeError> {
    let session = open_legacy(path).map_err(map_legacy)?;
    let dest = legacy_report_path(path);
    let meta = session.meta();
    let warning = match meta.provenance {
        TimestampProvenance::Estimated => Some(ESTIMATED_WARNING.to_owned()),
        TimestampProvenance::Recorded => Some(RECORDED_NOTE.to_owned()),
    };
    let directory = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    Ok(InspectedReport {
        preview: ReportPreview {
            kind_label: LEGACY_KIND.into(),
            origin: legacy_origin(path).into(),
            source: meta.source_label.clone(),
            sample_bits: meta.sample_bits.get(),
            interval_seconds: meta.interval.get(),
            fold: meta.fold.map(|fold| u32::from(fold.get())),
            status: status_label(meta.status).into(),
            row_count: session.records().len() as u64,
            warning,
            conflict: dest.exists(),
        },
        dest,
        input: path.to_path_buf(),
        directory,
        kind: ReportKind::Legacy,
    })
}

pub fn inspect_derived(path: &Path) -> Result<InspectedReport, SafeError> {
    let session = open_concatenation(path).map_err(map_derived)?;
    let stem = ConcatenationStem::parse(&session.meta().stem).map_err(map_derived)?;
    let dest = derived_report_path(path, &stem).map_err(map_xlsx)?;
    let directory = path.to_path_buf();
    Ok(InspectedReport {
        preview: ReportPreview {
            kind_label: DERIVED_KIND.into(),
            origin: DERIVED_ORIGIN.into(),
            source: session.meta().source_label.clone(),
            sample_bits: session.meta().sample_bits.get(),
            interval_seconds: session.meta().interval.get(),
            fold: session.meta().fold.map(|fold| u32::from(fold.get())),
            status: status_label(session.meta().status).into(),
            row_count: session.records().len() as u64,
            warning: Some(DERIVED_NOTE.into()),
            conflict: dest.exists(),
        },
        dest,
        input: directory.clone(),
        directory,
        kind: ReportKind::Derived,
    })
}

fn legacy_origin(path: &Path) -> &'static str {
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return LEGACY_KIND;
    };
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    match (
        parent.join(format!("{stem}.csv")).is_file(),
        parent.join(format!("{stem}.bin")).is_file(),
    ) {
        (true, true) => "Paired BIN and CSV",
        (true, false) => "CSV only",
        (false, true) => "BIN only",
        (false, false) => LEGACY_KIND,
    }
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

pub(super) fn map_legacy(error: RecordingError) -> SafeError {
    match error {
        RecordingError::OnesExceedSampleBits { .. } => {
            SafeError::invalid_configuration("A one-count exceeds the sample size.")
        }
        RecordingError::Corrupt { .. } | RecordingError::Csv(_) => {
            SafeError::corrupt_input("The selected file is corrupt.")
        }
        RecordingError::Io(io) if io.kind() == ErrorKind::PermissionDenied => {
            SafeError::permission_denied("The selected file could not be read.")
        }
        RecordingError::UnsupportedVersion { .. }
        | RecordingError::InvalidName { .. }
        | RecordingError::Io(_)
        | RecordingError::Core(_) => {
            SafeError::unsupported_input("That file is not a supported RngKitPSG v3 input.")
        }
        _ => SafeError::unsupported_input("That file is not a supported RngKitPSG v3 input."),
    }
}

pub(super) fn map_derived(error: RecordingError) -> SafeError {
    match error {
        RecordingError::OnesExceedSampleBits { .. } => {
            SafeError::invalid_configuration("A one-count exceeds the sample size.")
        }
        RecordingError::Corrupt { .. }
        | RecordingError::Json(_)
        | RecordingError::Csv(_)
        | RecordingError::InconsistentConcatenationRange
        | RecordingError::InvalidConcatenationHash => {
            SafeError::corrupt_input("The selected derived bundle is corrupt.")
        }
        RecordingError::Io(io) if io.kind() == ErrorKind::PermissionDenied => {
            SafeError::permission_denied("The selected derived bundle could not be read.")
        }
        RecordingError::UnsupportedConcatenationKind { .. }
        | RecordingError::UnsupportedSchema { .. }
        | RecordingError::UnsupportedVersion { .. }
        | RecordingError::InvalidName { .. }
        | RecordingError::Io(_) => {
            SafeError::unsupported_input("That folder is not a supported derived bundle.")
        }
        _ => SafeError::unsupported_input("That folder is not a supported derived bundle."),
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

pub(super) fn map_legacy_xlsx(error: XlsxError) -> SafeError {
    match error {
        XlsxError::Recording(error) => map_legacy(error),
        other => map_xlsx(other),
    }
}

pub(super) fn map_derived_xlsx(error: XlsxError) -> SafeError {
    match error {
        XlsxError::Recording(error) => map_derived(error),
        other => map_xlsx(other),
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
