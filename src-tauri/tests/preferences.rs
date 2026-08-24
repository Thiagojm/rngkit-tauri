//! Preference atomicity, schema fallback, dialog mapping, and DTO leak tests.

use std::fs;
use std::path::PathBuf;

use rngkit_lib::commands::dialogs::{FakeFolderPicker, apply_picked_folder};
use rngkit_lib::coordinator::AppCoordinator;
use rngkit_lib::dto::{CollectionState, ThemePreference};
use rngkit_lib::preferences::{
    DEFAULT_OUTPUT_DIRECTORY_NAME, PREFERENCES_FILE_NAME, PreferencesHandle, load_from_path,
    output_root_label,
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
    assert_eq!(outcome.preferences.sample_bits, 2048);
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
    assert_eq!(coordinator.snapshot().collection.sample_bits, 2048);
}

#[test]
fn clean_preferences_prepare_the_default_documents_root() {
    let root = temp_dir();
    let documents = root.join("Documents");
    let handle = PreferencesHandle::load_with_documents(root.join(PREFERENCES_FILE_NAME), || {
        Ok(documents.clone())
    });

    let preferences = handle.current();
    let output_root = preferences.output_root.expect("default root");
    assert_eq!(output_root, documents.join(DEFAULT_OUTPUT_DIRECTORY_NAME));
    assert!(output_root.is_dir());
    assert_eq!(output_root_label(&output_root), "RngKit");
    assert_eq!(preferences.sample_bits, 2048);
    assert!(handle.warning().is_none());
}

#[test]
fn missing_saved_root_falls_back_to_the_default_root() {
    let root = temp_dir();
    let documents = root.join("Documents");
    let custom = root.join("Custom");
    fs::create_dir_all(&custom).expect("custom");
    let prefs_path = root.join(PREFERENCES_FILE_NAME);
    let first = PreferencesHandle::load(prefs_path.clone());
    first
        .save_draft(rngkit_lib::preferences::SessionDraft {
            sample_bits: 16,
            interval_seconds: 1,
            fold: None,
            output_root: Some(custom.clone()),
            theme: ThemePreference::System,
        })
        .expect("save custom root");
    fs::remove_dir(&custom).expect("remove custom root");

    let handle = PreferencesHandle::load_with_documents(prefs_path, || Ok(documents.clone()));
    let output_root = handle.current().output_root.expect("fallback root");
    assert_eq!(output_root, documents.join(DEFAULT_OUTPUT_DIRECTORY_NAME));
    assert_eq!(handle.current().sample_bits, 16);
    assert!(
        handle
            .warning()
            .as_deref()
            .is_some_and(|warning| warning.contains("default RngKit"))
    );
}

#[test]
fn unavailable_documents_leave_a_recoverable_empty_root() {
    let handle =
        PreferencesHandle::load_with_documents(temp_dir().join(PREFERENCES_FILE_NAME), || {
            Err(rngkit_lib::errors::SafeError::permission_denied("blocked"))
        });

    assert!(handle.current().output_root.is_none());
    assert!(
        handle
            .warning()
            .as_deref()
            .is_some_and(|warning| warning.contains("Choose an output folder"))
    );
}
