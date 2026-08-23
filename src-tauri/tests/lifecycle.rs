//! Close policy, stop-and-exit races, channel-loss recovery, and copy diagnostics.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use rngkit_lib::collection::{CollectionHandle, FailingSender, VecSender};
use rngkit_lib::coordinator::{AppCoordinator, pseudo_candidate};
use rngkit_lib::diagnostics::format_copy;
use rngkit_lib::dto::{CollectionState, ErrorCode};
use rngkit_lib::errors::SafeError;
use rngkit_lib::lifecycle::{ClosePolicy, close_policy};

fn temp_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rngkit-lifecycle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&dir).expect("temp");
    dir
}

fn ready_with_root(root: &std::path::Path) -> AppCoordinator {
    let mut coordinator = AppCoordinator::new();
    let generation = coordinator.begin_discover().expect("discover");
    coordinator
        .complete_discover(generation, vec![pseudo_candidate()])
        .expect("results");
    coordinator.set_output_root(root).expect("root");
    coordinator.select_token("mock-pseudo-1").expect("select");
    coordinator
}

fn assert_copy_is_safe(text: &str) {
    let lower = text.to_ascii_lowercase();
    assert!(!text.contains(":\\"), "{text}");
    assert!(!text.contains("/dev/"), "{text}");
    assert!(!lower.contains("com3"), "{text}");
    assert!(!lower.contains("entropy"), "{text}");
    assert!(!lower.contains("seed"), "{text}");
    assert!(!lower.contains("serial"), "{text}");
    assert!(!lower.contains("selector"), "{text}");
}

#[test]
fn close_policy_never_abandons_an_active_session() {
    assert_eq!(
        close_policy(CollectionState::Collecting),
        ClosePolicy::Confirm
    );
    assert_eq!(
        close_policy(CollectionState::Stopping),
        ClosePolicy::WaitForFinalize
    );
    assert_eq!(close_policy(CollectionState::Idle), ClosePolicy::Allow);
    assert_eq!(close_policy(CollectionState::Completed), ClosePolicy::Allow);
}

#[test]
fn repeated_stop_during_stopping_is_idempotent() {
    let root = temp_root();
    let mut coordinator = ready_with_root(&root);
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
fn stop_then_join_reaches_a_terminal_state() {
    let root = temp_root();
    let coordinator = Mutex::new(ready_with_root(&root));
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
    assert!(matches!(
        snapshot.collection.state,
        CollectionState::Completed | CollectionState::Failed
    ));
    assert_ne!(
        close_policy(snapshot.collection.state),
        ClosePolicy::Confirm
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn channel_loss_is_terminal_and_queryable() {
    let root = temp_root();
    let coordinator = Mutex::new(ready_with_root(&root));
    let plan = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .begin_collection()
        .expect("start");
    let handle = CollectionHandle::fake();
    let sender = FailingSender::on_sample_committed(VecSender::new());
    handle.run_blocking(&coordinator, &sender, plan);
    let coordinator = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let snapshot = coordinator.snapshot();
    assert_eq!(snapshot.collection.state, CollectionState::Failed);
    assert_eq!(
        snapshot.collection.error_code,
        Some(ErrorCode::UnexpectedFailure)
    );
    assert!(!snapshot.diagnostics.is_empty());
    assert_eq!(
        snapshot.collection.error_recovery.as_deref(),
        Some("Start another session if you want to collect again.")
    );
    let copy = format_copy(&coordinator.diagnostics());
    assert_copy_is_safe(&copy);
    assert!(copy.contains("op-"));
    let dump = serde_json::to_string(&snapshot).expect("json");
    assert_copy_is_safe(&dump);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn copied_diagnostics_redact_sensitive_detail() {
    let mut coordinator = AppCoordinator::new();
    coordinator.record_diagnostic(
        ErrorCode::UnexpectedFailure,
        "open failed C:\\Users\\dev\\rng.bin COM3 /dev/ttyUSB0 seed=00ff serial=AABB",
    );
    let copy = format_copy(&coordinator.diagnostics());
    assert_copy_is_safe(&copy);
    assert!(copy.contains("[redacted]"));
    assert!(copy.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn failed_finish_keeps_recovery_action() {
    let root = temp_root();
    let mut coordinator = ready_with_root(&root);
    coordinator.start().expect("start");
    coordinator
        .finish_failed(SafeError::source_unavailable())
        .expect("failed");
    let snapshot = coordinator.snapshot();
    assert_eq!(snapshot.collection.state, CollectionState::Failed);
    assert_eq!(
        snapshot.collection.error_recovery.as_deref(),
        Some("Select another source and try again.")
    );
    assert!(!snapshot.diagnostics.is_empty());
    let _ = fs::remove_dir_all(&root);
}
