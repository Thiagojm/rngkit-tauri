//! Ignored, opt-in, serial physical-source smokes. Default `cargo test` compiles
//! this file and does not run it. Only genuine absence may skip.
//!
//! ```text
//! cargo test --manifest-path src-tauri/Cargo.toml --test hardware bitb -- --ignored --test-threads=1 --nocapture
//! ```

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use rngkit_lib::collection::{CollectionHandle, VecSender};
use rngkit_lib::coordinator::AppCoordinator;
use rngkit_lib::discovery::LiveDiscovery;
use rngkit_lib::dto::{CollectionEventDto, CollectionState, SourceCandidateDto};
use rngkit_recording::{ManifestStatus, NativeSession};
use rngkit_sources::{SourceCandidate, SourceConfig};

static HARDWARE: Mutex<()> = Mutex::new(());

const SAMPLES: u32 = 3;

fn temp_root() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rngkit-hardware-{}-{}",
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
    let lower = dump.to_ascii_lowercase();
    assert!(!dump.contains(":\\"), "{dump}");
    assert!(!dump.contains("/dev/"), "{dump}");
    assert!(!lower.contains("entropy byte"), "{dump}");
    assert!(!lower.contains("seed="), "{dump}");
    assert!(!lower.contains("serial"), "{dump}");
    assert!(!lower.contains("selector"), "{dump}");
}

fn collect_bitb(coordinator: &Mutex<AppCoordinator>, candidate: &SourceCandidateDto) {
    let root = temp_root();
    let variant = candidate
        .variant
        .as_deref()
        .expect("BitBabbler variant")
        .to_owned();
    {
        let mut inner = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.set_output_root(&root).expect("root");
        inner.select_token(&candidate.token).expect("select");
        inner.set_fold(Some(0)).expect("fold 0");
        let snapshot = inner.snapshot();
        assert_eq!(
            snapshot.collection.selected_token.as_deref(),
            Some(candidate.token.as_str())
        );
        assert_eq!(snapshot.collection.fold, Some(0));
        let (lib_variant, serial) = match inner.selected_library_source() {
            Some(SourceCandidate::Bitb {
                serial, variant, ..
            }) => (variant.clone(), serial.clone()),
            _ => panic!("selected library source is not BitBabbler"),
        };
        assert_eq!(lib_variant, variant);
        let SourceConfig::Bitb {
            fold,
            serial: config_serial,
        } = inner.selected_source_config().expect("config")
        else {
            panic!("reconstructed config is not BitBabbler");
        };
        assert_eq!(fold.get(), 0);
        assert_eq!(config_serial.as_ref(), Some(&serial));
    }

    let plan = coordinator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .begin_collection()
        .expect("start");
    let handle = CollectionHandle::live_for_tests(SAMPLES);
    let sender = VecSender::new();
    handle.run_blocking(coordinator, &sender, plan);

    let (snapshot, directory) = {
        let mut inner = coordinator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let snapshot = inner.snapshot();
        assert_eq!(
            snapshot.collection.selected_token.as_deref(),
            Some(candidate.token.as_str()),
            "another candidate must not replace the explicit selection"
        );
        assert_eq!(snapshot.collection.state, CollectionState::Completed);
        assert_eq!(snapshot.collection.sample_count, u64::from(SAMPLES));
        assert_eq!(snapshot.collection.fold, Some(0));
        let directory = inner.session_directory().expect("dir").to_path_buf();
        inner.start_another().expect("reset");
        (snapshot, directory)
    };

    let native = NativeSession::open(&directory).expect("native");
    let manifest = native.manifest();
    assert_eq!(manifest.status(), ManifestStatus::Completed);
    assert_eq!(manifest.source_id().as_str(), "bitb");
    assert_eq!(manifest.source_variant(), Some(variant.as_str()));
    assert_eq!(manifest.fold().map(|fold| fold.get()), Some(0));
    assert_eq!(manifest.committed_samples(), u64::from(SAMPLES));
    assert_eq!(native.records().len(), usize::try_from(SAMPLES).expect("n"));
    assert!(
        sender
            .events()
            .iter()
            .any(|event| matches!(event, CollectionEventDto::CleanStop { .. }))
    );
    assert_safe_json(&serde_json::to_string(&snapshot).expect("snap"));
    assert_safe_json(&serde_json::to_string(&sender.events()).expect("events"));
    eprintln!(
        "BitBabbler fold=0 variant={} ordinal={} samples={} native=ok",
        variant, candidate.ordinal, SAMPLES
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
#[ignore]
fn bitb_fold0_explicit_selection_writes_native_bundle() {
    let _guard = HARDWARE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut coordinator = AppCoordinator::new();
    let snapshot = coordinator
        .refresh_with(&LiveDiscovery)
        .expect("live discover");
    assert!(
        snapshot.collection.selected_token.is_none(),
        "discovery must not silently select a source"
    );

    let bitbs: Vec<SourceCandidateDto> = snapshot
        .collection
        .candidates
        .iter()
        .filter(|candidate| candidate.source_id == "bitb")
        .cloned()
        .collect();
    if bitbs.is_empty() {
        if snapshot
            .collection
            .family_warning
            .as_deref()
            .is_some_and(|warning| warning.contains("BitBabbler discovery reported a problem"))
        {
            panic!("BitBabbler discovery reported a problem; absence is the only skip");
        }
        eprintln!(
            "UNVERIFIED: no BitBabbler attached; Checkpoint 11A remains unverified, not passed."
        );
        return;
    }

    let tokens: HashSet<&str> = bitbs
        .iter()
        .map(|candidate| candidate.token.as_str())
        .collect();
    assert_eq!(
        tokens.len(),
        bitbs.len(),
        "each candidate needs its own token"
    );
    for candidate in &bitbs {
        assert_eq!(candidate.family_label, "BitBabbler");
        assert!(candidate.requires_fold);
        assert!(
            candidate
                .variant
                .as_deref()
                .is_some_and(|value| !value.is_empty())
        );
        assert!(candidate.ordinal >= 1);
        eprintln!(
            "discovered BitBabbler variant={} ordinal={}",
            candidate.variant.as_deref().expect("variant"),
            candidate.ordinal
        );
    }

    let shared = Mutex::new(coordinator);
    let mut used_serials = Vec::new();
    for candidate in &bitbs {
        {
            let mut inner = shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inner.select_token(&candidate.token).expect("select");
            let serial = match inner.selected_library_source() {
                Some(SourceCandidate::Bitb { serial, .. }) => serial.clone(),
                _ => panic!("explicit BitBabbler selection lost the library candidate"),
            };
            assert!(
                !used_serials.iter().any(|seen| seen == &serial),
                "two BitBabbler candidates reconstructed the same backend selector"
            );
            used_serials.push(serial);
        }
        collect_bitb(&shared, candidate);
    }
}
