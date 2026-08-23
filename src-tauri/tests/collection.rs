//! Fake source/clock/channel collection tests. Default tests open no hardware.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use rngkit_core::SourceErrorKind;
use rngkit_lib::collection::{CollectionHandle, FailingSender, VecSender};
use rngkit_lib::coordinator::{AppCoordinator, bitb_candidate, pseudo_candidate};
use rngkit_lib::dto::{CollectionEventDto, CollectionState, ErrorCode, SourceCandidateDto};
use rngkit_recording::NativeSession;
use rngkit_sources::SourceConfig;

fn temp_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rngkit-collect-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&dir).expect("temp");
    dir
}

fn ready_with_root(root: &std::path::Path, token: &str) -> AppCoordinator {
    let mut coordinator = AppCoordinator::new();
    let generation = coordinator.begin_discover().expect("discover");
    coordinator
        .complete_discover(
            generation,
            vec![
                bitb_candidate(),
                trng_candidate(),
                rdseed_candidate(),
                pseudo_candidate(),
            ],
        )
        .expect("results");
    coordinator.set_output_root(root).expect("root");
    coordinator.select_token(token).expect("select");
    coordinator
}

fn trng_candidate() -> SourceCandidateDto {
    SourceCandidateDto {
        token: "mock-trng-1".into(),
        source_id: "trng".into(),
        family_label: "TrueRNG v1/v2/v3".into(),
        variant: None,
        ordinal: 1,
        requires_fold: false,
    }
}

fn rdseed_candidate() -> SourceCandidateDto {
    SourceCandidateDto {
        token: "mock-rdseed-1".into(),
        source_id: "rdseed".into(),
        family_label: "Intel RDSEED".into(),
        variant: None,
        ordinal: 1,
        requires_fold: false,
    }
}

fn assert_safe_json(dump: &str) {
    let lower = dump.to_ascii_lowercase();
    assert!(!dump.contains(":\\"), "{dump}");
    assert!(!dump.contains("/dev/"), "{dump}");
    assert!(!lower.contains("entropy byte"), "{dump}");
    assert!(!lower.contains("seed="), "{dump}");
    assert!(!lower.contains("serial="), "{dump}");
    assert!(!lower.contains("selector"), "{dump}");
}

#[test]
fn fake_session_writes_native_bundle_and_finalizes() {
    let root = temp_root();
    let coordinator = Mutex::new(ready_with_root(&root, "mock-pseudo-1"));
    let plan = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .begin_collection()
        .expect("start");
    let handle = CollectionHandle::fake();
    let sender = VecSender::new();
    handle.run_blocking(&coordinator, &sender, plan);

    let snapshot = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .snapshot();
    assert_eq!(snapshot.collection.state, CollectionState::Completed);
    assert_eq!(snapshot.collection.sample_count, 3);
    let stem = snapshot
        .collection
        .session_stem
        .as_deref()
        .expect("stem")
        .to_owned();
    let directory = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_directory()
        .expect("dir")
        .to_path_buf();
    assert!(directory.ends_with(&stem));
    let names: Vec<_> = fs::read_dir(&directory)
        .expect("read")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .into_string()
                .expect("utf8")
        })
        .collect();
    assert!(names.iter().any(|name| name.ends_with(".bin")));
    assert!(names.iter().any(|name| name.ends_with(".csv")));
    assert!(names.iter().any(|name| name == "manifest.json"));
    NativeSession::open(&directory).expect("native");

    let events = sender.events();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, CollectionEventDto::SessionStarted { .. }))
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, CollectionEventDto::SampleCommitted { .. }))
            .count(),
        3
    );
    for event in &events {
        if let CollectionEventDto::SampleCommitted { cumulative_z, .. } = event {
            assert!(cumulative_z.is_finite());
        }
    }
    assert!(
        events
            .iter()
            .any(|event| matches!(event, CollectionEventDto::CleanStop { .. }))
    );
    let dump = serde_json::to_string(&events).expect("json");
    assert_safe_json(&dump);
    assert_safe_json(&serde_json::to_string(&snapshot).expect("snap"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn double_start_is_rejected() {
    let root = temp_root();
    let mut coordinator = ready_with_root(&root, "mock-pseudo-1");
    coordinator.begin_collection().expect("start");
    let error = coordinator.begin_collection().expect_err("double");
    assert_eq!(error.code, ErrorCode::InvalidTransition);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn repeated_stop_is_safe() {
    let root = temp_root();
    let mut coordinator = ready_with_root(&root, "mock-pseudo-1");
    coordinator.start().expect("start");
    coordinator.request_stop().expect("stop");
    coordinator.request_stop().expect("repeat");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Stopping
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn channel_failure_finalizes_failed_state() {
    let root = temp_root();
    let coordinator = Mutex::new(ready_with_root(&root, "mock-pseudo-1"));
    let plan = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .begin_collection()
        .expect("start");
    let handle = CollectionHandle::fake();
    let sender = FailingSender::on_sample_committed(VecSender::new());
    handle.run_blocking(&coordinator, &sender, plan);
    let snapshot = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .snapshot();
    assert_eq!(snapshot.collection.state, CollectionState::Failed);
    assert_eq!(
        snapshot.collection.error_code,
        Some(ErrorCode::UnexpectedFailure)
    );
    let dump = serde_json::to_string(&snapshot).expect("json");
    assert_safe_json(&dump);
    assert_safe_json(&serde_json::to_string(&sender.events()).expect("events"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn clean_stop_channel_failure_replaces_completed_state_with_failed() {
    let root = temp_root();
    let coordinator = Mutex::new(ready_with_root(&root, "mock-pseudo-1"));
    let plan = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .begin_collection()
        .expect("start");
    let handle = CollectionHandle::fake();
    let sender = FailingSender::on_clean_stop(VecSender::new());

    handle.run_blocking(&coordinator, &sender, plan);

    let snapshot = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .snapshot();
    assert_eq!(snapshot.collection.state, CollectionState::Failed);
    assert_eq!(
        snapshot.collection.error_code,
        Some(ErrorCode::UnexpectedFailure)
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn stale_events_are_ignored() {
    let root = temp_root();
    let mut coordinator = ready_with_root(&root, "mock-pseudo-1");
    coordinator.start().expect("start");
    let error = coordinator
        .ingest_collection_update(
            "s-other",
            rngkit_lib::coordinator::CollectionUpdate::TimingOverrun,
        )
        .expect_err("stale");
    assert_eq!(error.code, ErrorCode::InvalidTransition);
    assert_eq!(coordinator.snapshot().collection.overrun_count, 0);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn maps_every_source_config_variant() {
    let root = temp_root();
    let mut coordinator = ready_with_root(&root, "mock-bitb-1");
    match coordinator.selected_source_config().expect("bitb") {
        SourceConfig::Bitb { fold, serial } => {
            assert_eq!(fold.get(), 0);
            assert!(serial.is_none());
        }
        other => panic!("expected bitb, got {other:?}"),
    }
    coordinator.select_token("mock-trng-1").expect("trng");
    assert!(matches!(
        coordinator.selected_source_config().expect("trng"),
        SourceConfig::Trng { path: None }
    ));
    coordinator.select_token("mock-rdseed-1").expect("rdseed");
    assert!(matches!(
        coordinator.selected_source_config().expect("rdseed"),
        SourceConfig::Rdseed { .. }
    ));
    coordinator.select_token("mock-pseudo-1").expect("pseudo");
    assert!(matches!(
        coordinator.selected_source_config().expect("pseudo"),
        SourceConfig::Pseudo { .. }
    ));
    let dump = format!("{:?}", coordinator.selected_source_config().expect("dbg"));
    assert!(!dump.to_ascii_lowercase().contains("serial="));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn live_pseudo_adapter_writes_native_bundle() {
    let root = temp_root();
    let coordinator = Mutex::new(ready_with_root(&root, "mock-pseudo-1"));
    let plan = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .begin_collection()
        .expect("start");
    let handle = CollectionHandle::pseudo_for_tests(3);
    let sender = VecSender::new();
    handle.run_blocking(&coordinator, &sender, plan);

    let snapshot = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .snapshot();
    assert_eq!(snapshot.collection.state, CollectionState::Completed);
    assert_eq!(snapshot.collection.sample_count, 3);
    let directory = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .session_directory()
        .expect("dir")
        .to_path_buf();
    NativeSession::open(&directory).expect("native");
    let dump = serde_json::to_string(&sender.events()).expect("json");
    assert_safe_json(&dump);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn open_session_folder_uses_backend_known_path_only() {
    let root = temp_root();
    let coordinator = Mutex::new(ready_with_root(&root, "mock-pseudo-1"));
    let plan = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .begin_collection()
        .expect("start");
    let handle = CollectionHandle::fake();
    handle.run_blocking(&coordinator, &VecSender::new(), plan);
    {
        let coordinator = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        handle
            .open_known_session_folder(&coordinator)
            .expect("open");
        let snapshot = coordinator.snapshot();
        let dump = serde_json::to_string(&snapshot).expect("json");
        assert_safe_json(&dump);
        let debug = format!("{coordinator:?}");
        let dir = coordinator
            .session_directory()
            .expect("dir")
            .to_string_lossy();
        if dir.contains(":\\") {
            assert!(!debug.contains(&*dir), "{debug}");
        }
    }
    assert_eq!(handle.opened_folders().len(), 1);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn source_open_failure_finishes_failed() {
    let root = temp_root();
    let coordinator = Mutex::new(ready_with_root(&root, "mock-pseudo-1"));
    let plan = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .begin_collection()
        .expect("start");
    let handle = CollectionHandle::failing_open(SourceErrorKind::NotAvailable);
    let sender = VecSender::new();
    handle.run_blocking(&coordinator, &sender, plan);
    let snapshot = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .snapshot();
    assert_eq!(snapshot.collection.state, CollectionState::Failed);
    assert_eq!(
        snapshot.collection.error_code,
        Some(ErrorCode::SourceUnavailable)
    );
    assert!(sender.events().iter().any(|event| matches!(
        event,
        CollectionEventDto::TerminalFailure {
            code: ErrorCode::SourceUnavailable,
            ..
        }
    )));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn worker_start_failure_finishes_the_matching_session() {
    let root = temp_root();
    let mut coordinator = ready_with_root(&root, "mock-pseudo-1");
    let plan = coordinator.begin_collection().expect("start");

    coordinator
        .finish_worker_failure(
            &plan.session_id,
            rngkit_lib::errors::SafeError::unexpected_failure(),
        )
        .expect("finish failed");

    let snapshot = coordinator.snapshot();
    assert_eq!(snapshot.collection.state, CollectionState::Failed);
    assert_eq!(
        snapshot.collection.error_code,
        Some(ErrorCode::UnexpectedFailure)
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn begin_collection_requires_a_validated_output_directory() {
    let mut coordinator = AppCoordinator::new();
    let generation = coordinator.begin_discover().expect("discover");
    coordinator
        .complete_discover(generation, vec![pseudo_candidate()])
        .expect("results");
    coordinator
        .set_output_root_label("Chosen folder")
        .expect("label");
    coordinator.select_token("mock-pseudo-1").expect("select");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Ready
    );
    let error = coordinator.begin_collection().expect_err("no path");
    assert_eq!(error.code, ErrorCode::InvalidConfiguration);
}
