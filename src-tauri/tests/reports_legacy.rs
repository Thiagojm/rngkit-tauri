//! Legacy v3 report inspection and generation. Default tests open no hardware.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rngkit_lib::coordinator::AppCoordinator;
use rngkit_lib::dto::ErrorCode;
use rngkit_lib::reports::{
    generate_inspected, inspect_legacy, inspect_picked, write_legacy_report,
};
use rngkit_xlsx::{REF_MINUS, REF_PLUS, SAMPLES_SHEET, SUMMARY_SHEET, with_workbook_write_failure};

static TEMP_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rngkit-legacy-{}-{}-{}",
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

fn hash(path: &Path) -> Vec<u8> {
    fs::read(path).expect("hash")
}

fn write_pair(dir: &Path, stem: &str, csv: &str, bin: &[u8]) -> (PathBuf, PathBuf) {
    let csv_path = dir.join(format!("{stem}.csv"));
    let bin_path = dir.join(format!("{stem}.bin"));
    fs::write(&csv_path, csv).expect("csv");
    fs::write(&bin_path, bin).expect("bin");
    (csv_path, bin_path)
}

#[test]
fn csv_only_bin_only_and_paired_generate_without_mutation() {
    let root = temp_root();
    let stem = "20260821T183000_trng_s16_i1";
    let sample = [0xFFu8, 0x00];
    let csv = "20260821T18:30:00,8\n";
    let (csv_path, bin_path) = write_pair(&root, stem, csv, &sample);
    let before_csv = hash(&csv_path);
    let before_bin = hash(&bin_path);

    let mut coordinator = AppCoordinator::new();
    inspect_picked(&mut coordinator, &csv_path).expect("paired inspect");
    let preview = coordinator.snapshot().reports.preview.expect("preview");
    assert_eq!(preview.kind_label, "Legacy v3 CSV");
    assert_eq!(preview.origin, "Paired BIN and CSV");
    assert_eq!(preview.source, "TrueRNG v1/v2/v3");
    assert_eq!(preview.sample_bits, 16);
    assert_eq!(preview.interval_seconds, 1);
    assert_eq!(preview.fold, None);
    assert_eq!(preview.status, "Completed");
    assert_eq!(preview.row_count, 1);
    assert_eq!(
        preview.warning.as_deref(),
        Some("Timestamps are recorded in the CSV input.")
    );
    assert_safe_json(&serde_json::to_string(&coordinator.snapshot()).expect("json"));

    generate_inspected(&mut coordinator, false).expect("generate paired");
    let dest = coordinator.report_dest().expect("dest").to_path_buf();
    assert_eq!(
        dest.file_name().and_then(|name| name.to_str()),
        Some("20260821T183000_trng_s16_i1.xlsx")
    );
    let bytes = fs::read(&dest).expect("xlsx");
    assert_eq!(&bytes[..2], b"PK");
    assert_eq!(SUMMARY_SHEET, "Summary");
    assert_eq!(SAMPLES_SHEET, "Samples");
    assert_eq!(REF_PLUS, "Reference +1.96");
    assert_eq!(REF_MINUS, "Reference -1.96");
    assert_eq!(hash(&csv_path), before_csv);
    assert_eq!(hash(&bin_path), before_bin);

    fs::remove_file(&dest).expect("remove xlsx");
    fs::remove_file(&bin_path).expect("remove bin");
    let csv_only = inspect_legacy(&csv_path).expect("csv only");
    assert_eq!(csv_only.preview.origin, "CSV only");
    assert_eq!(
        csv_only.preview.warning.as_deref(),
        Some("Timestamps are recorded in the CSV input.")
    );
    write_legacy_report(&csv_path, false).expect("csv generate");
    assert!(dest.is_file());
    assert_eq!(hash(&csv_path), before_csv);

    fs::remove_file(&dest).expect("remove xlsx");
    fs::write(&bin_path, sample).expect("restore bin");
    fs::remove_file(&csv_path).expect("remove csv");
    let bin_only = inspect_legacy(&bin_path).expect("bin only");
    assert_eq!(bin_only.preview.origin, "BIN only");
    assert_eq!(
        bin_only.preview.warning.as_deref(),
        Some("Timestamps are estimated from the filename start and interval.")
    );
    write_legacy_report(&bin_path, false).expect("bin generate");
    assert!(dest.is_file());
    assert_eq!(hash(&bin_path), before_bin);

    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn supplied_compact_timestamp_shape_generates_without_mutation() {
    let root = temp_root();
    let input = root.join("20260824T145947_bitb_s2048_i1_f0.csv");
    fs::write(
        &input,
        concat!(
            "20260824T145948,1014\n",
            "20260824T145949,1025\n",
            "20260824T145950,1044\n",
            "20260824T145951,1056\n",
            "20260824T145952,1026\n",
        ),
    )
    .expect("compact fixture");
    let before = hash(&input);

    let mut coordinator = AppCoordinator::new();
    inspect_picked(&mut coordinator, &input).expect("inspect compact legacy csv");
    let preview = coordinator.snapshot().reports.preview.expect("preview");
    assert_eq!(preview.kind_label, "Legacy v3 CSV");
    assert_eq!(preview.origin, "CSV only");
    assert_eq!(preview.source, "BitBabbler");
    assert_eq!(preview.sample_bits, 2048);
    assert_eq!(preview.interval_seconds, 1);
    assert_eq!(preview.fold, Some(0));
    assert_eq!(preview.row_count, 5);

    generate_inspected(&mut coordinator, false).expect("generate compact report");
    let dest = coordinator.report_dest().expect("dest");
    assert_eq!(&fs::read(dest).expect("xlsx")[..2], b"PK");
    assert_eq!(hash(&input), before);

    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn invalid_legacy_fixtures_fail_without_partial_xlsx() {
    let root = temp_root();

    let v2 = root.join("20260821-183000_trng_s16_i1.csv");
    fs::write(&v2, "18:30:00 8\n").expect("v2");
    let error = inspect_legacy(&v2).expect_err("v2");
    assert_eq!(error.code, ErrorCode::UnsupportedInput);
    assert_safe_json(&serde_json::to_string(&error).expect("json"));
    assert!(!root.join("20260821-183000_trng_s16_i1.xlsx").exists());

    let mismatch_stem = "20260821T183000_bitb_s16_i1_f0";
    let (mismatch, _) = write_pair(&root, mismatch_stem, "20260821T18:30:00,8\n", &[0x00, 0x00]);
    let error = inspect_legacy(&mismatch).expect_err("popcount");
    assert_eq!(error.code, ErrorCode::CorruptInput);
    assert!(!root.join(format!("{mismatch_stem}.xlsx")).exists());

    let partial = root.join("20260821T183000_pseudo_s16_i1.bin");
    fs::write(&partial, [0x00u8]).expect("partial");
    let error = inspect_legacy(&partial).expect_err("partial");
    assert_eq!(error.code, ErrorCode::CorruptInput);
    assert!(!root.join("20260821T183000_pseudo_s16_i1.xlsx").exists());

    let overflow = root.join("20260821T183001_trng_s16_i1.csv");
    fs::write(&overflow, "20260821T18:30:01,17\n").expect("overflow");
    let error = inspect_legacy(&overflow).expect_err("ones");
    assert_eq!(error.code, ErrorCode::InvalidConfiguration);
    assert!(!root.join("20260821T183001_trng_s16_i1.xlsx").exists());

    let space = root.join("20260821T183002_trng_s16_i1.csv");
    fs::write(&space, "20260821T18:30:02 8\n").expect("space");
    let error = inspect_legacy(&space).expect_err("space csv");
    assert_eq!(error.code, ErrorCode::UnsupportedInput);

    let malformed_current = root.join("20260821T183004_trng_s16_i1.csv");
    fs::write(
        &malformed_current,
        "sample_index,timestamp,elapsed_ms\n1,2026-08-21T18:30:04Z,0\n",
    )
    .expect("malformed current");
    let mut coordinator = AppCoordinator::new();
    let error =
        inspect_picked(&mut coordinator, &malformed_current).expect_err("malformed current header");
    assert_eq!(error.code, ErrorCode::CorruptInput);
    assert!(!root.join("20260821T183004_trng_s16_i1.xlsx").exists());

    let derived = root.join("20260821T183003_concat_trng_s16_i1.csv");
    fs::write(&derived, "20260821T18:30:03,8\n").expect("derived");
    let error = inspect_legacy(&derived).expect_err("derived");
    assert_eq!(error.code, ErrorCode::UnsupportedInput);

    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn existing_xlsx_is_not_replaced_without_explicit_request() {
    let root = temp_root();
    let csv_path = root.join("20260821T183000_trng_s16_i1.csv");
    fs::write(&csv_path, "20260821T18:30:00,8\n").expect("csv");
    let mut coordinator = AppCoordinator::new();
    inspect_picked(&mut coordinator, &csv_path).expect("inspect");
    generate_inspected(&mut coordinator, false).expect("first");
    let dest = coordinator.report_dest().expect("dest").to_path_buf();
    let original = fs::read(&dest).expect("bytes");

    let error = generate_inspected(&mut coordinator, false).expect_err("conflict");
    assert_eq!(error.code, ErrorCode::OutputExists);
    assert_eq!(fs::read(&dest).expect("unchanged"), original);
    assert_eq!(hash(&csv_path), b"20260821T18:30:00,8\n".to_vec());

    generate_inspected(&mut coordinator, true).expect("replace");
    let replaced = fs::read(&dest).expect("replaced");
    assert_eq!(&replaced[..2], b"PK");
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn failed_write_leaves_no_xlsx() {
    let root = temp_root();
    let csv_path = root.join("20260821T183000_pseudo_s16_i1.csv");
    fs::write(&csv_path, "20260821T18:30:00,8\n").expect("csv");
    let dest = root.join("20260821T183000_pseudo_s16_i1.xlsx");
    let before = hash(&csv_path);
    with_workbook_write_failure(|| write_legacy_report(&csv_path, false)).expect_err("fail");
    assert!(!dest.exists());
    assert_eq!(hash(&csv_path), before);
    fs::remove_dir_all(&root).expect("cleanup");
}
