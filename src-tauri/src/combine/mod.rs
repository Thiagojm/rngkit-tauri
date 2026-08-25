//! CSV concatenation preview and derived-bundle creation.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use rngkit_core::{SOURCE_ID_BITB, SOURCE_ID_PSEUDO, SOURCE_ID_RDSEED, SOURCE_ID_TRNG};
use rngkit_recording::{
    ConcatenationPreview, RecordingError, StandaloneInputFormat, create_csv_concatenation,
    inspect_csv_inputs,
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
    coordinator.replace_combine_inputs(paths.to_vec());
    let result = inspect_csv_inputs(paths);
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
            "Select compatible CSV files before creating a bundle.",
        ));
    }
    let result = create_csv_concatenation(&paths, &root);
    let _ = coordinator.finish_file_job();
    match result {
        Ok(directory) => finish_created(coordinator, directory),
        Err(error) => {
            let mapped = map_combine(error);
            coordinator.note_combine_failure(mapped.message());
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
                inspected.options,
            );
            let options = coordinator.report_options().cloned().ok_or_else(|| {
                SafeError::invalid_configuration(
                    "Inspect the derived bundle before generating XLSX.",
                )
            });
            options.and_then(|options| {
                write_inspected_report(&directory, ReportKind::Derived, replace, &options)
            })
        }
        Err(error) => Err(error),
    };
    let _ = coordinator.finish_file_job();
    match written {
        Ok(_) => {
            coordinator.mark_report_written(replace);
            Ok(coordinator.snapshot())
        }
        Err(error) if error.code == crate::dto::ErrorCode::OutputExists => {
            coordinator.mark_report_conflict();
            coordinator.note_report_failure(error.message());
            Err(error)
        }
        Err(error) => {
            coordinator.note_report_failure(error.message());
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
            coordinator.set_combine_preview(snapshot_from_preview(
                &preview,
                coordinator.combine_input_ids(),
                paths,
            ));
            Ok(coordinator.snapshot())
        }
        Err(error) => {
            let snapshot = incompatible_snapshot(paths, coordinator.combine_input_ids(), &error);
            let mapped = map_combine(error);
            coordinator.record_diagnostic(mapped.code, mapped.message());
            coordinator.set_combine_preview(snapshot);
            Ok(coordinator.snapshot())
        }
    }
}

fn incompatible_snapshot(
    paths: &[PathBuf],
    input_ids: &[String],
    error: &RecordingError,
) -> CombineSnapshot {
    let affected = affected_basenames(error);
    let global_message = map_combine_ref(error);
    let mut inputs = paths
        .iter()
        .zip(input_ids.iter())
        .map(|(path, input_id)| {
            let basename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unsupported input")
                .to_owned();
            match inspect_csv_inputs(std::slice::from_ref(path)) {
                Ok(preview) => {
                    let mut row = row_from_preview(&preview, 0, input_id, 0);
                    if affected.iter().any(|name| name == &basename) {
                        row.valid = false;
                        row.error = Some(global_message.clone());
                    }
                    row
                }
                Err(single_error) => {
                    invalid_row(input_id, 0, basename, map_combine(single_error).message())
                }
            }
        })
        .collect::<Vec<_>>();
    inputs.sort_by(|left, right| {
        left.first_timestamp
            .cmp(&right.first_timestamp)
            .then_with(|| left.basename.cmp(&right.basename))
    });
    for (index, row) in inputs.iter_mut().enumerate() {
        row.ordinal = u32::try_from(index + 1).unwrap_or(u32::MAX);
    }
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
        | RecordingError::InvalidNativeCsvHeader { basename }
        | RecordingError::DecreasingConcatenationTimestamp { basename }
        | RecordingError::ConcatenationInputChanged { basename } => vec![basename.clone()],
        _ => Vec::new(),
    }
}

fn map_combine_ref(error: &RecordingError) -> String {
    match error {
        RecordingError::EmptyConcatenationInputs => "Choose at least one CSV file.",
        RecordingError::EmptyConcatenationInput { .. } => "A selected CSV file is empty.",
        RecordingError::DuplicateConcatenationInput { .. } => {
            "The same CSV file was selected more than once."
        }
        RecordingError::ConcatenationInputNotCsv { .. } => "Combine accepts CSV files only.",
        RecordingError::InvalidNativeCsvHeader { .. } => {
            "A selected current CSV header is invalid."
        }
        RecordingError::NativeConcatenationInput { .. } => "That current CSV is not supported.",
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

fn row_from_preview(
    preview: &ConcatenationPreview,
    index: usize,
    input_id: &str,
    ordinal: u32,
) -> CombineInputRow {
    let entry = &preview.inputs()[index];
    CombineInputRow {
        input_id: input_id.to_owned(),
        ordinal,
        basename: entry.basename().to_owned(),
        format: entry.format().map(format_label).unwrap_or("unknown").into(),
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

fn invalid_row(input_id: &str, ordinal: u32, basename: String, message: &str) -> CombineInputRow {
    CombineInputRow {
        input_id: input_id.to_owned(),
        ordinal,
        basename,
        format: "unknown".into(),
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
    let notice_directory = directory.clone();
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
        inspected.options,
    );
    coordinator.note_derived_created(&notice_directory);
    Ok(coordinator.snapshot())
}

fn snapshot_from_preview(
    preview: &ConcatenationPreview,
    input_ids: &[String],
    paths: &[PathBuf],
) -> CombineSnapshot {
    let ordered_ids = ordered_input_ids(preview, input_ids, paths);
    CombineSnapshot {
        inputs: preview
            .inputs()
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let ordinal = u32::try_from(index + 1).unwrap_or(u32::MAX);
                let input_id = ordered_ids
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or("combine-unknown");
                row_from_preview(preview, index, input_id, ordinal)
            })
            .collect(),
        compatible: true,
        incompatibility: None,
        result: None,
    }
}

fn ordered_input_ids(
    preview: &ConcatenationPreview,
    input_ids: &[String],
    paths: &[PathBuf],
) -> Vec<String> {
    let mut indexed = paths
        .iter()
        .zip(input_ids.iter())
        .enumerate()
        .filter_map(|(index, (path, input_id))| {
            inspect_csv_inputs(std::slice::from_ref(path))
                .ok()
                .and_then(|single| single.inputs().first().cloned())
                .map(|entry| (index, input_id.clone(), entry))
        })
        .collect::<Vec<_>>();
    indexed.sort_by(|left, right| {
        left.2
            .first_timestamp()
            .cmp(&right.2.first_timestamp())
            .then_with(|| left.2.basename().cmp(right.2.basename()))
            .then_with(|| left.0.cmp(&right.0))
    });
    if indexed.len() == preview.inputs().len() {
        indexed.into_iter().map(|(_, id, _)| id).collect()
    } else {
        input_ids.to_vec()
    }
}

fn format_label(format: StandaloneInputFormat) -> &'static str {
    match format {
        StandaloneInputFormat::CurrentCsv => "current_csv",
        StandaloneInputFormat::LegacyV3Csv => "legacy_v3_csv",
        StandaloneInputFormat::Bin => "bin",
        StandaloneInputFormat::FlatLegacyConcatenation => "flat_legacy_concatenation",
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
            SafeError::invalid_configuration("Choose at least one CSV file.")
        }
        RecordingError::EmptyConcatenationInput { .. } => {
            SafeError::invalid_configuration("A selected CSV file is empty.")
        }
        RecordingError::DuplicateConcatenationInput { .. } => {
            SafeError::invalid_configuration("The same CSV file was selected more than once.")
        }
        RecordingError::ConcatenationInputNotCsv { .. } => {
            SafeError::unsupported_input("Combine accepts CSV files only.")
        }
        RecordingError::InvalidNativeCsvHeader { .. } => {
            SafeError::corrupt_input("A selected current CSV header is invalid.")
        }
        RecordingError::NativeConcatenationInput { .. } => {
            SafeError::unsupported_input("That current CSV is not supported.")
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
