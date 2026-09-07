//! Authoritative collection and file-job state machine.

use std::collections::VecDeque;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use rngkit_core::{Fold, IntervalSeconds, SampleBits};
use rngkit_sources::SourceConfig;
use rngkit_xlsx::ReportOptions;

use crate::diagnostics::redact_detail;
use crate::discovery::{DiscoveryOutcome, MappedCandidate};
use crate::dto::{
    AppStateDto, CollectionEventDto, CollectionSnapshot, CollectionState, CombineResult,
    CombineSnapshot, DiagnosticRecord, ErrorCode, FileJobState, OutcomeActionId, OutcomeNotice,
    OutcomeOperation, OutcomePathRow, OutcomeSeverity, ReportPreview, ReportsSnapshot,
    SourceCandidateDto, ThemePreference,
};
use crate::errors::SafeError;
use crate::preferences::{self, Preferences, SessionDraft};

use super::fixtures::{self, DevScenario};

const MAX_DIAGNOSTICS: usize = 32;

/// Backend-only start payload. Selectors stay off the DTO.
pub struct CollectionStart {
    pub session_id: String,
    pub output_root: PathBuf,
    pub sample_bits: SampleBits,
    pub interval: IntervalSeconds,
    pub source_config: SourceConfig,
}

impl fmt::Debug for CollectionStart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CollectionStart")
            .field("session_id", &self.session_id)
            .field("has_output_root", &true)
            .field("sample_bits", &self.sample_bits.get())
            .field("interval_seconds", &self.interval.get())
            .field("source_id", &source_id_of(&self.source_config))
            .finish_non_exhaustive()
    }
}

/// Backend-only inspected report kind. Paths stay off the DTO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportKind {
    Native,
    Legacy,
    Derived,
    Standalone,
    FlatLegacyConcatenation,
}

/// Internal collection update applied before the frontend event is sent.
pub enum CollectionUpdate {
    SessionStarted {
        stem: String,
        directory: PathBuf,
    },
    SampleCommitted {
        sample_index: u64,
        sample_count: u64,
        elapsed_label: String,
        ones_proportion_label: String,
        cumulative_z: f64,
        cumulative_z_label: String,
    },
    TimingOverrun,
    CleanStop {
        sample_count: u64,
        overrun_count: u64,
    },
    TerminalFailure {
        error: SafeError,
        diagnostic: String,
    },
}

struct CandidateEntry {
    view: SourceCandidateDto,
    generation: u64,
    source: Option<rngkit_sources::SourceCandidate>,
}

impl fmt::Debug for CandidateEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CandidateEntry")
            .field("token", &self.view.token)
            .field("source_id", &self.view.source_id)
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

/// Rust-owned coordinator. Frontend snapshots are derived from this state.
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
    preferred_fold: Option<u32>,
    output_root: Option<PathBuf>,
    output_root_label: Option<String>,
    theme: ThemePreference,
    preferences_warning: Option<String>,
    sample_count: u64,
    elapsed_label: String,
    ones_proportion_label: String,
    cumulative_z_label: String,
    overrun_count: u64,
    session_stem: Option<String>,
    session_directory: Option<PathBuf>,
    session_id: Option<String>,
    next_session_seq: u64,
    last_event_sequence: u64,
    next_event_sequence: u64,
    error_code: Option<ErrorCode>,
    error_message: Option<String>,
    error_recovery: Option<String>,
    reports: ReportsSnapshot,
    report_directory: Option<PathBuf>,
    report_dest: Option<PathBuf>,
    report_input: Option<PathBuf>,
    report_kind: Option<ReportKind>,
    report_options: Option<ReportOptions>,
    pending_outcome: Option<OutcomeNotice>,
    next_outcome_id: u64,
    combine: CombineSnapshot,
    combine_inputs: Vec<PathBuf>,
    combine_input_ids: Vec<String>,
    combine_last_directory: Option<PathBuf>,
    combine_directory: Option<PathBuf>,
    diagnostics: VecDeque<DiagnosticRecord>,
    next_operation_seq: u64,
    next_combine_input_seq: u64,
}

impl fmt::Debug for AppCoordinator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppCoordinator")
            .field("collection", &self.collection)
            .field("file_job", &self.file_job)
            .field("discovery_generation", &self.discovery_generation)
            .field("candidates", &self.candidates)
            .field("selected_token", &self.selected_token)
            .field("family_warning", &self.family_warning)
            .field("sample_bits", &self.sample_bits)
            .field("interval_seconds", &self.interval_seconds)
            .field("fold", &self.fold)
            .field("preferred_fold", &self.preferred_fold)
            .field("has_output_root", &self.output_root.is_some())
            .field("output_root_label", &self.output_root_label)
            .field("theme", &self.theme)
            .field("preferences_warning", &self.preferences_warning)
            .field("has_session_directory", &self.session_directory.is_some())
            .field("has_report_directory", &self.report_directory.is_some())
            .field("has_report_input", &self.report_input.is_some())
            .field("report_kind", &self.report_kind)
            .field(
                "pending_outcome_id",
                &self.pending_outcome.as_ref().map(|notice| notice.id),
            )
            .field("has_combine_inputs", &!self.combine_inputs.is_empty())
            .field("combine_input_count", &self.combine_inputs.len())
            .field(
                "has_combine_last_directory",
                &self.combine_last_directory.is_some(),
            )
            .field("has_combine_directory", &self.combine_directory.is_some())
            .finish_non_exhaustive()
    }
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
            sample_bits: 2048,
            interval_seconds: 1,
            fold: None,
            preferred_fold: None,
            output_root: None,
            output_root_label: None,
            theme: ThemePreference::System,
            preferences_warning: None,
            sample_count: 0,
            elapsed_label: "00:00:00".into(),
            ones_proportion_label: "—".into(),
            cumulative_z_label: "—".into(),
            overrun_count: 0,
            session_stem: None,
            session_directory: None,
            session_id: None,
            next_session_seq: 0,
            last_event_sequence: 0,
            next_event_sequence: 0,
            error_code: None,
            error_message: None,
            error_recovery: None,
            reports: ReportsSnapshot::empty(),
            report_directory: None,
            report_dest: None,
            report_input: None,
            report_kind: None,
            report_options: None,
            pending_outcome: None,
            next_outcome_id: 0,
            combine: CombineSnapshot::empty(),
            combine_inputs: Vec::new(),
            combine_input_ids: Vec::new(),
            combine_last_directory: None,
            combine_directory: None,
            diagnostics: VecDeque::new(),
            next_operation_seq: 0,
            next_combine_input_seq: 0,
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
                error_recovery: self.error_recovery.clone(),
            },
            file_job: self.file_job,
            reports: self.reports.clone(),
            combine: self.combine.clone(),
            theme: self.theme,
            preferences_warning: self.preferences_warning.clone(),
            diagnostics: self.diagnostics.iter().cloned().collect(),
            pending_outcome: self.pending_outcome.clone(),
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

    #[must_use]
    pub fn sample_bits(&self) -> u32 {
        self.sample_bits
    }

    #[must_use]
    pub fn interval_seconds(&self) -> u32 {
        self.interval_seconds
    }

    #[must_use]
    pub fn preferred_fold(&self) -> Option<u32> {
        self.preferred_fold
    }

    #[must_use]
    pub fn output_root(&self) -> Option<&Path> {
        self.output_root.as_deref()
    }

    #[must_use]
    pub fn session_directory(&self) -> Option<&Path> {
        self.session_directory.as_deref()
    }

    #[must_use]
    pub fn live_recording_directory(&self) -> Option<&Path> {
        if matches!(
            self.collection,
            CollectionState::Collecting | CollectionState::Stopping
        ) {
            self.session_directory.as_deref()
        } else {
            None
        }
    }

    #[must_use]
    pub fn report_directory(&self) -> Option<&Path> {
        self.report_directory.as_deref()
    }

    #[must_use]
    pub fn report_dest(&self) -> Option<&Path> {
        self.report_dest.as_deref()
    }

    #[must_use]
    pub fn report_input(&self) -> Option<&Path> {
        self.report_input.as_deref()
    }

    #[must_use]
    pub fn report_kind(&self) -> Option<ReportKind> {
        self.report_kind
    }

    #[must_use]
    pub fn report_options(&self) -> Option<&ReportOptions> {
        self.report_options.as_ref()
    }

    #[must_use]
    pub fn pending_outcome(&self) -> Option<&OutcomeNotice> {
        self.pending_outcome.as_ref()
    }

    pub fn acknowledge_outcome(&mut self, notice_id: u64) -> Result<(), SafeError> {
        match self.pending_outcome.as_ref().map(|notice| notice.id) {
            Some(current) if current == notice_id => {
                self.pending_outcome = None;
                Ok(())
            }
            Some(_) => Err(SafeError::invalid_transition(
                "That outcome is no longer current.",
            )),
            None => Err(SafeError::invalid_transition(
                "There is no pending outcome to acknowledge.",
            )),
        }
    }

    #[must_use]
    pub fn report_ready(&self) -> bool {
        self.reports.report_ready
    }

    pub fn set_inspected_report(
        &mut self,
        preview: ReportPreview,
        directory: PathBuf,
        dest: PathBuf,
        input: PathBuf,
        kind: ReportKind,
        options: ReportOptions,
    ) {
        self.reports.report_ready = preview.conflict;
        self.reports.preview = Some(preview);
        self.report_directory = Some(directory);
        self.report_dest = Some(dest);
        self.report_input = Some(input);
        self.report_kind = Some(kind);
        self.report_options = Some(options);
    }

    pub fn mark_report_written(&mut self, replaced: bool) {
        if let Some(preview) = self.reports.preview.as_mut() {
            preview.conflict = true;
        }
        self.reports.report_ready = true;
        self.note_report_success(replaced);
    }

    pub fn mark_report_conflict(&mut self) {
        if let Some(preview) = self.reports.preview.as_mut() {
            preview.conflict = true;
        }
    }

    pub fn note_report_failure(&mut self, message: &str) {
        let paths = self
            .report_dest
            .as_deref()
            .filter(|path| is_regular_file(path))
            .into_iter()
            .filter_map(|path| outcome_path("Existing XLSX report", path))
            .collect();
        self.replace_outcome(
            OutcomeSeverity::Error,
            OutcomeOperation::Report,
            "Report not completed",
            message.to_owned(),
            paths,
            Vec::new(),
        );
    }

    pub fn note_derived_created(&mut self, directory: &Path) {
        let stem = self
            .combine
            .result
            .as_ref()
            .map(|result| result.stem.as_str())
            .unwrap_or("derived");
        let mut paths = Vec::new();
        if let Some(path) = outcome_path("Derived folder", directory) {
            paths.push(path);
        }
        if let Some(path) = outcome_path("Derived CSV", &directory.join(format!("{stem}.csv"))) {
            paths.push(path);
        }
        if let Some(path) = outcome_path("Derived manifest", &directory.join("manifest.json")) {
            paths.push(path);
        }
        self.replace_outcome(
            OutcomeSeverity::Success,
            OutcomeOperation::Combine,
            "Derived bundle created",
            "The derived bundle was created successfully.".into(),
            paths,
            vec![OutcomeActionId::OpenDerivedFolder],
        );
    }

    pub fn note_combine_failure(&mut self, message: &str) {
        let paths = self
            .output_root
            .as_deref()
            .filter(|path| is_directory(path))
            .into_iter()
            .filter_map(|path| outcome_path("Combine output folder", path))
            .collect();
        self.replace_outcome(
            OutcomeSeverity::Error,
            OutcomeOperation::Combine,
            "Derived bundle not created",
            message.to_owned(),
            paths,
            Vec::new(),
        );
    }

    fn note_report_success(&mut self, replaced: bool) {
        let paths = self
            .report_dest
            .as_deref()
            .filter(|path| is_regular_file(path))
            .into_iter()
            .filter_map(|path| outcome_path("XLSX report", path))
            .collect();
        let (title, message) = if replaced {
            (
                "Report replaced",
                "The XLSX report was replaced successfully.",
            )
        } else {
            (
                "Report generated",
                "The XLSX report was generated successfully.",
            )
        };
        self.replace_outcome(
            OutcomeSeverity::Success,
            OutcomeOperation::Report,
            title,
            message.into(),
            paths,
            vec![
                OutcomeActionId::OpenReport,
                OutcomeActionId::OpenReportFolder,
            ],
        );
    }

    fn note_collection_success(&mut self) {
        let paths = self.collection_artifact_paths();
        let message = format!(
            "Collection completed with {} committed samples.",
            self.sample_count
        );
        self.replace_outcome(
            OutcomeSeverity::Success,
            OutcomeOperation::Collection,
            "Collection completed",
            message,
            paths,
            vec![OutcomeActionId::OpenSessionFolder],
        );
    }

    fn note_collection_failure(&mut self, error: &SafeError) {
        let mut message = error.message().to_owned();
        if let Some(recovery) = error.recovery() {
            message.push(' ');
            message.push_str(recovery);
        }
        let actions = self
            .session_directory
            .as_deref()
            .filter(|path| is_directory(path))
            .map(|_| vec![OutcomeActionId::OpenSessionFolder])
            .unwrap_or_default();
        self.replace_outcome(
            OutcomeSeverity::Error,
            OutcomeOperation::Collection,
            "Collection failed",
            message,
            self.collection_artifact_paths(),
            actions,
        );
    }

    fn collection_artifact_paths(&self) -> Vec<OutcomePathRow> {
        let Some(directory) = self.session_directory.as_deref() else {
            return Vec::new();
        };
        let mut paths = Vec::new();
        if let Some(path) = outcome_path("Session folder", directory) {
            paths.push(path);
        }
        if let Some(stem) = self.session_stem.as_deref() {
            for (label, extension) in [("Session CSV", "csv"), ("Session BIN", "bin")] {
                if let Some(path) =
                    outcome_path(label, &directory.join(format!("{stem}.{extension}")))
                {
                    paths.push(path);
                }
            }
            if let Some(path) = outcome_path("Session manifest", &directory.join("manifest.json")) {
                paths.push(path);
            }
        }
        paths
    }

    fn replace_outcome(
        &mut self,
        severity: OutcomeSeverity,
        operation: OutcomeOperation,
        title: impl Into<String>,
        message: String,
        paths: Vec<OutcomePathRow>,
        actions: Vec<OutcomeActionId>,
    ) {
        self.next_outcome_id = self.next_outcome_id.saturating_add(1);
        self.pending_outcome = Some(OutcomeNotice {
            id: self.next_outcome_id,
            severity,
            operation,
            title: title.into(),
            message,
            paths,
            actions,
        });
    }

    #[must_use]
    pub fn combine_inputs(&self) -> &[PathBuf] {
        &self.combine_inputs
    }

    #[must_use]
    pub fn combine_input_ids(&self) -> &[String] {
        &self.combine_input_ids
    }

    #[must_use]
    pub fn combine_last_directory(&self) -> Option<&Path> {
        self.combine_last_directory.as_deref()
    }

    pub fn remember_combine_directory(&mut self, directory: PathBuf) {
        self.combine_last_directory = Some(directory);
    }

    pub fn replace_combine_inputs(&mut self, paths: Vec<PathBuf>) {
        self.combine_inputs.clear();
        self.combine_input_ids.clear();
        self.add_combine_inputs(paths);
    }

    pub fn add_combine_inputs(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            self.next_combine_input_seq = self.next_combine_input_seq.saturating_add(1);
            let id = format!("combine-{}", self.next_combine_input_seq);
            self.combine_inputs.push(path);
            self.combine_input_ids.push(id);
        }
    }

    pub fn remove_combine_input(&mut self, input_id: &str) -> Result<(), SafeError> {
        let Some(index) = self.combine_input_ids.iter().position(|id| id == input_id) else {
            return Err(SafeError::invalid_configuration(
                "That Combine input is no longer selected.",
            ));
        };
        self.combine_input_ids.remove(index);
        self.combine_inputs.remove(index);
        self.combine_directory = None;
        self.combine.result = None;
        Ok(())
    }

    pub fn clear_combine_inputs(&mut self) {
        self.combine_inputs.clear();
        self.combine_input_ids.clear();
        self.combine = CombineSnapshot::empty();
        self.combine_directory = None;
    }

    #[must_use]
    pub fn combine_directory(&self) -> Option<&Path> {
        self.combine_directory.as_deref()
    }

    pub fn set_combine_preview(&mut self, snapshot: CombineSnapshot) {
        self.combine = snapshot;
        self.combine_directory = None;
    }

    pub fn set_combine_result(&mut self, result: CombineResult, directory: PathBuf) {
        self.combine.result = Some(result);
        self.combine.compatible = true;
        self.combine.incompatibility = None;
        self.combine_directory = Some(directory);
    }

    #[must_use]
    pub fn theme(&self) -> ThemePreference {
        self.theme
    }

    #[must_use]
    pub fn session_draft(&self) -> SessionDraft {
        SessionDraft {
            sample_bits: self.sample_bits,
            interval_seconds: self.interval_seconds,
            fold: self.preferred_fold,
            output_root: self.output_root.clone(),
            theme: self.theme,
        }
    }

    pub fn set_theme(&mut self, theme: ThemePreference) {
        self.theme = theme;
    }

    pub(crate) fn restore_session_draft(&mut self, draft: &SessionDraft) {
        self.sample_bits = draft.sample_bits;
        self.interval_seconds = draft.interval_seconds;
        self.preferred_fold = draft.fold;
        self.output_root = draft.output_root.clone();
        self.output_root_label = draft
            .output_root
            .as_deref()
            .map(preferences::output_root_label);
        self.theme = draft.theme;
        let selected_requires_fold = self
            .selected_token
            .as_deref()
            .and_then(|token| self.current_candidate(token))
            .is_some_and(|candidate| candidate.requires_fold);
        self.fold = selected_requires_fold.then(|| draft.fold.unwrap_or(0));
        self.sync_ready_state();
    }

    pub fn set_preferences_warning(&mut self, warning: Option<String>) {
        self.preferences_warning = warning;
    }

    /// Restore safe draft fields. Never restores a candidate token or family.
    pub fn apply_persisted_draft(&mut self, preferences: &Preferences) {
        self.sample_bits = preferences.sample_bits;
        self.interval_seconds = preferences.interval_seconds;
        self.preferred_fold = preferences.fold;
        self.theme = preferences.theme;
        self.selected_token = None;
        if let Some(path) = preferences.output_root.as_deref() {
            match preferences::validate_output_root(path) {
                Ok(valid) => {
                    self.output_root_label = Some(preferences::output_root_label(&valid));
                    self.output_root = Some(valid);
                }
                Err(_) => {
                    self.output_root = None;
                    self.output_root_label = None;
                }
            }
        } else {
            self.output_root = None;
            self.output_root_label = None;
        }
        self.sync_ready_state();
    }

    pub fn set_output_root(&mut self, path: &Path) -> Result<(), SafeError> {
        self.ensure_configurable()?;
        let valid = preferences::validate_output_root(path)?;
        self.output_root_label = Some(preferences::output_root_label(&valid));
        self.output_root = Some(valid);
        self.sync_ready_state();
        Ok(())
    }

    #[must_use]
    pub fn selected_library_source(&self) -> Option<&rngkit_sources::SourceCandidate> {
        let token = self.selected_token.as_deref()?;
        self.candidates.iter().find_map(|candidate| {
            (candidate.generation == self.discovery_generation && candidate.view.token == token)
                .then_some(candidate.source.as_ref())
                .flatten()
        })
    }

    pub fn refresh_with(
        &mut self,
        discovery: &dyn crate::discovery::DiscoveryService,
    ) -> Result<AppStateDto, SafeError> {
        let generation = self.begin_discover()?;
        let outcome = discovery.discover();
        self.apply_discovery(generation, outcome)?;
        Ok(self.snapshot())
    }

    /// Record a redacted diagnostic. Copy diagnostics formats this history.
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
        self.finish_discover(
            generation,
            candidates
                .into_iter()
                .map(|view| MappedCandidate { view, source: None })
                .collect(),
            None,
            &[],
        )
    }

    pub fn apply_discovery(
        &mut self,
        generation: u64,
        outcome: DiscoveryOutcome,
    ) -> Result<(), SafeError> {
        let warning = outcome.family_warning();
        self.finish_discover(
            generation,
            outcome.candidates,
            warning,
            &outcome.diagnostics,
        )
    }

    pub fn fail_discover(&mut self, generation: u64) -> Result<(), SafeError> {
        if generation != self.discovery_generation {
            return Ok(());
        }
        if self.collection != CollectionState::Discovering {
            return Ok(());
        }
        self.candidates.clear();
        self.selected_token = None;
        self.fold = None;
        self.family_warning =
            Some("Source discovery failed unexpectedly. Refresh to try again.".into());
        self.record_diagnostic(ErrorCode::UnexpectedFailure, "discovery task failed");
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
            self.fold = Some(self.preferred_fold.unwrap_or(0));
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
            self.output_root = None;
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
        if fold.is_some() {
            self.preferred_fold = fold;
        }
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
        self.error_recovery = None;
        self.session_stem = None;
        self.session_directory = None;
        self.collection = CollectionState::Collecting;
        Ok(session_id)
    }

    /// Reconstructs `SourceConfig` from the selected token. Hardware selectors
    /// stay backend-only and are never written to the DTO.
    pub fn selected_source_config(&self) -> Result<SourceConfig, SafeError> {
        let token = self
            .selected_token
            .as_deref()
            .ok_or_else(SafeError::expired_selection)?;
        let view = self
            .current_candidate(token)
            .ok_or_else(SafeError::expired_selection)?;
        match view.source_id.as_str() {
            "pseudo" => Ok(SourceConfig::Pseudo {
                max_range_samples: None,
            }),
            "rdseed" => Ok(SourceConfig::Rdseed {
                max_rdseed_attempts_per_word: None,
                max_range_samples: None,
            }),
            "bitb" => {
                let fold_value = self.fold.ok_or_else(|| {
                    SafeError::invalid_configuration("Fold is required for BitBabbler.")
                })?;
                let fold = u8::try_from(fold_value)
                    .ok()
                    .and_then(|value| Fold::new(value).ok())
                    .ok_or_else(|| {
                        SafeError::invalid_configuration("Fold must be between 0 and 4.")
                    })?;
                let serial = match self.selected_library_source() {
                    Some(rngkit_sources::SourceCandidate::Bitb { serial, .. }) => {
                        Some(serial.clone())
                    }
                    Some(_) => {
                        return Err(SafeError::invalid_configuration(
                            "The selected source does not match the BitBabbler family.",
                        ));
                    }
                    None => None,
                };
                Ok(SourceConfig::Bitb { fold, serial })
            }
            "trng" => {
                let path = match self.selected_library_source() {
                    Some(rngkit_sources::SourceCandidate::Trng { port_name }) => {
                        Some(port_name.clone())
                    }
                    Some(_) => {
                        return Err(SafeError::invalid_configuration(
                            "The selected source does not match the TrueRNG family.",
                        ));
                    }
                    None => None,
                };
                Ok(SourceConfig::Trng { path })
            }
            _ => Err(SafeError::expired_selection()),
        }
    }

    pub fn begin_collection(&mut self) -> Result<CollectionStart, SafeError> {
        let output_root = self.output_root.clone().ok_or_else(|| {
            SafeError::invalid_configuration("Choose an output folder before starting.")
        })?;
        let source_config = self.selected_source_config()?;
        let sample_bits = SampleBits::new(self.sample_bits).map_err(|_| {
            SafeError::invalid_configuration("Sample size must be a positive multiple of 8 bits.")
        })?;
        let interval = IntervalSeconds::new(self.interval_seconds).map_err(|_| {
            SafeError::invalid_configuration("Sample interval must be at least one second.")
        })?;
        let session_id = self.start()?;
        Ok(CollectionStart {
            session_id,
            output_root,
            sample_bits,
            interval,
            source_config,
        })
    }

    pub fn ingest_collection_update(
        &mut self,
        session_id: &str,
        update: CollectionUpdate,
    ) -> Result<CollectionEventDto, SafeError> {
        if self.session_id.as_deref() != Some(session_id) {
            return Err(SafeError::invalid_transition(
                "Stale collection events were ignored.",
            ));
        }
        if !matches!(
            self.collection,
            CollectionState::Collecting | CollectionState::Stopping
        ) {
            return Err(SafeError::invalid_transition(
                "Collection events are accepted only during an active session.",
            ));
        }
        let sequence = self.next_event_sequence;
        self.next_event_sequence = self.next_event_sequence.saturating_add(1);
        self.last_event_sequence = sequence;
        match update {
            CollectionUpdate::SessionStarted { stem, directory } => {
                self.session_stem = Some(stem.clone());
                self.session_directory = Some(directory);
                Ok(CollectionEventDto::SessionStarted {
                    session_id: session_id.to_owned(),
                    sequence,
                    stem,
                })
            }
            CollectionUpdate::SampleCommitted {
                sample_index,
                sample_count,
                elapsed_label,
                ones_proportion_label,
                cumulative_z,
                cumulative_z_label,
            } => {
                self.sample_count = sample_count;
                self.elapsed_label = elapsed_label.clone();
                self.ones_proportion_label = ones_proportion_label.clone();
                self.cumulative_z_label = cumulative_z_label.clone();
                Ok(CollectionEventDto::SampleCommitted {
                    session_id: session_id.to_owned(),
                    sequence,
                    sample_index,
                    sample_count,
                    elapsed_label,
                    ones_proportion_label,
                    cumulative_z,
                    cumulative_z_label,
                })
            }
            CollectionUpdate::TimingOverrun => {
                self.overrun_count = self.overrun_count.saturating_add(1);
                Ok(CollectionEventDto::TimingOverrun {
                    session_id: session_id.to_owned(),
                    sequence,
                    overrun_count: self.overrun_count,
                })
            }
            CollectionUpdate::CleanStop {
                sample_count,
                overrun_count,
            } => {
                self.sample_count = sample_count;
                self.overrun_count = overrun_count;
                self.finish_completed()?;
                Ok(CollectionEventDto::CleanStop {
                    session_id: session_id.to_owned(),
                    sequence,
                    sample_count,
                    overrun_count,
                })
            }
            CollectionUpdate::TerminalFailure { error, diagnostic } => {
                self.record_diagnostic(error.code, &diagnostic);
                let dto = CollectionEventDto::TerminalFailure {
                    session_id: session_id.to_owned(),
                    sequence,
                    code: error.code,
                    message: error.message().to_owned(),
                    recovery: error.recovery().map(str::to_owned),
                };
                self.note_collection_failure(&error);
                self.apply_failed_error(error);
                self.collection = CollectionState::Failed;
                Ok(dto)
            }
        }
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
        self.error_recovery = None;
        self.collection = CollectionState::Completed;
        self.note_collection_success();
        Ok(())
    }

    pub fn finish_failed(&mut self, error: SafeError) -> Result<(), SafeError> {
        self.require_active_session()?;
        self.record_diagnostic(error.code, error.message());
        self.note_collection_failure(&error);
        self.apply_failed_error(error);
        self.collection = CollectionState::Failed;
        Ok(())
    }

    /// Finalize a worker-side failure for the active session. A delivery failure
    /// after `CleanStop` may replace the provisional completed UI state.
    pub fn finish_worker_failure(
        &mut self,
        session_id: &str,
        error: SafeError,
    ) -> Result<(), SafeError> {
        if self.session_id.as_deref() != Some(session_id) {
            return Err(SafeError::invalid_transition(
                "Stale collection events were ignored.",
            ));
        }
        if self.collection == CollectionState::Failed {
            return Ok(());
        }
        if !matches!(
            self.collection,
            CollectionState::Collecting | CollectionState::Stopping | CollectionState::Completed
        ) {
            return Err(SafeError::invalid_transition(
                "A worker failure requires an active or just-completed session.",
            ));
        }
        self.record_diagnostic(error.code, error.message());
        self.note_collection_failure(&error);
        self.apply_failed_error(error);
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
                    source: None,
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
        self.error_recovery = collection.error_recovery;
        self.reports = snapshot.reports;
        self.report_directory = None;
        self.report_dest = None;
        self.report_input = None;
        self.report_kind = None;
        self.report_options = None;
        self.combine = snapshot.combine;
        self.combine_inputs.clear();
        self.combine_input_ids.clear();
        self.combine_last_directory = None;
        self.combine_directory = None;
        self.theme = snapshot.theme;
        self.preferences_warning = snapshot.preferences_warning;
        self.diagnostics = snapshot.diagnostics.into();
        self.next_outcome_id = snapshot
            .pending_outcome
            .as_ref()
            .map(|notice| notice.id)
            .unwrap_or(self.next_outcome_id);
        self.pending_outcome = snapshot.pending_outcome;
    }

    pub(crate) fn ensure_configurable(&self) -> Result<(), SafeError> {
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

    fn finish_discover(
        &mut self,
        generation: u64,
        candidates: Vec<MappedCandidate>,
        family_warning: Option<String>,
        diagnostics: &[(ErrorCode, String)],
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
            .map(|mapped| CandidateEntry {
                view: mapped.view,
                source: mapped.source,
                generation,
            })
            .collect();
        self.selected_token = None;
        self.fold = None;
        self.family_warning = family_warning;
        for (code, detail) in diagnostics {
            self.record_diagnostic(*code, detail);
        }
        self.collection = CollectionState::Idle;
        self.sync_ready_state();
        Ok(())
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
        self.session_directory = None;
        self.error_code = None;
        self.error_message = None;
        self.error_recovery = None;
    }

    fn apply_failed_error(&mut self, error: SafeError) {
        self.error_code = Some(error.code);
        self.error_recovery = error.recovery().map(str::to_owned);
        self.error_message = Some(error.into_message());
    }

    fn reset_live_metrics(&mut self) {
        self.sample_count = 0;
        self.elapsed_label = "00:00:00".into();
        self.ones_proportion_label = "—".into();
        self.cumulative_z_label = "—".into();
        self.overrun_count = 0;
    }
}

fn outcome_path(label: &str, path: &Path) -> Option<OutcomePathRow> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !path.is_absolute()
        || metadata.file_type().is_symlink()
        || (!metadata.file_type().is_file() && !metadata.file_type().is_dir())
    {
        return None;
    }
    Some(OutcomePathRow {
        label: label.to_owned(),
        path: display_path(path)?,
    })
}

fn display_path(path: &Path) -> Option<String> {
    let path = path.to_str()?;
    #[cfg(windows)]
    {
        if let Some(path) = path.strip_prefix(r"\\?\UNC\") {
            return Some(format!(r"\\{path}"));
        }
        if let Some(path) = path.strip_prefix(r"\\?\") {
            return Some(path.to_owned());
        }
    }
    Some(path.to_owned())
}

fn is_regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_file())
        .unwrap_or(false)
}

fn is_directory(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_dir())
        .unwrap_or(false)
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
            "Use Start another session in Session."
        }
    }
}

fn looks_like_path(label: &str) -> bool {
    label.contains(":\\") || label.contains('/') || label.contains('\\') || label.starts_with("COM")
}

fn source_id_of(config: &SourceConfig) -> &'static str {
    match config {
        SourceConfig::Bitb { .. } => "bitb",
        SourceConfig::Trng { .. } => "trng",
        SourceConfig::Rdseed { .. } => "rdseed",
        SourceConfig::Pseudo { .. } => "pseudo",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::AppCoordinator;
    #[cfg(windows)]
    use super::display_path;
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
        assert_eq!(snapshot.theme, crate::dto::ThemePreference::System);
        assert!(snapshot.preferences_warning.is_none());
        assert!(snapshot.collection.selected_token.is_none());
    }

    #[cfg(windows)]
    #[test]
    fn display_paths_hide_windows_extended_prefixes() {
        assert_eq!(
            display_path(std::path::Path::new(r"\\?\D:\Reports\report.xlsx")).as_deref(),
            Some(r"D:\Reports\report.xlsx")
        );
        assert_eq!(
            display_path(std::path::Path::new(r"\\?\UNC\server\share\report.xlsx")).as_deref(),
            Some(r"\\server\share\report.xlsx")
        );
        assert_eq!(
            display_path(std::path::Path::new(r"D:\Reports\report.xlsx")).as_deref(),
            Some(r"D:\Reports\report.xlsx")
        );
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

    #[test]
    fn persisted_draft_never_restores_a_source_selection() {
        let mut coordinator = ready();
        let dir = std::env::temp_dir().join(format!(
            "rngkit-coord-{}-{}",
            std::process::id(),
            coordinator.discovery_generation()
        ));
        std::fs::create_dir_all(&dir).expect("dir");
        coordinator.set_output_root(&dir).expect("root");
        coordinator.set_sample_bits(16).expect("bits");
        coordinator.set_fold(Some(2)).expect("fold");

        let mut restored = AppCoordinator::new();
        restored.apply_persisted_draft(&crate::preferences::Preferences {
            sample_bits: 16,
            interval_seconds: 1,
            fold: Some(2),
            output_root: Some(dir.clone()),
            theme: crate::dto::ThemePreference::Dark,
            window: None,
        });
        let snapshot = restored.snapshot();
        assert!(snapshot.collection.selected_token.is_none());
        assert!(snapshot.collection.candidates.is_empty());
        assert_eq!(snapshot.collection.state, CollectionState::Idle);
        assert_eq!(snapshot.collection.sample_bits, 16);
        assert_eq!(snapshot.theme, crate::dto::ThemePreference::Dark);
        assert_eq!(
            snapshot.collection.output_root_label.as_deref(),
            dir.file_name().and_then(|name| name.to_str())
        );
        let dump = format!("{restored:?}");
        let dir_str = dir.to_string_lossy();
        if dir_str.contains(":\\") {
            assert!(!dump.contains(&*dir_str), "{dump}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
