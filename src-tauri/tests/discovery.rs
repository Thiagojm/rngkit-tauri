//! Discovery mapping, generation invalidation, and selector leak tests.
//! Default tests inject [`FakeDiscovery`] and never enumerate or open hardware.

use rngkit_core::{SourceErrorKind, SourceId};
use rngkit_lib::coordinator::AppCoordinator;
use rngkit_lib::discovery::{DiscoveryService, FakeDiscovery};
use rngkit_lib::dto::{CollectionState, ErrorCode};

fn assert_safe_json(dump: &str) {
    let lower = dump.to_ascii_lowercase();
    assert!(!dump.contains(":\\"), "{dump}");
    assert!(!dump.contains("/dev/"), "{dump}");
    assert!(!dump.contains("COM3"), "{dump}");
    assert!(!dump.contains("COM12"), "{dump}");
    assert!(!lower.contains("serial="), "{dump}");
    assert!(!lower.contains("selector"), "{dump}");
    assert!(!lower.contains("entropy"), "{dump}");
    assert!(!lower.contains("seed="), "{dump}");
}

#[test]
fn empty_discovery_stays_idle_without_selection() {
    let mut coordinator = AppCoordinator::new();
    let snapshot = coordinator
        .refresh_with(&FakeDiscovery::empty())
        .expect("empty");
    assert_eq!(snapshot.collection.state, CollectionState::Idle);
    assert!(snapshot.collection.candidates.is_empty());
    assert!(snapshot.collection.selected_token.is_none());
    assert!(snapshot.collection.family_warning.is_none());
    assert_eq!(snapshot.collection.sample_bits, 2048);
}

#[test]
fn partial_discovery_keeps_other_families_selectable() {
    let discovery = FakeDiscovery::empty()
        .with_pseudo("fake-pseudo-1")
        .with_issue(SourceId::bitb(), SourceErrorKind::PermissionDenied);
    let mut coordinator = AppCoordinator::new();
    let snapshot = coordinator.refresh_with(&discovery).expect("partial");

    assert_eq!(snapshot.collection.candidates.len(), 1);
    assert_eq!(snapshot.collection.candidates[0].token, "fake-pseudo-1");
    assert_eq!(snapshot.collection.candidates[0].source_id, "pseudo");
    assert!(snapshot.collection.selected_token.is_none());
    let warning = snapshot
        .collection
        .family_warning
        .as_deref()
        .expect("warning");
    assert!(warning.contains("BitBabbler"));
    assert!(warning.contains("permission was denied"));
    assert_eq!(
        coordinator.diagnostics()[0].code,
        ErrorCode::PermissionDenied
    );
    assert_safe_json(&serde_json::to_string(&snapshot).expect("json"));
}

#[test]
fn multiple_devices_stay_separate_and_never_auto_select() {
    let discovery = FakeDiscovery::empty()
        .with_trng("fake-trng-1", "COM3")
        .with_trng("fake-trng-2", r"\\.\COM12")
        .with_bitb("fake-bitb-1", "White", "ABCDEF0123456789")
        .with_rdseed("fake-rdseed-1")
        .with_pseudo("fake-pseudo-1");
    let mut coordinator = AppCoordinator::new();
    let snapshot = coordinator.refresh_with(&discovery).expect("discover");

    assert_eq!(snapshot.collection.candidates.len(), 5);
    assert_eq!(snapshot.collection.candidates[0].ordinal, 1);
    assert_eq!(snapshot.collection.candidates[1].ordinal, 2);
    assert_eq!(
        snapshot.collection.candidates[0].family_label,
        "TrueRNG v1/v2/v3"
    );
    assert!(snapshot.collection.selected_token.is_none());
    assert_eq!(snapshot.collection.state, CollectionState::Idle);

    coordinator.select_token("fake-trng-2").expect("select");
    assert_eq!(
        coordinator.snapshot().collection.selected_token.as_deref(),
        Some("fake-trng-2")
    );
    assert!(coordinator.selected_library_source().is_none());

    let dump = serde_json::to_string(&coordinator.snapshot()).expect("json");
    assert_safe_json(&dump);
    assert!(!dump.contains("ABCDEF0123456789"), "{dump}");
}

#[test]
fn refresh_expires_previous_tokens() {
    let first = FakeDiscovery::empty().with_pseudo("fake-pseudo-1");
    let mut coordinator = AppCoordinator::new();
    coordinator.refresh_with(&first).expect("first");
    coordinator.select_token("fake-pseudo-1").expect("select");

    let second = FakeDiscovery::empty().with_pseudo("fake-pseudo-2");
    coordinator.refresh_with(&second).expect("refresh");
    assert!(coordinator.snapshot().collection.selected_token.is_none());
    let error = coordinator
        .select_token("fake-pseudo-1")
        .expect_err("expired");
    assert_eq!(error.code, ErrorCode::ExpiredSelection);
    coordinator.select_token("fake-pseudo-2").expect("fresh");
}

#[test]
fn coordinator_debug_omits_library_selectors() {
    let discovery = FakeDiscovery::empty().with_bitb("fake-bitb-1", "White", "SUPER-SECRET-SERIAL");
    let mut coordinator = AppCoordinator::new();
    coordinator.refresh_with(&discovery).expect("discover");
    coordinator.select_token("fake-bitb-1").expect("select");
    let dump = format!("{coordinator:?}");
    assert!(!dump.contains("SUPER-SECRET-SERIAL"), "{dump}");
    assert!(!dump.contains("serial"), "{dump}");
}

#[test]
fn stale_discovery_generation_is_ignored() {
    let mut coordinator = AppCoordinator::new();
    let generation = coordinator.begin_discover().expect("begin");
    let outcome = FakeDiscovery::empty()
        .with_pseudo("fake-pseudo-1")
        .discover();
    let error = coordinator
        .apply_discovery(generation.saturating_sub(1), outcome)
        .expect_err("stale");
    assert_eq!(error.code, ErrorCode::InvalidTransition);
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Discovering
    );
}

#[test]
fn discovery_does_not_open_a_source() {
    let discovery = FakeDiscovery::empty().with_pseudo("fake-pseudo-1");
    let mut coordinator = AppCoordinator::new();
    coordinator.refresh_with(&discovery).expect("discover");
    coordinator
        .set_output_root_label("Chosen folder")
        .expect("folder");
    coordinator.select_token("fake-pseudo-1").expect("select");
    assert_eq!(
        coordinator.snapshot().collection.state,
        CollectionState::Ready
    );
    assert!(coordinator.selected_library_source().is_none());
}
