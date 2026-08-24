//! Legacy v3 CSV concatenation preview and derived-bundle creation.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use rngkit_core::{SOURCE_ID_BITB, SOURCE_ID_PSEUDO, SOURCE_ID_RDSEED, SOURCE_ID_TRNG};
use rngkit_recording::{
    ConcatenationPreview, RecordingError, create_legacy_csv_concatenation, inspect_legacy_csvs,
};

use crate::coordinator::{AppCoordinator, ReportKind};
use crate::dto::{AppStateDto, CombineInputRow, CombineResult, CombineSnapshot, FileJobState};
use crate::errors::SafeError;
use crate::reports::{inspect_derived, write_inspected_report};

pub fn preview_csvs(
    coordinator: &mut AppCoordinator,
    paths: &[PathBuf],
) -> Result<AppStateDto, SafeError> {
    coordinator.begin_file_job(FileJobState::Inspecting)?;
    let result = inspect_legacy_csvs(paths);
    let _ = coordinator.finish_file_job();
    apply_preview(coordinator, paths, result)
}

pub fn create_previewed(coordinator: &mut AppCoordinator) -> Result<AppStateDto, SafeError> {
    coordinator.begin_file_job(FileJobState::Combining)?;
    let paths = coordinator.combine_inputs().to_vec();
    let Some(root) = coordinator.output_root().map(Path::to_path_buf) else {
        let _ = coordinator.finish_file_job();
        return Err(SafeError::invalid_configuration(
            "Choose an output folder before creating a derived bundle.",
        ));
    };
    if paths.is_empty() {
        let _ = coordinator.finish_file_job();
        return Err(SafeError::invalid_configuration(
            "Select compatible legacy v3 CSV files before creating a bundle.",
        ));
    }
    let result = create_legacy_csv_concatenation(&paths, &root);
    let _ = coordinator.finish_file_job();
    match result {
        Ok(directory) => finish_created(coordinator, directory),
        Err(error) => {
            let mapped = map_combine(error);
            coordinator.record_diagnostic(mapped.code, mapped.message());
            Err(mapped)
        }
    }
}

pub fn generate_derived_report(
    coordinator: &mut AppCoordinator,
    replace: bool,
) -> Result<AppStateDto, SafeError> {
    coordinator.begin_file_job(FileJobState::GeneratingReport)?;
    let Some(directory) = coordinator.combine_directory().map(Path::to_path_buf) else {
        let _ = coordinator.finish_file_job();
        return Err(SafeError::invalid_configuration(
            "Create a derived bundle before generating XLSX.",
        ));
    };
    let written = match inspect_derived(&directory) {
        Ok(inspected) => {
            coordinator.set_inspected_report(
                inspected.preview,
                inspected.directory,
                inspected.dest,
                inspected.input,
                inspected.kind,
            );
            write_inspected_report(&directory, ReportKind::Derived, replace)
        }
        Err(error) => Err(error),
    };
    let _ = coordinator.finish_file_job();
    match written {
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

pub(crate) fn apply_preview(
    coordinator: &mut AppCoordinator,
    paths: &[PathBuf],
    result: Result<ConcatenationPreview, RecordingError>,
) -> Result<AppStateDto, SafeError> {
    match result {
        Ok(preview) => {
            coordinator.set_combine_preview(snapshot_from_preview(&preview), paths.to_vec());
            Ok(coordinator.snapshot())
        }
        Err(error) => {
            let snapshot = incompatible_snapshot(paths, &error);
            let mapped = map_combine(error);
            coordinator.record_diagnostic(mapped.code, mapped.message());
            coordinator.set_combine_preview(snapshot, Vec::new());
            Ok(coordinator.snapshot())
        }
    }
}

fn incompatible_snapshot(paths: &[PathBuf], error: &RecordingError) -> CombineSnapshot {
    let affected = affected_basenames(error);
    let global_message = map_combine_ref(error);
    let mut inputs = paths
        .iter()
        .map(|path| {
            let basename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unsupported input")
                .to_owned();
            match inspect_legacy_csvs(std::slice::from_ref(path)) {
                Ok(preview) => {
                    let mut row = row_from_preview(&preview, 0);
                    if affected.iter().any(|name| name == &basename) {
                        row.valid = false;
                        row.error = Some(global_message.clone());
                    }
                    row
                }
                Err(single_error) => invalid_row(basename, map_combine(single_error).message()),
            }
        })
        .collect::<Vec<_>>();
    inputs.sort_by(|left, right| {
        left.first_timestamp
            .cmp(&right.first_timestamp)
            .then_with(|| left.basename.cmp(&right.basename))
    });
    CombineSnapshot {
        inputs,
        compatible: false,
        incompatibility: Some(global_message),
        result: None,
    }
}

fn affected_basenames(error: &RecordingError) -> Vec<String> {
    match error {
        RecordingError::IncompatibleConcatenationInputs {
            left_basename,
            right_basename,
            ..
        }
        | RecordingError::OverlappingConcatenationRanges {
            left_basename,
            right_basename,
        } => vec![left_basename.clone(), right_basename.clone()],
        RecordingError::EmptyConcatenationInput { basename }
        | RecordingError::DuplicateConcatenationInput { basename }
        | RecordingError::ConcatenationInputNotCsv { basename }
        | RecordingError::NativeConcatenationInput { basename }
        | RecordingError::DecreasingConcatenationTimestamp { basename }
        | RecordingError::ConcatenationInputChanged { basename } => vec![basename.clone()],
        _ => Vec::new(),
    }
}

fn map_combine_ref(error: &RecordingError) -> String {
    match error {
        RecordingError::EmptyConcatenationInputs => "Choose at least one legacy v3 CSV file.",
        RecordingError::EmptyConcatenationInput { .. } => "A selected CSV file is empty.",
        RecordingError::DuplicateConcatenationInput { .. } => {
            "The same CSV file was selected more than once."
        }
        RecordingError::ConcatenationInputNotCsv { .. } => {
            "Combine accepts only RngKitPSG v3 CSV files."
        }
        RecordingError::NativeConcatenationInput { .. } => {
            "Native session CSV files cannot be combined."
        }
        RecordingError::IncompatibleConcatenationInputs { .. } => {
            "The selected CSV files are not compatible."
        }
        RecordingError::DecreasingConcatenationTimestamp { .. } => {
            "A selected CSV file has decreasing timestamps."
        }
        RecordingError::OverlappingConcatenationRanges { .. } => {
            "Overlapping timestamp ranges are rejected, including equal boundaries."
        }
        RecordingError::ConcatenationInputChanged { .. } => {
            "An input file changed after preview. Choose the files again."
        }
        RecordingError::AlreadyExists { .. } => "A derived bundle with that name already exists.",
        RecordingError::OnesExceedSampleBits { .. } => "A one-count exceeds the sample size.",
        RecordingError::UnsupportedVersion { .. } | RecordingError::InvalidName { .. } => {
            "That file is not a supported RngKitPSG v3 input."
        }
        RecordingError::Corrupt { .. } | RecordingError::Csv(_) => {
            "A selected CSV file is corrupt."
        }
        RecordingError::Io(io) if io.kind() == ErrorKind::PermissionDenied => {
            "The selected files could not be read."
        }
        RecordingError::ConcatenationWrite { .. } | RecordingError::Commit { .. } => {
            "The operation failed unexpectedly."
        }
        _ => "Those files cannot be combined.",
    }
    .to_owned()
}

fn row_from_preview(preview: &ConcatenationPreview, index: usize) -> CombineInputRow {
    let entry = &preview.inputs()[index];
    CombineInputRow {
        basename: entry.basename().to_owned(),
        source: source_label(preview.source_id().as_str()),
        sample_bits: preview.sample_bits().get(),
        interval_seconds: preview.interval().get(),
        fold: preview.fold().map(|fold| u32::from(fold.get())),
        first_timestamp: rfc3339(entry.first_timestamp()),
        last_timestamp: rfc3339(entry.last_timestamp()),
        rows: entry.row_count(),
        valid: true,
        error: None,
    }
}

fn invalid_row(basename: String, message: &str) -> CombineInputRow {
    CombineInputRow {
        basename,
        source: "—".into(),
        sample_bits: 0,
        interval_seconds: 0,
        fold: None,
        first_timestamp: "—".into(),
        last_timestamp: "—".into(),
        rows: 0,
        valid: false,
        error: Some(message.to_owned()),
    }
}

pub(crate) fn finish_created(
    coordinator: &mut AppCoordinator,
    directory: PathBuf,
) -> Result<AppStateDto, SafeError> {
    let inspected = inspect_derived(&directory)?;
    let stem = directory
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("derived")
        .to_owned();
    let input_count = coordinator.combine_inputs().len() as u32;
    coordinator.set_combine_result(
        CombineResult {
            stem,
            input_count,
            total_rows: inspected.preview.row_count,
        },
        directory,
    );
    coordinator.set_inspected_report(
        inspected.preview,
        inspected.directory,
        inspected.dest,
        inspected.input,
        inspected.kind,
    );
    Ok(coordinator.snapshot())
}

fn snapshot_from_preview(preview: &ConcatenationPreview) -> CombineSnapshot {
    CombineSnapshot {
        inputs: preview
            .inputs()
            .iter()
            .enumerate()
            .map(|(index, _)| row_from_preview(preview, index))
            .collect(),
        compatible: true,
        incompatibility: None,
        result: None,
    }
}

fn source_label(id: &str) -> String {
    match id {
        SOURCE_ID_BITB => "BitBabbler".into(),
        SOURCE_ID_TRNG => "TrueRNG v1/v2/v3".into(),
        SOURCE_ID_RDSEED => "RDSEED".into(),
        SOURCE_ID_PSEUDO => "PseudoRNG".into(),
        other => other.into(),
    }
}

fn rfc3339(timestamp: rngkit_core::UtcTimestamp) -> String {
    serde_json::to_value(timestamp)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "—".into())
}

pub(crate) fn map_combine(error: RecordingError) -> SafeError {
    match error {
        RecordingError::EmptyConcatenationInputs => {
            SafeError::invalid_configuration("Choose at least one legacy v3 CSV file.")
        }
        RecordingError::EmptyConcatenationInput { .. } => {
            SafeError::invalid_configuration("A selected CSV file is empty.")
        }
        RecordingError::DuplicateConcatenationInput { .. } => {
            SafeError::invalid_configuration("The same CSV file was selected more than once.")
        }
        RecordingError::ConcatenationInputNotCsv { .. } => {
            SafeError::unsupported_input("Combine accepts only RngKitPSG v3 CSV files.")
        }
        RecordingError::NativeConcatenationInput { .. } => {
            SafeError::unsupported_input("Native session CSV files cannot be combined.")
        }
        RecordingError::IncompatibleConcatenationInputs { .. } => {
            SafeError::invalid_configuration("The selected CSV files are not compatible.")
        }
        RecordingError::DecreasingConcatenationTimestamp { .. } => {
            SafeError::invalid_configuration("A selected CSV file has decreasing timestamps.")
        }
        RecordingError::OverlappingConcatenationRanges { .. } => SafeError::invalid_configuration(
            "Overlapping timestamp ranges are rejected, including equal boundaries.",
        ),
        RecordingError::ConcatenationInputChanged { .. } => SafeError::operation_conflict(
            "An input file changed after preview. Choose the files again.",
        ),
        RecordingError::AlreadyExists { .. } => SafeError::derived_exists(),
        RecordingError::OnesExceedSampleBits { .. } => {
            SafeError::invalid_configuration("A one-count exceeds the sample size.")
        }
        RecordingError::UnsupportedVersion { .. } | RecordingError::InvalidName { .. } => {
            SafeError::unsupported_input("That file is not a supported RngKitPSG v3 input.")
        }
        RecordingError::Corrupt { .. } | RecordingError::Csv(_) => {
            SafeError::corrupt_input("A selected CSV file is corrupt.")
        }
        RecordingError::Io(io) if io.kind() == ErrorKind::PermissionDenied => {
            SafeError::permission_denied("The selected files could not be read.")
        }
        RecordingError::ConcatenationWrite { .. } | RecordingError::Commit { .. } => {
            SafeError::unexpected_failure()
        }
        _ => SafeError::unsupported_input("Those files cannot be combined."),
    }
}
