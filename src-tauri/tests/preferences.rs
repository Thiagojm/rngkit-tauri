//! Preference atomicity, schema fallback, dialog mapping, and DTO leak tests.

use std::fs;
use std::path::PathBuf;

use rngkit_lib::commands::dialogs::{FakeFolderPicker, apply_picked_folder};
use rngkit_lib::coordinator::AppCoordinator;
use rngkit_lib::dto::{CollectionState, ThemePreference};
use rngkit_lib::preferences::{
    PREFERENCES_FILE_NAME, PreferencesHandle, load_from_path, output_root_label,
};

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rngkit-pref-it-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&dir).expect("temp");
    dir
}

fn assert_safe_json(dump: &str) {
    assert!(!dump.contains(":\\"), "{dump}");
    assert!(!dump.contains("/dev/"), "{dump}");
    assert!(!dump.contains("mock-bitb"), "{dump}");
    let lower = dump.to_ascii_lowercase();
    assert!(!lower.contains("entropy byte"), "{dump}");
    assert!(!lower.contains("seed="), "{dump}");
    assert!(!lower.contains("serial="), "{dump}");
}

#[test]
fn handle_round_trip_restores_safe_draft_without_selection() {
    let root = temp_dir();
    let sessions = root.join("Sessions");
    fs::create_dir_all(&sessions).expect("sessions");
    let prefs_path = root.join(PREFERENCES_FILE_NAME);

    let mut coordinator = AppCoordinator::new();
    coordinator.set_sample_bits(16).expect("bits");
    coordinator.set_interval_seconds(2).expect("interval");
    coordinator.set_theme(ThemePreference::Dark);
    coordinator.set_output_root(&sessions).expect("folder");
    let generation = coordinator.begin_discover().expect("discover");
    coordinator
        .complete_discover(generation, vec![rngkit_lib::coordinator::bitb_candidate()])
        .expect("discover");
    coordinator.select_token("mock-bitb-1").expect("select");
    coordinator.set_fold(Some(3)).expect("fold");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Ready
    );

    let handle = PreferencesHandle::load(prefs_path.clone());
    handle
        .save_draft(coordinator.session_draft())
        .expect("save");

    let mut restored = AppCoordinator::new();
    restored.apply_persisted_draft(&load_from_path(&prefs_path).preferences);
    let snapshot = restored.snapshot();
    assert_eq!(snapshot.collection.sample_bits, 16);
    assert_eq!(snapshot.collection.interval_seconds, 2);
    assert_eq!(snapshot.theme, ThemePreference::Dark);
    assert_eq!(
        snapshot.collection.output_root_label.as_deref(),
        Some("Sessions")
    );
    assert!(snapshot.collection.selected_token.is_none());
    assert!(snapshot.collection.candidates.is_empty());
    assert_eq!(snapshot.collection.state, CollectionState::Idle);
    assert_safe_json(&serde_json::to_string(&snapshot).expect("json"));

    let file = fs::read_to_string(&prefs_path).expect("read");
    assert!(file.contains("Sessions") || file.contains("outputRoot"));
    assert!(!file.contains("mock-bitb-1"));
    assert!(!file.contains("selectedToken"));
    assert!(!file.contains("sourceId"));
    assert!(!file.contains("bitb"));
}

#[test]
fn corrupt_preferences_leave_session_files_untouched() {
    let root = temp_dir();
    let session = root.join("20260823T100000_bitb_s8_i1.csv");
    fs::write(&session, b"sample_index,captured_at_utc,ones\n").expect("session");
    let prefs_path = root.join(PREFERENCES_FILE_NAME);
    fs::write(&prefs_path, b"{broken").expect("prefs");

    let outcome = load_from_path(&prefs_path);
    assert!(outcome.warning.is_some());
    assert_eq!(outcome.preferences.sample_bits, 8);
    assert_eq!(
        fs::read(&session).expect("read session"),
        b"sample_index,captured_at_utc,ones\n"
    );
}

#[test]
fn fake_dialog_applies_a_validated_directory_label_only() {
    let root = temp_dir();
    let folder = root.join("Output");
    fs::create_dir_all(&folder).expect("folder");
    let mut coordinator = AppCoordinator::new();
    let picker = FakeFolderPicker::with_folder(folder.clone());
    let snapshot = apply_picked_folder(&mut coordinator, &picker).expect("pick");
    assert_eq!(
        snapshot.collection.output_root_label.as_deref(),
        Some("Output")
    );
    assert_eq!(
        output_root_label(&folder).as_str(),
        snapshot.collection.output_root_label.as_deref().unwrap()
    );
    let dump = serde_json::to_string(&snapshot).expect("json");
    assert_safe_json(&dump);
    assert!(!dump.contains(&folder.to_string_lossy().replace('/', "\\")));
}

#[test]
fn cancelled_dialog_keeps_existing_draft() {
    let mut coordinator = AppCoordinator::new();
    coordinator
        .set_output_root_label("Chosen folder")
        .expect("label");
    let snapshot =
        apply_picked_folder(&mut coordinator, &FakeFolderPicker::cancelled()).expect("cancel");
    assert_eq!(
        snapshot.collection.output_root_label.as_deref(),
        Some("Chosen folder")
    );
}

#[test]
fn invalid_bits_do_not_reach_ready() {
    let mut coordinator = AppCoordinator::new();
    let generation = coordinator.begin_discover().expect("discover");
    coordinator
        .complete_discover(
            generation,
            vec![rngkit_lib::coordinator::pseudo_candidate()],
        )
        .expect("complete");
    coordinator
        .set_output_root_label("Chosen folder")
        .expect("folder");
    coordinator.select_token("mock-pseudo-1").expect("select");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Ready
    );
    let error = coordinator.set_sample_bits(7).expect_err("invalid");
    assert_eq!(error.code, rngkit_lib::dto::ErrorCode::InvalidConfiguration);
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Ready
    );
    assert_eq!(coordinator.snapshot().collection.sample_bits, 8);
}
