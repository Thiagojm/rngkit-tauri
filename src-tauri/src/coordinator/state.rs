//! Authoritative collection and file-job state machine.

use std::collections::VecDeque;

use crate::dto::{
    AppStateDto, CollectionSnapshot, CollectionState, CombineSnapshot, DiagnosticRecord, ErrorCode,
    FileJobState, ReportsSnapshot, SourceCandidateDto,
};
use crate::errors::{SafeError, redact_detail};

use super::fixtures::{self, DevScenario};

const MAX_DIAGNOSTICS: usize = 32;

#[derive(Debug, Clone)]
struct CandidateEntry {
    view: SourceCandidateDto,
    generation: u64,
}

/// Rust-owned coordinator. Frontend snapshots are derived from this state.
#[derive(Debug)]
pub struct AppCoordinator {
    collection: CollectionState,
    file_job: FileJobState,
    discovery_generation: u64,
    candidates: Vec<CandidateEntry>,
    selected_token: Option<String>,
    family_warning: Option<String>,
    sample_bits: u32,
    interval_seconds: u32,
    fold: Option<u32>,
    output_root_label: Option<String>,
    sample_count: u64,
    elapsed_label: String,
    ones_proportion_label: String,
    cumulative_z_label: String,
    overrun_count: u64,
    session_stem: Option<String>,
    session_id: Option<String>,
    next_session_seq: u64,
    last_event_sequence: u64,
    next_event_sequence: u64,
    error_code: Option<ErrorCode>,
    error_message: Option<String>,
    reports: ReportsSnapshot,
    combine: CombineSnapshot,
    diagnostics: VecDeque<DiagnosticRecord>,
    next_operation_seq: u64,
}

impl Default for AppCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl AppCoordinator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            collection: CollectionState::Idle,
            file_job: FileJobState::Idle,
            discovery_generation: 0,
            candidates: Vec::new(),
            selected_token: None,
            family_warning: None,
            sample_bits: 8,
            interval_seconds: 1,
            fold: None,
            output_root_label: None,
            sample_count: 0,
            elapsed_label: "00:00:00".into(),
            ones_proportion_label: "—".into(),
            cumulative_z_label: "—".into(),
            overrun_count: 0,
            session_stem: None,
            session_id: None,
            next_session_seq: 0,
            last_event_sequence: 0,
            next_event_sequence: 0,
            error_code: None,
            error_message: None,
            reports: ReportsSnapshot::empty(),
            combine: CombineSnapshot::empty(),
            diagnostics: VecDeque::new(),
            next_operation_seq: 0,
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> AppStateDto {
        AppStateDto {
            collection: CollectionSnapshot {
                state: self.collection,
                status_label: status_label(self.collection).into(),
                candidates: self
                    .candidates
                    .iter()
                    .filter(|candidate| candidate.generation == self.discovery_generation)
                    .map(|candidate| candidate.view.clone())
                    .collect(),
                selected_token: self.selected_token.clone(),
                family_warning: self.family_warning.clone(),
                sample_bits: self.sample_bits,
                interval_seconds: self.interval_seconds,
                fold: self.fold,
                output_root_label: self.output_root_label.clone(),
                sample_count: self.sample_count,
                elapsed_label: self.elapsed_label.clone(),
                ones_proportion_label: self.ones_proportion_label.clone(),
                cumulative_z_label: self.cumulative_z_label.clone(),
                overrun_count: self.overrun_count,
                session_stem: self.session_stem.clone(),
                session_id: self.session_id.clone(),
                last_event_sequence: self.last_event_sequence,
                error_code: self.error_code,
                error_message: self.error_message.clone(),
            },
            file_job: self.file_job,
            reports: self.reports.clone(),
            combine: self.combine.clone(),
        }
    }

    #[must_use]
    pub fn diagnostics(&self) -> Vec<&DiagnosticRecord> {
        self.diagnostics.iter().collect()
    }

    #[must_use]
    pub fn discovery_generation(&self) -> u64 {
        self.discovery_generation
    }

    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    #[must_use]
    pub fn next_event_sequence(&self) -> u64 {
        self.next_event_sequence
    }

    /// Record a redacted diagnostic. Production copy-diagnostics IPC is later.
    pub fn record_diagnostic(&mut self, code: ErrorCode, raw_detail: &str) -> DiagnosticRecord {
        self.next_operation_seq += 1;
        let record = DiagnosticRecord {
            app_version: env!("CARGO_PKG_VERSION").into(),
            library_revision: crate::RNGKIT_CORE_REVISION.into(),
            operation_id: format!("op-{}", self.next_operation_seq),
            code,
            detail: redact_detail(raw_detail),
        };
        if self.diagnostics.len() >= MAX_DIAGNOSTICS {
            self.diagnostics.pop_front();
        }
        self.diagnostics.push_back(record.clone());
        record
    }

    /// Install a development/test snapshot. Not a legal production transition.
    pub fn load_dev_fixture(&mut self, scenario: DevScenario) {
        let snapshot = fixtures::scenario_snapshot(scenario);
        self.apply_snapshot(snapshot);
    }

    pub fn begin_discover(&mut self) -> Result<u64, SafeError> {
        if matches!(
            self.collection,
            CollectionState::Collecting | CollectionState::Stopping
        ) {
            return Err(SafeError::operation_conflict(
                "Source discovery cannot run while a session is collecting or stopping.",
            ));
        }
        if self.collection == CollectionState::Discovering {
            return Err(SafeError::invalid_transition(
                "Source discovery is still running.",
            ));
        }
        self.discovery_generation = self.discovery_generation.saturating_add(1);
        self.candidates.clear();
        self.selected_token = None;
        self.fold = None;
        self.family_warning = None;
        self.reset_session_view();
        self.collection = CollectionState::Discovering;
        Ok(self.discovery_generation)
    }

    pub fn complete_discover(
        &mut self,
        generation: u64,
        candidates: Vec<SourceCandidateDto>,
    ) -> Result<(), SafeError> {
        if self.collection != CollectionState::Discovering {
            return Err(SafeError::invalid_transition(
                "Discovery results are accepted only while discovering.",
            ));
        }
        if generation != self.discovery_generation {
            return Err(SafeError::invalid_transition(
                "Stale discovery results were ignored.",
            ));
        }
        self.candidates = candidates
            .into_iter()
            .map(|view| CandidateEntry { view, generation })
            .collect();
        self.selected_token = None;
        self.fold = None;
        self.collection = CollectionState::Idle;
        Ok(())
    }

    pub fn select_token(&mut self, token: &str) -> Result<(), SafeError> {
        self.ensure_configurable()?;
        if self.collection == CollectionState::Discovering {
            return Err(SafeError::invalid_transition(
                "Wait for source discovery to finish.",
            ));
        }
        let candidate = self
            .current_candidate(token)
            .ok_or_else(SafeError::expired_selection)?;
        let requires_fold = candidate.requires_fold;
        self.selected_token = Some(token.to_owned());
        if requires_fold {
            if self.fold.is_none() {
                self.fold = Some(0);
            }
        } else {
            self.fold = None;
        }
        self.sync_ready_state();
        Ok(())
    }

    pub fn set_output_root_label(&mut self, label: &str) -> Result<(), SafeError> {
        self.ensure_configurable()?;
        if looks_like_path(label) {
            return Err(SafeError::invalid_configuration(
                "The output folder label cannot include a filesystem path.",
            ));
        }
        if label.trim().is_empty() {
            self.output_root_label = None;
        } else {
            self.output_root_label = Some(label.to_owned());
        }
        self.sync_ready_state();
        Ok(())
    }

    pub fn set_sample_bits(&mut self, bits: u32) -> Result<(), SafeError> {
        self.ensure_configurable()?;
        if bits == 0 || bits % 8 != 0 {
            return Err(SafeError::invalid_configuration(
                "Sample size must be a positive multiple of 8 bits.",
            ));
        }
        self.sample_bits = bits;
        self.sync_ready_state();
        Ok(())
    }

    pub fn set_interval_seconds(&mut self, seconds: u32) -> Result<(), SafeError> {
        self.ensure_configurable()?;
        if seconds < 1 {
            return Err(SafeError::invalid_configuration(
                "Sample interval must be at least one second.",
            ));
        }
        self.interval_seconds = seconds;
        self.sync_ready_state();
        Ok(())
    }

    pub fn set_fold(&mut self, fold: Option<u32>) -> Result<(), SafeError> {
        self.ensure_configurable()?;
        if fold.is_some_and(|value| value > 4) {
            return Err(SafeError::invalid_configuration(
                "Fold must be between 0 and 4.",
            ));
        }
        let selected_requires_fold = self
            .selected_token
            .as_deref()
            .and_then(|token| self.current_candidate(token))
            .is_some_and(|candidate| candidate.requires_fold);
        if fold.is_some() && !selected_requires_fold {
            return Err(SafeError::invalid_configuration(
                "Fold is available only for a selected BitBabbler source.",
            ));
        }
        self.fold = fold;
        self.sync_ready_state();
        Ok(())
    }

    pub fn start(&mut self) -> Result<String, SafeError> {
        if self.file_job != FileJobState::Idle {
            return Err(SafeError::operation_conflict(
                "A file job is already running.",
            ));
        }
        if self.collection != CollectionState::Ready {
            return Err(SafeError::invalid_transition(start_reason(self.collection)));
        }
        self.next_session_seq += 1;
        let session_id = format!("s{}", self.next_session_seq);
        self.session_id = Some(session_id.clone());
        self.last_event_sequence = 0;
        self.next_event_sequence = 1;
        self.reset_live_metrics();
        self.error_code = None;
        self.error_message = None;
        self.session_stem = None;
        self.collection = CollectionState::Collecting;
        Ok(session_id)
    }

    pub fn request_stop(&mut self) -> Result<(), SafeError> {
        match self.collection {
            CollectionState::Collecting => {
                self.collection = CollectionState::Stopping;
                Ok(())
            }
            CollectionState::Stopping => Ok(()),
            _ => Err(SafeError::invalid_transition(
                "Stop is available only while collecting.",
            )),
        }
    }

    pub fn finish_completed(&mut self) -> Result<(), SafeError> {
        self.require_active_session()?;
        self.error_code = None;
        self.error_message = None;
        self.collection = CollectionState::Completed;
        Ok(())
    }

    pub fn finish_failed(&mut self, error: SafeError) -> Result<(), SafeError> {
        self.require_active_session()?;
        self.record_diagnostic(error.code, error.message());
        self.error_code = Some(error.code);
        self.error_message = Some(error.into_message());
        self.collection = CollectionState::Failed;
        Ok(())
    }

    pub fn start_another(&mut self) -> Result<(), SafeError> {
        if !matches!(
            self.collection,
            CollectionState::Completed | CollectionState::Failed
        ) {
            return Err(SafeError::invalid_transition(
                "Start another session is available after a session ends.",
            ));
        }
        self.reset_session_view();
        self.collection = CollectionState::Idle;
        self.sync_ready_state();
        Ok(())
    }

    pub fn begin_file_job(&mut self, job: FileJobState) -> Result<(), SafeError> {
        if job == FileJobState::Idle {
            return Err(SafeError::invalid_configuration(
                "A file job kind is required.",
            ));
        }
        if matches!(
            self.collection,
            CollectionState::Collecting | CollectionState::Stopping
        ) {
            return Err(SafeError::operation_conflict(
                "File jobs cannot run while a session is collecting or stopping.",
            ));
        }
        if self.file_job != FileJobState::Idle {
            return Err(SafeError::operation_conflict(
                "Only one file job can run at a time.",
            ));
        }
        self.file_job = job;
        Ok(())
    }

    pub fn finish_file_job(&mut self) -> Result<(), SafeError> {
        if self.file_job == FileJobState::Idle {
            return Err(SafeError::invalid_transition("No file job is running."));
        }
        self.file_job = FileJobState::Idle;
        Ok(())
    }

    fn apply_snapshot(&mut self, snapshot: AppStateDto) {
        let collection = snapshot.collection;
        self.collection = collection.state;
        self.file_job = snapshot.file_job;
        if collection.candidates.is_empty() {
            self.discovery_generation = match collection.state {
                CollectionState::Discovering => 1,
                _ => 0,
            };
            self.candidates.clear();
        } else {
            self.discovery_generation = 1;
            self.candidates = collection
                .candidates
                .into_iter()
                .map(|view| CandidateEntry {
                    generation: 1,
                    view,
                })
                .collect();
        }
        self.selected_token = collection.selected_token;
        self.family_warning = collection.family_warning;
        self.sample_bits = collection.sample_bits;
        self.interval_seconds = collection.interval_seconds;
        self.fold = collection.fold;
        self.output_root_label = collection.output_root_label;
        self.sample_count = collection.sample_count;
        self.elapsed_label = collection.elapsed_label;
        self.ones_proportion_label = collection.ones_proportion_label;
        self.cumulative_z_label = collection.cumulative_z_label;
        self.overrun_count = collection.overrun_count;
        self.session_stem = collection.session_stem;
        self.session_id = collection.session_id;
        self.last_event_sequence = collection.last_event_sequence;
        self.next_event_sequence = collection.last_event_sequence.saturating_add(1);
        self.next_session_seq = match self.session_id.as_deref() {
            Some(id) if id.starts_with('s') => id[1..].parse().unwrap_or(1),
            _ => 0,
        };
        self.error_code = collection.error_code;
        self.error_message = collection.error_message;
        self.reports = snapshot.reports;
        self.combine = snapshot.combine;
        self.diagnostics.clear();
    }

    fn ensure_configurable(&self) -> Result<(), SafeError> {
        if matches!(
            self.collection,
            CollectionState::Collecting | CollectionState::Stopping
        ) {
            return Err(SafeError::invalid_transition(
                "Session settings cannot change during collection.",
            ));
        }
        Ok(())
    }

    fn require_active_session(&self) -> Result<(), SafeError> {
        if matches!(
            self.collection,
            CollectionState::Collecting | CollectionState::Stopping
        ) {
            Ok(())
        } else {
            Err(SafeError::invalid_transition(
                "A session can finish only while collecting or stopping.",
            ))
        }
    }

    fn current_candidate(&self, token: &str) -> Option<&SourceCandidateDto> {
        self.candidates.iter().find_map(|candidate| {
            (candidate.generation == self.discovery_generation && candidate.view.token == token)
                .then_some(&candidate.view)
        })
    }

    fn draft_is_valid(&self) -> bool {
        let Some(token) = self.selected_token.as_deref() else {
            return false;
        };
        let Some(candidate) = self.current_candidate(token) else {
            return false;
        };
        if self.output_root_label.is_none() {
            return false;
        }
        if candidate.requires_fold != self.fold.is_some() {
            return false;
        }
        self.sample_bits > 0 && self.sample_bits % 8 == 0 && self.interval_seconds >= 1
    }

    fn sync_ready_state(&mut self) {
        if matches!(
            self.collection,
            CollectionState::Idle | CollectionState::Ready
        ) {
            self.collection = if self.draft_is_valid() {
                CollectionState::Ready
            } else {
                CollectionState::Idle
            };
        }
    }

    fn reset_session_view(&mut self) {
        self.session_id = None;
        self.last_event_sequence = 0;
        self.next_event_sequence = 0;
        self.reset_live_metrics();
        self.session_stem = None;
        self.error_code = None;
        self.error_message = None;
    }

    fn reset_live_metrics(&mut self) {
        self.sample_count = 0;
        self.elapsed_label = "00:00:00".into();
        self.ones_proportion_label = "—".into();
        self.cumulative_z_label = "—".into();
        self.overrun_count = 0;
    }
}

fn status_label(state: CollectionState) -> &'static str {
    match state {
        CollectionState::Idle => "Idle",
        CollectionState::Discovering => "Discovering sources",
        CollectionState::Ready => "Ready",
        CollectionState::Collecting => "Collecting",
        CollectionState::Stopping => "Stopping",
        CollectionState::Completed => "Completed",
        CollectionState::Failed => "Failed",
    }
}

fn start_reason(state: CollectionState) -> &'static str {
    match state {
        CollectionState::Idle => "Select a source and valid session settings before starting.",
        CollectionState::Discovering => "Wait for source discovery to finish.",
        CollectionState::Ready => "",
        CollectionState::Collecting => "A session is already collecting.",
        CollectionState::Stopping => "Wait for the current session to finish stopping.",
        CollectionState::Completed | CollectionState::Failed => {
            "Use Start another session from the summary."
        }
    }
}

fn looks_like_path(label: &str) -> bool {
    label.contains(":\\") || label.contains('/') || label.contains('\\') || label.starts_with("COM")
}

#[cfg(test)]
mod tests {
    use super::AppCoordinator;
    use crate::coordinator::fixtures::{DevScenario, bitb_candidate, pseudo_candidate};
    use crate::dto::{CollectionState, ErrorCode, FileJobState};
    use crate::errors::SafeError;

    fn ready() -> AppCoordinator {
        let mut coordinator = AppCoordinator::new();
        let generation = coordinator.begin_discover().expect("discover");
        coordinator
            .complete_discover(generation, vec![bitb_candidate(), pseudo_candidate()])
            .expect("results");
        coordinator
            .set_output_root_label("Chosen folder")
            .expect("folder");
        coordinator.select_token("mock-bitb-1").expect("select");
        assert_eq!(
            coordinator.snapshot().collection.state,
            CollectionState::Ready
        );
        coordinator
    }

    #[test]
    fn default_start_is_idle_without_candidates() {
        let snapshot = AppCoordinator::new().snapshot();
        assert_eq!(snapshot.collection.state, CollectionState::Idle);
        assert!(snapshot.collection.candidates.is_empty());
        assert_eq!(snapshot.file_job, FileJobState::Idle);
        assert!(snapshot.collection.session_id.is_none());
    }

    #[test]
    fn rejects_start_until_ready() {
        let mut coordinator = AppCoordinator::new();
        let error = coordinator.start().expect_err("idle start");
        assert_eq!(error.code, ErrorCode::InvalidTransition);
    }

    #[test]
    fn stop_is_idempotent_during_stopping() {
        let mut coordinator = ready();
        coordinator.start().expect("start");
        coordinator.request_stop().expect("stop");
        assert_eq!(
            coordinator.snapshot().collection.state,
            CollectionState::Stopping
        );
        coordinator.request_stop().expect("repeat stop");
        assert_eq!(
            coordinator.snapshot().collection.state,
            CollectionState::Stopping
        );
    }

    #[test]
    fn stale_discovery_generation_is_rejected() {
        let mut coordinator = AppCoordinator::new();
        let generation = coordinator.begin_discover().expect("discover");
        let error = coordinator
            .complete_discover(generation.saturating_sub(1), vec![pseudo_candidate()])
            .expect_err("stale");
        assert_eq!(error.code, ErrorCode::InvalidTransition);
        assert_eq!(
            coordinator.snapshot().collection.state,
            CollectionState::Discovering
        );
    }

    #[test]
    fn refresh_invalidates_previous_tokens() {
        let mut coordinator = ready();
        let generation = coordinator.begin_discover().expect("refresh");
        coordinator
            .complete_discover(generation, vec![pseudo_candidate()])
            .expect("results");
        let error = coordinator
            .select_token("mock-bitb-1")
            .expect_err("expired");
        assert_eq!(error.code, ErrorCode::ExpiredSelection);
    }

    #[test]
    fn diagnostics_are_redacted_and_bounded() {
        let mut coordinator = AppCoordinator::new();
        for index in 0..40 {
            coordinator.record_diagnostic(
                ErrorCode::UnexpectedFailure,
                &format!("failed C:\\Users\\dev\\rng.bin COM{index} seed={index}"),
            );
        }
        let diagnostics = coordinator.diagnostics();
        assert_eq!(diagnostics.len(), 32);
        for record in diagnostics {
            let dump = serde_json::to_string(record).expect("json");
            assert!(!dump.contains(":\\"));
            assert!(!dump.to_ascii_lowercase().contains("seed="));
            assert!(!dump.contains("COM"));
            assert!(record.detail.chars().count() <= 240);
        }
    }

    #[test]
    fn load_failed_fixture_stays_safe() {
        let mut coordinator = AppCoordinator::new();
        coordinator.load_dev_fixture(DevScenario::Failed);
        let dump = serde_json::to_string(&coordinator.snapshot()).expect("json");
        assert!(!dump.contains(":\\"));
        assert!(!dump.contains("/dev/"));
        assert!(!dump.to_ascii_lowercase().contains("entropy"));
        assert!(!dump.to_ascii_lowercase().contains("selector"));
        assert_eq!(
            coordinator.snapshot().collection.error_code,
            Some(ErrorCode::SourceUnavailable)
        );
        let _ = SafeError::source_unavailable();
    }
}
