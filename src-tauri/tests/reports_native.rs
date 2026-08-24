//! Native report inspection and generation. Default tests open no hardware.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use rngkit_lib::collection::{CollectionHandle, VecSender};
use rngkit_lib::coordinator::{AppCoordinator, pseudo_candidate};
use rngkit_lib::dto::{CollectionState, ErrorCode, FileJobState};
use rngkit_lib::reports::{
    ReportsHandle, generate_inspected, inspect_native, inspect_picked, write_native_report,
};
use rngkit_recording::NativeSession;
use rngkit_xlsx::{
    REF_MINUS, REF_PLUS, SAMPLES_SHEET, SUMMARY_SHEET, native_report_path, with_report_promote_hook,
};

static TEMP_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rngkit-reports-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0),
        TEMP_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&dir).expect("temp");
    dir
}

fn assert_safe_json(dump: &str) {
    let lower = dump.to_ascii_lowercase();
    assert!(!dump.contains(":\\"), "{dump}");
    assert!(!dump.contains("/dev/"), "{dump}");
    assert!(!lower.contains("entropy byte"), "{dump}");
    assert!(!lower.contains("seed="), "{dump}");
    assert!(!lower.contains("serial="), "{dump}");
}

fn completed_session() -> (PathBuf, PathBuf) {
    let root = temp_root();
    let mut coordinator = AppCoordinator::new();
    let generation = coordinator.begin_discover().expect("discover");
    coordinator
        .complete_discover(generation, vec![pseudo_candidate()])
        .expect("results");
    coordinator.set_output_root(&root).expect("root");
    coordinator.select_token("mock-pseudo-1").expect("select");
    let coordinator = Mutex::new(coordinator);
    let plan = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .begin_collection()
        .expect("start");
    CollectionHandle::fake().run_blocking(&coordinator, &VecSender::new(), plan);
    let directory = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_directory()
        .expect("dir")
        .to_path_buf();
    (root, directory)
}

#[test]
fn completed_native_session_inspects_and_generates() {
    let (root, directory) = completed_session();
    let mut coordinator = AppCoordinator::new();
    inspect_picked(&mut coordinator, &directory).expect("inspect");
    let snapshot = coordinator.snapshot();
    let preview = snapshot.reports.preview.as_ref().expect("preview");
    assert_eq!(preview.kind_label, "Native session");
    assert_eq!(preview.origin, "Collected session");
    assert_eq!(preview.source, "PseudoRNG");
    assert_eq!(preview.status, "Completed");
    assert_eq!(preview.row_count, 3);
    assert!(!preview.conflict);
    assert!(!snapshot.reports.report_ready);
    assert_eq!(snapshot.file_job, FileJobState::Idle);
    assert_safe_json(&serde_json::to_string(&snapshot).expect("json"));

    generate_inspected(&mut coordinator, false).expect("generate");
    let snapshot = coordinator.snapshot();
    assert!(snapshot.reports.report_ready);
    assert!(snapshot.reports.preview.as_ref().expect("preview").conflict);
    let dest = coordinator.report_dest().expect("dest").to_path_buf();
    assert_eq!(dest.extension().and_then(|ext| ext.to_str()), Some("xlsx"));
    assert_eq!(
        fs::canonicalize(dest.parent().expect("parent")).ok(),
        fs::canonicalize(&directory).ok()
    );
    let bytes = fs::read(&dest).expect("xlsx");
    assert_eq!(&bytes[..2], b"PK");
    assert_eq!(SUMMARY_SHEET, "Summary");
    assert_eq!(SAMPLES_SHEET, "Samples");
    assert_eq!(REF_PLUS, "Reference +1.96");
    assert_eq!(REF_MINUS, "Reference -1.96");

    let handle = ReportsHandle::fake();
    handle.open_known_report(&coordinator).expect("open");
    handle.open_known_folder(&coordinator).expect("folder");
    assert_eq!(handle.opened().len(), 2);
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn bundle_artifacts_and_standalone_current_inputs_use_the_unified_reader() {
    let (root, directory) = completed_session();
    let native = NativeSession::open(&directory).expect("native");

    let mut bundle_file = AppCoordinator::new();
    inspect_picked(&mut bundle_file, native.csv_path()).expect("bundle csv");
    assert_eq!(
        bundle_file
            .snapshot()
            .reports
            .preview
            .expect("preview")
            .kind_label,
        "Native session"
    );
    let mut manifest_file = AppCoordinator::new();
    inspect_picked(&mut manifest_file, &directory.join("manifest.json")).expect("bundle manifest");
    assert_eq!(
        manifest_file
            .snapshot()
            .reports
            .preview
            .expect("manifest preview")
            .kind_label,
        "Native session"
    );

    let standalone_root = root.join("standalone");
    fs::create_dir_all(&standalone_root).expect("standalone root");
    let standalone_csv = standalone_root.join(native.csv_path().file_name().expect("csv name"));
    fs::copy(native.csv_path(), &standalone_csv).expect("copy csv");
    let csv_before = fs::read(&standalone_csv).expect("csv bytes");

    let mut current_csv = AppCoordinator::new();
    inspect_picked(&mut current_csv, &standalone_csv).expect("standalone csv");
    let preview = current_csv.snapshot().reports.preview.expect("preview");
    assert_eq!(preview.kind_label, "Current standalone CSV");
    assert_eq!(preview.origin, "Standalone current CSV");
    generate_inspected(&mut current_csv, false).expect("csv report");
    assert_eq!(
        &fs::read(current_csv.report_dest().expect("dest")).expect("xlsx")[..2],
        b"PK"
    );
    assert_eq!(
        fs::read(&standalone_csv).expect("csv unchanged"),
        csv_before
    );
    fs::remove_file(current_csv.report_dest().expect("csv report dest")).expect("remove csv xlsx");
    fs::remove_file(&standalone_csv).expect("remove csv sibling");

    let standalone_bin = standalone_root.join(native.bin_path().file_name().expect("bin name"));
    fs::copy(native.bin_path(), &standalone_bin).expect("copy bin");
    let bin_before = fs::read(&standalone_bin).expect("bin bytes");
    let mut current_bin = AppCoordinator::new();
    inspect_picked(&mut current_bin, &standalone_bin).expect("standalone bin");
    let preview = current_bin.snapshot().reports.preview.expect("preview");
    assert_eq!(preview.kind_label, "Standalone BIN");
    assert_eq!(preview.origin, "Standalone binary input");
    assert_eq!(
        preview.warning.as_deref(),
        Some("Timestamps are estimated from the filename start and interval.")
    );
    generate_inspected(&mut current_bin, false).expect("bin report");
    assert_eq!(
        &fs::read(current_bin.report_dest().expect("dest")).expect("xlsx")[..2],
        b"PK"
    );
    assert_eq!(
        fs::read(&standalone_bin).expect("bin unchanged"),
        bin_before
    );

    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn interrupted_committed_prefix_inspects() {
    let (root, directory) = completed_session();
    let manifest = directory.join("manifest.json");
    let text = fs::read_to_string(&manifest).expect("manifest");
    fs::write(&manifest, text.replace("\"completed\"", "\"recording\"")).expect("rewrite");
    let inspected = inspect_native(&directory, None).expect("inspect");
    assert_eq!(inspected.preview.status, "Interrupted");
    assert_eq!(inspected.preview.row_count, 3);
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn uncommitted_tail_is_a_safe_warning() {
    let (root, directory) = completed_session();
    let native = NativeSession::open(&directory).expect("native");
    let bin = native.bin_path().to_path_buf();
    drop(native);
    let mut bytes = fs::read(&bin).expect("bin");
    bytes.push(0);
    fs::write(&bin, bytes).expect("tail");
    let inspected = inspect_native(&directory, None).expect("inspect");
    assert_eq!(
        inspected.preview.warning.as_deref(),
        Some(
            "The session has an uncommitted binary tail. Report rows use the committed CSV prefix."
        )
    );
    let dump = serde_json::to_string(&inspected.preview).expect("json");
    assert_safe_json(&dump);
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn live_recording_bundle_is_rejected() {
    let (root, directory) = completed_session();
    let error = inspect_native(&directory, Some(&directory)).expect_err("live");
    assert_eq!(error.code, ErrorCode::OperationConflict);
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn collecting_blocks_file_jobs() {
    let mut collecting = AppCoordinator::new();
    let generation = collecting.begin_discover().expect("discover");
    collecting
        .complete_discover(generation, vec![pseudo_candidate()])
        .expect("results");
    collecting
        .set_output_root_label("Chosen folder")
        .expect("root");
    collecting.select_token("mock-pseudo-1").expect("select");
    collecting.start().expect("start");
    assert_eq!(
        collecting.snapshot().collection.state,
        CollectionState::Collecting
    );
    assert_eq!(
        inspect_picked(&mut collecting, std::path::Path::new("."))
            .expect_err("blocked")
            .code,
        ErrorCode::OperationConflict
    );
}

#[test]
fn corrupt_and_unsupported_inputs_fail_safely() {
    let root = temp_root();
    let missing = inspect_native(&root, None).expect_err("empty");
    assert_eq!(missing.code, ErrorCode::UnsupportedInput);

    let corrupt_dir = root.join("corrupt");
    fs::create_dir_all(&corrupt_dir).expect("dir");
    fs::write(corrupt_dir.join("manifest.json"), b"{not-json").expect("junk");
    let corrupt = inspect_native(&corrupt_dir, None).expect_err("corrupt");
    assert_eq!(corrupt.code, ErrorCode::CorruptInput);
    let corrupt_selected = corrupt_dir.join("20260821T183000_pseudo_s16_i1.csv");
    fs::write(&corrupt_selected, "20260821T18:30:00,8\n").expect("csv");
    let mut coordinator = AppCoordinator::new();
    let error =
        inspect_picked(&mut coordinator, &corrupt_selected).expect_err("corrupt parent manifest");
    assert_eq!(error.code, ErrorCode::CorruptInput);

    let unknown_kind_dir = root.join("unknown-kind");
    fs::create_dir_all(&unknown_kind_dir).expect("unknown kind dir");
    fs::write(
        unknown_kind_dir.join("manifest.json"),
        br#"{"kind":"future_bundle","schema_version":1}"#,
    )
    .expect("unknown kind manifest");
    let selected = unknown_kind_dir.join("input.csv");
    fs::write(&selected, "20260821T18:30:00,8\n").expect("selected file");
    let mut coordinator = AppCoordinator::new();
    let error = inspect_picked(&mut coordinator, &selected).expect_err("unknown manifest kind");
    assert_eq!(error.code, ErrorCode::CorruptInput);

    let invalid_entry_dir = root.join("invalid-manifest-entry");
    fs::create_dir_all(invalid_entry_dir.join("manifest.json")).expect("manifest directory");
    let selected = invalid_entry_dir.join("20260821T183000_trng_s16_i1.csv");
    fs::write(&selected, "20260821T18:30:00,8\n").expect("selected standalone shape");
    let mut coordinator = AppCoordinator::new();
    let error = inspect_picked(&mut coordinator, &selected).expect_err("manifest directory");
    assert_eq!(error.code, ErrorCode::CorruptInput);

    let dump = serde_json::to_string(&corrupt).expect("json");
    assert_safe_json(&dump);
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn unsupported_manifest_schema_fails_safely() {
    let (root, directory) = completed_session();
    let manifest = directory.join("manifest.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest).expect("manifest")).expect("json");
    value["schema_version"] = serde_json::json!(2);
    fs::write(&manifest, serde_json::to_vec(&value).expect("json")).expect("rewrite");

    let error = inspect_native(&directory, None).expect_err("unsupported schema");
    assert_eq!(error.code, ErrorCode::UnsupportedInput);
    assert_safe_json(&serde_json::to_string(&error).expect("json"));
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn report_artifacts_open_only_when_ready_and_idle() {
    let (root, directory) = completed_session();
    let mut coordinator = AppCoordinator::new();
    inspect_picked(&mut coordinator, &directory).expect("inspect");
    let handle = ReportsHandle::fake();
    assert_eq!(
        handle
            .open_known_report(&coordinator)
            .expect_err("not generated")
            .code,
        ErrorCode::InvalidTransition
    );
    assert_eq!(
        handle
            .open_known_folder(&coordinator)
            .expect_err("not generated")
            .code,
        ErrorCode::InvalidTransition
    );

    generate_inspected(&mut coordinator, false).expect("generate");
    coordinator
        .begin_file_job(FileJobState::GeneratingReport)
        .expect("job");
    assert_eq!(
        handle
            .open_known_report(&coordinator)
            .expect_err("busy")
            .code,
        ErrorCode::OperationConflict
    );
    assert_eq!(
        handle
            .open_known_folder(&coordinator)
            .expect_err("busy")
            .code,
        ErrorCode::OperationConflict
    );
    coordinator.finish_file_job().expect("finish");
    handle.open_known_report(&coordinator).expect("report");
    handle.open_known_folder(&coordinator).expect("folder");
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn existing_xlsx_is_not_replaced_without_explicit_request() {
    let (root, directory) = completed_session();
    let mut coordinator = AppCoordinator::new();
    inspect_picked(&mut coordinator, &directory).expect("inspect");
    generate_inspected(&mut coordinator, false).expect("first");
    let dest = coordinator.report_dest().expect("dest").to_path_buf();
    let original = fs::read(&dest).expect("bytes");

    let error = generate_inspected(&mut coordinator, false).expect_err("conflict");
    assert_eq!(error.code, ErrorCode::OutputExists);
    assert_eq!(fs::read(&dest).expect("unchanged"), original);

    generate_inspected(&mut coordinator, true).expect("replace");
    let replaced = fs::read(&dest).expect("replaced");
    assert_eq!(&replaced[..2], b"PK");
    assert_ne!(replaced, b"old-report".to_vec());
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn concurrent_destination_is_not_replaced() {
    let (root, directory) = completed_session();
    inspect_native(&directory, None).expect("inspect");
    let native = NativeSession::open(&directory).expect("native");
    let dest = native_report_path(native.directory(), native.session_stem()).expect("path");
    drop(native);
    let dest_for_hook = dest.clone();
    let err = with_report_promote_hook(
        move |_tmp| {
            fs::write(&dest_for_hook, b"concurrent-destination").unwrap();
        },
        || write_native_report(&directory, false),
    )
    .expect_err("race");
    assert_eq!(err.code, ErrorCode::OutputExists);
    assert_eq!(fs::read(&dest).expect("kept"), b"concurrent-destination");
    fs::remove_dir_all(&root).expect("cleanup");
}
