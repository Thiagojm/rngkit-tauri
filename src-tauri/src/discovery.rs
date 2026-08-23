//! Discovery adapter. Default tests inject [`FakeDiscovery`] and never enumerate hardware.

use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rngkit_core::{SourceErrorKind, SourceId};
use rngkit_sources::{DiscoveryReport, SourceCandidate};

use crate::dto::{ErrorCode, SourceCandidateDto};

/// Backend-only discovery result. Library selectors stay off the DTO.
pub struct DiscoveryOutcome {
    pub candidates: Vec<MappedCandidate>,
    pub warnings: Vec<String>,
    pub diagnostics: Vec<(ErrorCode, String)>,
}

impl DiscoveryOutcome {
    #[must_use]
    pub fn family_warning(&self) -> Option<String> {
        if self.warnings.is_empty() {
            None
        } else {
            Some(self.warnings.join(" "))
        }
    }
}

/// One discovered source: safe view plus the non-serializable library candidate.
pub struct MappedCandidate {
    pub view: SourceCandidateDto,
    pub source: Option<SourceCandidate>,
}

impl std::fmt::Debug for MappedCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MappedCandidate")
            .field("view", &self.view)
            .field("has_library_source", &self.source.is_some())
            .finish()
    }
}

pub trait DiscoveryService: Send + Sync {
    fn discover(&self) -> DiscoveryOutcome;
}

/// Production service. Calls `rngkit_sources::discover()` on the calling thread.
pub struct LiveDiscovery;

/// Injected discovery for deterministic tests. Never calls `discover()`.
#[derive(Clone, Default)]
pub struct FakeDiscovery {
    candidates: Vec<FakeCandidateSpec>,
    issues: Vec<FakeIssue>,
}

#[derive(Clone)]
struct FakeCandidateSpec {
    token: String,
    source_id: &'static str,
    family_label: &'static str,
    variant: Option<String>,
    requires_fold: bool,
    /// Backend-only selector used to prove it never reaches the DTO.
    hidden_selector: Option<String>,
}

#[derive(Clone)]
struct FakeIssue {
    source_id: SourceId,
    kind: SourceErrorKind,
}

impl FakeDiscovery {
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_bitb(mut self, token: &str, variant: &str, serial: &str) -> Self {
        self.candidates.push(FakeCandidateSpec {
            token: token.to_owned(),
            source_id: "bitb",
            family_label: "BitBabbler",
            variant: Some(variant.to_owned()),
            requires_fold: true,
            hidden_selector: Some(serial.to_owned()),
        });
        self
    }

    #[must_use]
    pub fn with_trng(mut self, token: &str, port_name: &str) -> Self {
        self.candidates.push(FakeCandidateSpec {
            token: token.to_owned(),
            source_id: "trng",
            family_label: "TrueRNG v1/v2/v3",
            variant: None,
            requires_fold: false,
            hidden_selector: Some(port_name.to_owned()),
        });
        self
    }

    #[must_use]
    pub fn with_rdseed(mut self, token: &str) -> Self {
        self.candidates.push(FakeCandidateSpec {
            token: token.to_owned(),
            source_id: "rdseed",
            family_label: "Intel RDSEED",
            variant: None,
            requires_fold: false,
            hidden_selector: None,
        });
        self
    }

    #[must_use]
    pub fn with_pseudo(mut self, token: &str) -> Self {
        self.candidates.push(FakeCandidateSpec {
            token: token.to_owned(),
            source_id: "pseudo",
            family_label: "PseudoRNG",
            variant: None,
            requires_fold: false,
            hidden_selector: None,
        });
        self
    }

    #[must_use]
    pub fn with_issue(mut self, source_id: SourceId, kind: SourceErrorKind) -> Self {
        self.issues.push(FakeIssue { source_id, kind });
        self
    }

    #[cfg(test)]
    fn hidden_selectors(&self) -> Vec<&str> {
        self.candidates
            .iter()
            .filter_map(|spec| spec.hidden_selector.as_deref())
            .collect()
    }
}

impl DiscoveryService for FakeDiscovery {
    fn discover(&self) -> DiscoveryOutcome {
        let parts = self
            .candidates
            .iter()
            .map(|spec| {
                let _ = spec.hidden_selector.as_ref();
                CandidateParts {
                    token: Some(spec.token.clone()),
                    source_id: spec.source_id.to_owned(),
                    family_label: spec.family_label.to_owned(),
                    variant: spec.variant.clone(),
                    requires_fold: spec.requires_fold,
                    library: None,
                }
            })
            .collect();
        map_parts(
            parts,
            self.issues.iter().map(|issue| {
                (
                    issue.source_id.clone(),
                    issue.kind,
                    format!("{}: {}", issue.source_id.as_str(), issue.kind),
                )
            }),
        )
    }
}

/// Shared handle stored in Tauri state.
#[derive(Clone)]
pub struct DiscoveryHandle {
    inner: Arc<dyn DiscoveryService>,
}

impl DiscoveryHandle {
    #[must_use]
    pub fn live() -> Self {
        Self {
            inner: Arc::new(LiveDiscovery),
        }
    }

    #[must_use]
    pub fn fake(service: FakeDiscovery) -> Self {
        Self {
            inner: Arc::new(service),
        }
    }

    #[must_use]
    pub fn discover(&self) -> DiscoveryOutcome {
        self.inner.discover()
    }
}

struct CandidateParts {
    token: Option<String>,
    source_id: String,
    family_label: String,
    variant: Option<String>,
    requires_fold: bool,
    library: Option<SourceCandidate>,
}

impl DiscoveryService for LiveDiscovery {
    fn discover(&self) -> DiscoveryOutcome {
        map_report(rngkit_sources::discover())
    }
}

fn map_report(report: DiscoveryReport) -> DiscoveryOutcome {
    let parts = report
        .candidates()
        .iter()
        .cloned()
        .map(parts_from_library)
        .collect();
    map_parts(
        parts,
        report.issues().iter().map(|issue| {
            (
                issue.source_id().clone(),
                issue.error().kind(),
                issue.error().to_string(),
            )
        }),
    )
}

fn parts_from_library(source: SourceCandidate) -> CandidateParts {
    let source_id = source.source_id().as_str().to_owned();
    let family_label = source.label().to_owned();
    let variant = match &source {
        SourceCandidate::Bitb { variant, .. } => Some(variant.clone()),
        _ => None,
    };
    let requires_fold = source_id == "bitb";
    CandidateParts {
        token: None,
        source_id,
        family_label,
        variant,
        requires_fold,
        library: Some(source),
    }
}

fn map_parts(
    parts: Vec<CandidateParts>,
    issues: impl IntoIterator<Item = (SourceId, SourceErrorKind, String)>,
) -> DiscoveryOutcome {
    let mut ordinals = HashMap::<String, u32>::new();
    let candidates = parts
        .into_iter()
        .map(|part| {
            let ordinal_key = match &part.variant {
                Some(variant) => format!("{}:{variant}", part.source_id),
                None => part.source_id.clone(),
            };
            let ordinal = ordinals.entry(ordinal_key).or_insert(0);
            *ordinal += 1;
            let ordinal = *ordinal;
            let token = part.token.unwrap_or_else(new_opaque_token);
            MappedCandidate {
                view: SourceCandidateDto {
                    token,
                    source_id: part.source_id,
                    family_label: part.family_label,
                    variant: part.variant,
                    ordinal,
                    requires_fold: part.requires_fold,
                },
                source: part.library,
            }
        })
        .collect();

    let mut warnings = Vec::new();
    let mut diagnostics = Vec::new();
    for (source_id, kind, detail) in issues {
        warnings.push(safe_issue_warning(source_id.as_str(), kind));
        diagnostics.push((error_code_for(kind), detail));
    }

    DiscoveryOutcome {
        candidates,
        warnings,
        diagnostics,
    }
}

fn safe_issue_warning(source_id: &str, kind: SourceErrorKind) -> String {
    let family = match source_id {
        "bitb" => "BitBabbler",
        "trng" => "TrueRNG",
        "rdseed" => "Intel RDSEED",
        "pseudo" => "PseudoRNG",
        _ => "A source family",
    };
    let problem = match kind {
        SourceErrorKind::PermissionDenied => "permission was denied",
        SourceErrorKind::DeviceBusy => "the interface is busy",
        SourceErrorKind::Disconnected => "the interface disconnected",
        SourceErrorKind::Timeout => "the request timed out",
        SourceErrorKind::DeviceNotFound => "no matching device was found",
        SourceErrorKind::NotAvailable => "the source is not available",
        SourceErrorKind::Protocol => "a protocol error occurred",
        SourceErrorKind::Unsupported => "the source is unsupported",
        SourceErrorKind::InvalidRequest => "the discovery request was invalid",
        SourceErrorKind::AllocationFailed => "memory allocation failed",
        SourceErrorKind::EntropyUnavailable => "OS entropy is unavailable",
        SourceErrorKind::MultipleDevices => "multiple devices need explicit selection",
        SourceErrorKind::SelectionRequired => "explicit selection is required",
        SourceErrorKind::Other => "discovery did not finish",
        _ => "discovery did not finish",
    };
    format!("{family} discovery reported a problem ({problem}). Other sources remain selectable.")
}

fn error_code_for(kind: SourceErrorKind) -> ErrorCode {
    match kind {
        SourceErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        SourceErrorKind::DeviceBusy => ErrorCode::SourceBusy,
        SourceErrorKind::Disconnected => ErrorCode::SourceDisconnected,
        SourceErrorKind::Timeout => ErrorCode::SourceTimedOut,
        SourceErrorKind::NotAvailable
        | SourceErrorKind::DeviceNotFound
        | SourceErrorKind::EntropyUnavailable => ErrorCode::SourceUnavailable,
        _ => ErrorCode::UnexpectedFailure,
    }
}

fn new_opaque_token() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(n);
    hasher.write_u128(nanos);
    format!("t{:016x}{n:08x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::{DiscoveryService, FakeDiscovery};
    use rngkit_core::{SourceErrorKind, SourceId};

    #[test]
    fn fake_discovery_keeps_devices_separate_without_selectors() {
        let outcome = FakeDiscovery::empty()
            .with_trng("fake-trng-1", "COM3")
            .with_trng("fake-trng-2", r"\\.\COM12")
            .with_bitb("fake-bitb-1", "White", "ABCDEF0123456789")
            .discover();

        assert_eq!(outcome.candidates.len(), 3);
        assert_eq!(outcome.candidates[0].view.token, "fake-trng-1");
        assert_eq!(outcome.candidates[0].view.ordinal, 1);
        assert_eq!(outcome.candidates[1].view.token, "fake-trng-2");
        assert_eq!(outcome.candidates[1].view.ordinal, 2);
        assert_eq!(outcome.candidates[2].view.family_label, "BitBabbler");
        assert_eq!(outcome.candidates[2].view.variant.as_deref(), Some("White"));
        assert!(outcome.candidates[2].view.requires_fold);

        let dump = serde_json::to_string(
            &outcome
                .candidates
                .iter()
                .map(|candidate| &candidate.view)
                .collect::<Vec<_>>(),
        )
        .expect("json");
        assert!(!dump.contains("COM3"), "{dump}");
        assert!(!dump.contains("COM12"), "{dump}");
        assert!(!dump.contains("ABCDEF0123456789"), "{dump}");
        assert!(!dump.contains("COM3"), "{dump}");
        assert!(!dump.to_ascii_lowercase().contains("serial"));
        for selector in FakeDiscovery::empty()
            .with_trng("fake-trng-1", "COM3")
            .with_trng("fake-trng-2", r"\\.\COM12")
            .with_bitb("fake-bitb-1", "White", "ABCDEF0123456789")
            .hidden_selectors()
        {
            assert!(!dump.contains(selector), "{dump}");
        }
    }

    #[test]
    fn partial_family_failure_does_not_hide_other_candidates() {
        let outcome = FakeDiscovery::empty()
            .with_pseudo("fake-pseudo-1")
            .with_issue(SourceId::bitb(), SourceErrorKind::PermissionDenied)
            .discover();

        assert_eq!(outcome.candidates.len(), 1);
        assert_eq!(outcome.candidates[0].view.source_id, "pseudo");
        assert_eq!(outcome.warnings.len(), 1);
        assert!(outcome.warnings[0].contains("BitBabbler"));
        assert!(outcome.warnings[0].contains("permission was denied"));
        assert!(!outcome.warnings[0].to_ascii_lowercase().contains("serial"));
        let dump = serde_json::to_string(&outcome.family_warning()).expect("json");
        assert!(!dump.contains("fake "), "{dump}");
    }

    #[test]
    fn empty_discovery_has_no_candidates_or_warnings() {
        let outcome = FakeDiscovery::empty().discover();
        assert!(outcome.candidates.is_empty());
        assert!(outcome.warnings.is_empty());
        assert!(outcome.family_warning().is_none());
    }

    #[test]
    fn entropy_unavailable_is_not_reported_as_a_timeout() {
        let outcome = FakeDiscovery::empty()
            .with_issue(SourceId::pseudo(), SourceErrorKind::EntropyUnavailable)
            .discover();

        assert_eq!(outcome.warnings.len(), 1);
        assert!(outcome.warnings[0].contains("OS entropy is unavailable"));
        assert!(!outcome.warnings[0].contains("timed out"));
        assert_eq!(
            outcome.diagnostics[0].0,
            crate::dto::ErrorCode::SourceUnavailable
        );
    }

    #[test]
    fn indistinguishable_bitb_variants_keep_separate_ordinals() {
        let outcome = FakeDiscovery::empty()
            .with_bitb("fake-bitb-white-1", "White", "serial-a")
            .with_bitb("fake-bitb-white-2", "White", "serial-b")
            .with_bitb("fake-bitb-black-1", "Black", "serial-c")
            .discover();
        assert_eq!(outcome.candidates[0].view.ordinal, 1);
        assert_eq!(outcome.candidates[1].view.ordinal, 2);
        assert_eq!(outcome.candidates[2].view.ordinal, 1);
        assert_eq!(outcome.candidates[2].view.variant.as_deref(), Some("Black"));
    }
}
