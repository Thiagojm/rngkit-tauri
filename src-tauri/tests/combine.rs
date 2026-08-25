//! Combine preview, derived creation, and derived reports. No hardware.

use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rngkit_lib::combine::{create_previewed, generate_derived_report, preview_csvs};
use rngkit_lib::coordinator::AppCoordinator;
use rngkit_lib::dto::ErrorCode;
use rngkit_lib::reports::{inspect_derived, inspect_picked};
use rngkit_recording::{
    CSV_CONCATENATION_KIND, ConcatenationFailPoint, with_concatenation_fail_point,
    with_concatenation_inspect_hook,
};
use rngkit_xlsx::{REF_MINUS, REF_PLUS, SAMPLES_SHEET, SUMMARY_SHEET};

const FILE_A: &str = "20260821T18:30:00,8\n20260821T18:30:01,8\n";
const FILE_B: &str = "20260821T18:30:10,4\n20260821T18:30:11,4\n";
const FILE_OVERLAP: &str = "20260821T18:30:01,8\n20260821T18:30:02,8\n";
const CURRENT_HEADER: &str =
    "sample_index,captured_at_utc,elapsed_ms,acquisition_ms,ones,byte_offset,byte_length\n";
const CURRENT_ROWS: &str =
    "1,2026-08-21T18:30:20Z,0,0,8,0,2\n2,2026-08-21T18:30:21Z,1000,0,4,2,2\n";
const STEM_A: &str = "20260821T183000_trng_s16_i1";
const STEM_B: &str = "20260821T183010_trng_s16_i1";
const STEM_C: &str = "20260821T183001_trng_s16_i1";
static COMBINE_TEST_LOCK: Mutex<()> = Mutex::new(());

fn combine_test_lock() -> MutexGuard<'static, ()> {
    COMBINE_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct TempRoot(PathBuf);

impl Deref for TempRoot {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Path> for TempRoot {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_root() -> TempRoot {
    let dir = std::env::temp_dir().join(format!(
        "rngkit-combine-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&dir).expect("temp");
    TempRoot(dir)
}

fn write_csv(dir: &Path, stem: &str, body: &str) -> PathBuf {
    let path = dir.join(format!("{stem}.csv"));
    fs::write(&path, body).expect("csv");
    path
}

fn write_current_csv(dir: &Path, stem: &str) -> PathBuf {
    write_csv(dir, stem, &format!("{CURRENT_HEADER}{CURRENT_ROWS}"))
}

fn hash(path: &Path) -> Vec<u8> {
    fs::read(path).expect("hash")
}

fn assert_safe_json(dump: &str) {
    let lower = dump.to_ascii_lowercase();
    assert!(!dump.contains(":\\"), "{dump}");
    assert!(!dump.contains("/dev/"), "{dump}");
    assert!(!lower.contains("entropy byte"), "{dump}");
    assert!(!lower.contains("seed="), "{dump}");
    assert!(!lower.contains("serial="), "{dump}");
}

fn assert_safe_snapshot(snapshot: &rngkit_lib::dto::AppStateDto, allowed_root: &Path) {
    let mut value = serde_json::to_value(snapshot).expect("snapshot json");
    let outcome = value
        .get_mut("pendingOutcome")
        .and_then(serde_json::Value::as_object_mut)
        .expect("outcome");
    let root = allowed_root.to_string_lossy();
    let canonical = fs::canonicalize(allowed_root)
        .unwrap_or_else(|_| allowed_root.to_path_buf())
        .to_string_lossy()
        .into_owned();
    for row in outcome
        .get("paths")
        .and_then(serde_json::Value::as_array)
        .expect("outcome paths")
    {
        let path = row["path"].as_str().expect("outcome path");
        assert!(
            path.starts_with(&*root) || path.starts_with(&canonical),
            "{path}"
        );
    }
    value["pendingOutcome"] = serde_json::Value::Null;
    assert_safe_json(&value.to_string());
}

fn ready_combine(root: &Path, paths: &[PathBuf]) -> AppCoordinator {
    let mut coordinator = AppCoordinator::new();
    coordinator.set_output_root(root).expect("root");
    preview_csvs(&mut coordinator, paths).expect("preview");
    coordinator
}

#[test]
fn compatible_csvs_create_bundle_without_mutation() {
    let _lock = combine_test_lock();
    let root = temp_root();
    let a = write_csv(&root, STEM_A, FILE_A);
    let b = write_csv(&root, STEM_B, FILE_B);
    let before_a = hash(&a);
    let before_b = hash(&b);

    let mut coordinator = ready_combine(&root, &[b.clone(), a.clone()]);
    let snapshot = coordinator.snapshot();
    assert!(snapshot.combine.compatible);
    assert_eq!(snapshot.combine.inputs.len(), 2);
    assert_eq!(snapshot.combine.inputs[0].basename, format!("{STEM_A}.csv"));
    assert_eq!(snapshot.combine.inputs[0].source, "TrueRNG v1/v2/v3");
    assert_eq!(snapshot.combine.inputs[0].rows, 2);
    assert!(snapshot.combine.inputs.iter().all(|row| row.valid));
    assert_safe_json(&serde_json::to_string(&snapshot).expect("json"));

    create_previewed(&mut coordinator).expect("create");
    let result = coordinator.snapshot().combine.result.expect("result");
    assert!(result.stem.contains("_concat_trng_s16_i1"));
    assert_eq!(result.input_count, 2);
    assert_eq!(result.total_rows, 4);
    let directory = coordinator.combine_directory().expect("dir").to_path_buf();
    let outcome = coordinator.pending_outcome().expect("combine outcome");
    assert_eq!(outcome.title, "Derived bundle created");
    assert_eq!(
        outcome.operation,
        rngkit_lib::dto::OutcomeOperation::Combine
    );
    assert_eq!(outcome.paths.len(), 3);
    assert!(
        outcome
            .actions
            .contains(&rngkit_lib::dto::OutcomeActionId::OpenDerivedFolder)
    );
    let outcome_json = serde_json::to_string(&outcome).expect("outcome json");
    assert!(!outcome_json.contains(&a.to_string_lossy().to_string()));
    assert!(!outcome_json.contains(&b.to_string_lossy().to_string()));
    assert!(directory.join(format!("{}.csv", result.stem)).is_file());
    assert!(directory.join("manifest.json").is_file());
    assert!(!directory.join(format!("{}.bin", result.stem)).exists());

    let manifest = fs::read_to_string(directory.join("manifest.json")).expect("manifest");
    assert!(manifest.contains(CSV_CONCATENATION_KIND));
    assert!(manifest.contains("\"schema_version\": 2"));
    assert_safe_json(&manifest);
    assert!(!manifest.contains(&root.display().to_string()));
    assert_eq!(hash(&a), before_a);
    assert_eq!(hash(&b), before_b);

    assert_safe_snapshot(&coordinator.snapshot(), &root);
    assert!(
        !serde_json::to_string(&coordinator.snapshot())
            .expect("json")
            .contains(&root.display().to_string())
    );
}

#[test]
fn mixed_current_and_legacy_csvs_create_schema_two_bundle() {
    let _lock = combine_test_lock();
    let root = temp_root();
    let current_dir = root.join("current");
    fs::create_dir_all(&current_dir).expect("current dir");
    let legacy = write_csv(&root, STEM_B, FILE_B);
    let current = write_current_csv(&current_dir, "20260821T183020_trng_s16_i1");

    let mut coordinator = ready_combine(&root, &[current, legacy]);
    let snapshot = coordinator.snapshot();
    assert!(snapshot.combine.compatible);
    assert_eq!(snapshot.combine.inputs.len(), 2);
    assert_eq!(snapshot.combine.inputs[0].format, "legacy_v3_csv");
    assert_eq!(snapshot.combine.inputs[1].format, "current_csv");
    assert_ne!(
        snapshot.combine.inputs[0].input_id,
        snapshot.combine.inputs[1].input_id
    );

    create_previewed(&mut coordinator).expect("create mixed bundle");
    let directory = coordinator.combine_directory().expect("dir");
    let manifest = fs::read_to_string(directory.join("manifest.json")).expect("manifest");
    assert!(manifest.contains(CSV_CONCATENATION_KIND));
    assert!(manifest.contains("\"format\": \"current_csv\""));
    assert!(manifest.contains("\"format\": \"legacy_v3_csv\""));
    assert_safe_json(&manifest);
}

#[test]
fn current_only_csv_creates_schema_two_bundle() {
    let _lock = combine_test_lock();
    let root = temp_root();
    let current = write_current_csv(&root, "20260821T183020_trng_s16_i1");

    let mut coordinator = ready_combine(&root, &[current]);
    assert!(coordinator.snapshot().combine.compatible);
    assert_eq!(
        coordinator.snapshot().combine.inputs[0].format,
        "current_csv"
    );
    create_previewed(&mut coordinator).expect("create current-only bundle");

    let directory = coordinator.combine_directory().expect("dir");
    let manifest = fs::read_to_string(directory.join("manifest.json")).expect("manifest");
    assert!(manifest.contains(CSV_CONCATENATION_KIND));
    assert!(manifest.contains("\"format\": \"current_csv\""));
}

#[test]
fn removing_one_opaque_combine_id_preserves_the_other_selection() {
    let _lock = combine_test_lock();
    let root = temp_root();
    let a = write_csv(&root, STEM_A, FILE_A);
    let b = write_csv(&root, STEM_B, FILE_B);
    let mut coordinator = ready_combine(&root, &[a, b]);
    let removed = coordinator.snapshot().combine.inputs[0].input_id.clone();

    coordinator
        .remove_combine_input(&removed)
        .expect("remove selected input");
    assert_eq!(coordinator.combine_inputs().len(), 1);
    assert_eq!(coordinator.combine_input_ids().len(), 1);
    assert_ne!(coordinator.combine_input_ids()[0], removed);

    coordinator.clear_combine_inputs();
    assert!(coordinator.combine_inputs().is_empty());
    assert!(coordinator.snapshot().combine.inputs.is_empty());
}

#[test]
fn same_basename_inputs_from_different_folders_have_distinct_ids_and_ordinals() {
    let _lock = combine_test_lock();
    let root = temp_root();
    let first_dir = root.join("first");
    let second_dir = root.join("second");
    fs::create_dir_all(&first_dir).expect("first dir");
    fs::create_dir_all(&second_dir).expect("second dir");
    let first = write_csv(&first_dir, STEM_A, FILE_A);
    let second = write_csv(&second_dir, STEM_A, FILE_B);

    let coordinator = ready_combine(&root, &[second, first]);
    let rows = coordinator.snapshot().combine.inputs;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].basename, rows[1].basename);
    assert_ne!(rows[0].input_id, rows[1].input_id);
    assert_eq!(rows[0].ordinal, 1);
    assert_eq!(rows[1].ordinal, 2);
}

#[test]
fn overlapping_and_mismatched_inputs_fail_before_a_bundle() {
    let _lock = combine_test_lock();
    let root = temp_root();
    let a = write_csv(&root, STEM_A, FILE_A);
    let overlap = write_csv(&root, STEM_C, FILE_OVERLAP);
    let mut coordinator = AppCoordinator::new();
    coordinator.set_output_root(&root).expect("root");
    preview_csvs(&mut coordinator, &[a.clone(), overlap]).expect("preview");
    let snapshot = coordinator.snapshot();
    assert!(!snapshot.combine.compatible);
    assert_eq!(snapshot.combine.inputs.len(), 2);
    assert!(snapshot.combine.inputs.iter().all(|row| !row.valid));
    assert!(
        snapshot
            .combine
            .inputs
            .iter()
            .all(|row| row.error.as_deref() == snapshot.combine.incompatibility.as_deref())
    );
    assert_eq!(
        snapshot.combine.incompatibility.as_deref(),
        Some("Overlapping timestamp ranges are rejected, including equal boundaries.")
    );
    assert!(snapshot.combine.result.is_none());
    let created: Vec<_> = fs::read_dir(&root)
        .expect("list")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().contains("_concat_"))
        .collect();
    assert!(created.is_empty());

    let bits = write_csv(&root, "20260821T184000_trng_s8_i1", "20260821T18:40:00,4\n");
    preview_csvs(&mut coordinator, &[a, bits]).expect("mismatch");
    let mismatch = coordinator.snapshot();
    assert!(!mismatch.combine.compatible);
    assert_eq!(mismatch.combine.inputs.len(), 2);
    assert!(mismatch.combine.inputs.iter().all(|row| !row.valid));
    assert_eq!(
        mismatch.combine.incompatibility.as_deref(),
        Some("The selected CSV files are not compatible.")
    );
}

#[test]
fn malformed_input_keeps_per_input_validation_rows_without_paths() {
    let _lock = combine_test_lock();
    let root = temp_root();
    let valid = write_csv(&root, STEM_A, FILE_A);
    let malformed = write_csv(&root, "20260821T184000_trng_s16_i1", "not-a-timestamp,4\n");
    let mut coordinator = AppCoordinator::new();
    coordinator.set_output_root(&root).expect("root");
    preview_csvs(&mut coordinator, &[valid, malformed]).expect("preview error");

    let snapshot = coordinator.snapshot();
    assert!(!snapshot.combine.compatible);
    assert_eq!(snapshot.combine.inputs.len(), 2);
    assert!(snapshot.combine.inputs.iter().any(|row| row.valid));
    let invalid = snapshot
        .combine
        .inputs
        .iter()
        .find(|row| !row.valid)
        .expect("invalid row");
    assert_eq!(invalid.source, "—");
    assert!(invalid.error.is_some());
    assert_safe_json(&serde_json::to_string(&snapshot).expect("json"));
    assert!(
        !serde_json::to_string(&snapshot)
            .expect("json")
            .contains(&root.display().to_string())
    );
}

#[test]
fn changed_after_preview_and_write_failure_leave_inputs_and_no_bundle() {
    let _lock = combine_test_lock();
    let root = temp_root();
    let a = write_csv(&root, STEM_A, FILE_A);
    let b = write_csv(&root, STEM_B, FILE_B);
    let before_a = hash(&a);
    let mut coordinator = ready_combine(&root, &[a.clone(), b.clone()]);
    let changed = a.clone();
    let error = with_concatenation_inspect_hook(
        move || {
            fs::write(&changed, "20260821T18:30:00,1\n20260821T18:30:01,1\n").expect("mutate");
        },
        || create_previewed(&mut coordinator),
    )
    .expect_err("changed");
    assert_eq!(error.code, ErrorCode::OperationConflict);
    assert_eq!(
        coordinator
            .pending_outcome()
            .expect("failure outcome")
            .title,
        "Derived bundle not created"
    );
    assert!(coordinator.snapshot().combine.result.is_none());
    assert_ne!(hash(&a), before_a);
    fs::write(&a, FILE_A).expect("restore");
    assert_eq!(hash(&a), before_a);

    let mut coordinator = ready_combine(&root, &[a.clone(), b.clone()]);
    let error = with_concatenation_fail_point(ConcatenationFailPoint::AfterCsvWrite, || {
        create_previewed(&mut coordinator)
    })
    .expect_err("write");
    assert_eq!(error.code, ErrorCode::UnexpectedFailure);
    assert_eq!(
        coordinator.pending_outcome().expect("write outcome").title,
        "Derived bundle not created"
    );
    let created: Vec<_> = fs::read_dir(&root)
        .expect("list")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().contains("_concat_"))
        .collect();
    assert!(created.is_empty());
    assert_eq!(hash(&a), before_a);
}

#[test]
fn derived_report_matches_ordered_rows_and_existing_xlsx_needs_replace() {
    let _lock = combine_test_lock();
    let root = temp_root();
    let a = write_csv(&root, STEM_A, FILE_A);
    let b = write_csv(&root, STEM_B, FILE_B);
    let mut coordinator = ready_combine(&root, &[a, b]);
    create_previewed(&mut coordinator).expect("create");
    let directory = coordinator.combine_directory().expect("dir").to_path_buf();

    inspect_picked(&mut coordinator, &directory).expect("inspect derived");
    let preview = coordinator.snapshot().reports.preview.expect("preview");
    assert_eq!(preview.kind_label, "Derived bundle");
    assert_eq!(preview.origin, "Concatenated legacy v3 CSV");
    assert_eq!(preview.source, "TrueRNG v1/v2/v3");
    assert_eq!(preview.row_count, 4);
    assert_eq!(
        preview.warning.as_deref(),
        Some("Timestamps are copied from the concatenated inputs.")
    );
    assert!(!preview.conflict);

    let manifest = directory.join("manifest.json");
    let mut manifest_coordinator = AppCoordinator::new();
    inspect_picked(&mut manifest_coordinator, &manifest).expect("inspect manifest");
    assert_eq!(
        manifest_coordinator
            .snapshot()
            .reports
            .preview
            .expect("manifest preview")
            .kind_label,
        "Derived bundle"
    );

    generate_derived_report(&mut coordinator, false).expect("xlsx");
    let dest = coordinator.report_dest().expect("dest").to_path_buf();
    assert_eq!(
        dest.file_stem().and_then(|name| name.to_str()),
        directory.file_name().and_then(|name| name.to_str())
    );
    let bytes = fs::read(&dest).expect("xlsx");
    assert_eq!(&bytes[..2], b"PK");
    assert_eq!(SUMMARY_SHEET, "Summary");
    assert_eq!(SAMPLES_SHEET, "Samples");
    assert_eq!(REF_PLUS, "Reference +1.96");
    assert_eq!(REF_MINUS, "Reference -1.96");

    let error = generate_derived_report(&mut coordinator, false).expect_err("exists");
    assert_eq!(error.code, ErrorCode::OutputExists);
    generate_derived_report(&mut coordinator, true).expect("replace");

    let inspected = inspect_derived(&directory).expect("open");
    assert_eq!(inspected.preview.row_count, 4);
    assert!(inspected.preview.conflict);
}
