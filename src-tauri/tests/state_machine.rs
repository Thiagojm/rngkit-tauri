//! Coordinator transition and serialization tests. No hardware, no session files.

use rngkit_lib::coordinator::{AppCoordinator, DevScenario, bitb_candidate, pseudo_candidate};
use rngkit_lib::dto::{CollectionState, ErrorCode, FileJobState};
use rngkit_lib::errors::SafeError;

fn ready_coordinator() -> AppCoordinator {
    let mut coordinator = AppCoordinator::new();
    let generation = coordinator.begin_discover().expect("discover");
    coordinator
        .complete_discover(generation, vec![bitb_candidate(), pseudo_candidate()])
        .expect("results");
    coordinator
        .set_output_root_label("Chosen folder")
        .expect("folder");
    coordinator.select_token("mock-bitb-1").expect("select");
    coordinator
}

fn assert_safe_json(dump: &str) {
    assert!(!dump.contains(":\\"), "{dump}");
    assert!(!dump.contains("/dev/"), "{dump}");
    let lower = dump.to_ascii_lowercase();
    assert!(!lower.contains("entropy byte"), "{dump}");
    assert!(!lower.contains("seed="), "{dump}");
    assert!(!lower.contains("serial="), "{dump}");
    assert!(!lower.contains("selector"), "{dump}");
    assert!(!lower.contains("caused by"), "{dump}");
}

#[test]
fn idle_to_collecting_requires_ready() {
    let mut coordinator = AppCoordinator::new();
    assert_eq!(
        coordinator.start().expect_err("idle").code,
        ErrorCode::InvalidTransition
    );

    let generation = coordinator.begin_discover().expect("begin");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Discovering
    );
    assert_eq!(
        coordinator.start().expect_err("discovering").code,
        ErrorCode::InvalidTransition
    );

    coordinator
        .complete_discover(generation, vec![pseudo_candidate()])
        .expect("complete");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Idle
    );
    coordinator
        .set_output_root_label("Chosen folder")
        .expect("folder");
    coordinator.select_token("mock-pseudo-1").expect("select");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Ready
    );

    let session_id = coordinator.start().expect("start");
    assert_eq!(session_id, "s1");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Collecting
    );
    assert_eq!(coordinator.session_id(), Some("s1"));
    assert_eq!(coordinator.next_event_sequence(), 1);
}

#[test]
fn rejects_every_named_prohibited_transition() {
    let mut idle = AppCoordinator::new();
    assert_eq!(
        idle.request_stop().expect_err("stop idle").code,
        ErrorCode::InvalidTransition
    );
    assert_eq!(
        idle.finish_completed().expect_err("complete idle").code,
        ErrorCode::InvalidTransition
    );
    assert_eq!(
        idle.finish_failed(SafeError::source_unavailable())
            .expect_err("fail idle")
            .code,
        ErrorCode::InvalidTransition
    );
    assert_eq!(
        idle.start_another().expect_err("another idle").code,
        ErrorCode::InvalidTransition
    );
    assert_eq!(
        idle.complete_discover(1, Vec::new())
            .expect_err("results idle")
            .code,
        ErrorCode::InvalidTransition
    );

    let mut collecting = ready_coordinator();
    collecting.start().expect("start");
    assert_eq!(
        collecting.start().expect_err("double start").code,
        ErrorCode::InvalidTransition
    );
    assert_eq!(
        collecting.begin_discover().expect_err("discover live").code,
        ErrorCode::OperationConflict
    );
    assert_eq!(
        collecting
            .begin_file_job(FileJobState::Inspecting)
            .expect_err("file live")
            .code,
        ErrorCode::OperationConflict
    );
    assert_eq!(
        collecting
            .select_token("mock-bitb-1")
            .expect_err("select live")
            .code,
        ErrorCode::InvalidTransition
    );

    collecting.request_stop().expect("stop");
    assert_eq!(
        collecting.begin_discover().expect_err("discover stop").code,
        ErrorCode::OperationConflict
    );
    collecting.request_stop().expect("idempotent stop");
}

#[test]
fn file_jobs_are_mutually_exclusive() {
    let mut coordinator = ready_coordinator();
    coordinator
        .begin_file_job(FileJobState::GeneratingReport)
        .expect("report");
    assert_eq!(
        coordinator.snapshot().file_job,
        FileJobState::GeneratingReport
    );
    assert_eq!(
        coordinator
            .begin_file_job(FileJobState::Combining)
            .expect_err("overlap")
            .code,
        ErrorCode::OperationConflict
    );
    assert_eq!(
        coordinator.start().expect_err("start during job").code,
        ErrorCode::OperationConflict
    );
    coordinator.finish_file_job().expect("done");
    assert_eq!(coordinator.snapshot().file_job, FileJobState::Idle);
}

#[test]
fn terminal_paths_and_start_another() {
    let mut coordinator = ready_coordinator();
    coordinator.start().expect("start");
    coordinator.finish_completed().expect("completed");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Completed
    );
    coordinator.start_another().expect("again");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Ready
    );
    assert!(coordinator.session_id().is_none());

    let mut failed = ready_coordinator();
    failed.start().expect("start");
    failed.request_stop().expect("stop");
    failed
        .finish_failed(SafeError::source_unavailable())
        .expect("failed");
    assert_eq!(failed.snapshot().collection.state, CollectionState::Failed);
    assert_eq!(
        failed.snapshot().collection.error_code,
        Some(ErrorCode::SourceUnavailable)
    );
    assert_eq!(failed.diagnostics().len(), 1);
}

#[test]
fn output_root_label_rejects_paths() {
    let mut coordinator = AppCoordinator::new();
    let error = coordinator
        .set_output_root_label(r"C:\Users\dev\output")
        .expect_err("path");
    assert_eq!(error.code, ErrorCode::InvalidConfiguration);
}

#[test]
fn non_fold_source_rejects_fold_configuration() {
    let mut coordinator = AppCoordinator::new();
    let generation = coordinator.begin_discover().expect("discover");
    coordinator
        .complete_discover(generation, vec![pseudo_candidate()])
        .expect("results");
    coordinator
        .set_output_root_label("Chosen folder")
        .expect("folder");
    coordinator.select_token("mock-pseudo-1").expect("select");

    let error = coordinator.set_fold(Some(0)).expect_err("pseudo fold");
    assert_eq!(error.code, ErrorCode::InvalidConfiguration);
    assert_eq!(coordinator.snapshot().collection.fold, None);
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Ready
    );
}

#[test]
fn fixtures_serialize_without_secrets_or_paths() {
    let scenarios = [
        DevScenario::Idle,
        DevScenario::Discovering,
        DevScenario::Ready,
        DevScenario::Collecting,
        DevScenario::Stopping,
        DevScenario::Completed,
        DevScenario::Failed,
        DevScenario::ReportsPreview,
        DevScenario::ReportsConflict,
        DevScenario::CombineCompatible,
        DevScenario::CombineIncompatible,
    ];
    for scenario in scenarios {
        let mut coordinator = AppCoordinator::new();
        coordinator.load_dev_fixture(scenario);
        let snapshot = coordinator.snapshot();
        assert_safe_json(&serde_json::to_string(&snapshot).expect("json"));
        for row in &snapshot.combine.inputs {
            assert!(!row.basename.contains('/') && !row.basename.contains('\\'));
        }
        if snapshot.collection.state == CollectionState::Collecting {
            assert_eq!(snapshot.collection.session_id.as_deref(), Some("s1"));
            assert_eq!(snapshot.collection.last_event_sequence, 12);
        }
    }
}

#[test]
fn unknown_dev_scenario_is_rejected() {
    assert_eq!(
        DevScenario::parse("nope").expect_err("unknown").code,
        ErrorCode::InvalidConfiguration
    );
}

#[test]
fn snapshot_wire_format_is_camel_case() {
    let mut coordinator = AppCoordinator::new();
    coordinator.load_dev_fixture(DevScenario::Ready);
    let value = serde_json::to_value(coordinator.snapshot()).expect("json");
    assert!(value.get("fileJob").is_some());
    assert_eq!(value["collection"]["statusLabel"], "Ready");
    assert_eq!(value["collection"]["selectedToken"], "mock-bitb-1");
    assert!(value["collection"].get("lastEventSequence").is_some());
    assert!(value["collection"].get("sessionId").is_some());
}
