//! Camel-case application DTOs. These are independent of library types.

use serde::{Deserialize, Serialize};

use super::{DiagnosticRecord, ErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CollectionState {
    Idle,
    Discovering,
    Ready,
    Collecting,
    Stopping,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileJobState {
    Idle,
    Inspecting,
    GeneratingReport,
    Combining,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCandidateDto {
    pub token: String,
    pub source_id: String,
    pub family_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    pub ordinal: u32,
    pub requires_fold: bool,
}

/// Sequenced collection event. Contains no entropy bytes or selectors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CollectionEventDto {
    SessionStarted {
        session_id: String,
        sequence: u64,
        stem: String,
    },
    SampleCommitted {
        session_id: String,
        sequence: u64,
        sample_index: u64,
        sample_count: u64,
        elapsed_label: String,
        ones_proportion_label: String,
        cumulative_z: f64,
        cumulative_z_label: String,
    },
    TimingOverrun {
        session_id: String,
        sequence: u64,
        overrun_count: u64,
    },
    CleanStop {
        session_id: String,
        sequence: u64,
        sample_count: u64,
        overrun_count: u64,
    },
    TerminalFailure {
        session_id: String,
        sequence: u64,
        code: ErrorCode,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        recovery: Option<String>,
    },
}

impl CollectionEventDto {
    #[must_use]
    pub fn session_id(&self) -> &str {
        match self {
            Self::SessionStarted { session_id, .. }
            | Self::SampleCommitted { session_id, .. }
            | Self::TimingOverrun { session_id, .. }
            | Self::CleanStop { session_id, .. }
            | Self::TerminalFailure { session_id, .. } => session_id,
        }
    }

    #[must_use]
    pub fn sequence(&self) -> u64 {
        match self {
            Self::SessionStarted { sequence, .. }
            | Self::SampleCommitted { sequence, .. }
            | Self::TimingOverrun { sequence, .. }
            | Self::CleanStop { sequence, .. }
            | Self::TerminalFailure { sequence, .. } => *sequence,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSnapshot {
    pub state: CollectionState,
    pub status_label: String,
    pub candidates: Vec<SourceCandidateDto>,
    pub selected_token: Option<String>,
    pub family_warning: Option<String>,
    pub sample_bits: u32,
    pub interval_seconds: u32,
    pub fold: Option<u32>,
    pub output_root_label: Option<String>,
    pub sample_count: u64,
    pub elapsed_label: String,
    pub ones_proportion_label: String,
    pub cumulative_z_label: String,
    pub overrun_count: u64,
    pub session_stem: Option<String>,
    pub session_id: Option<String>,
    pub last_event_sequence: u64,
    pub error_code: Option<ErrorCode>,
    pub error_message: Option<String>,
    pub error_recovery: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportPreview {
    pub kind_label: String,
    pub origin: String,
    pub source: String,
    pub sample_bits: u32,
    pub interval_seconds: u32,
    pub fold: Option<u32>,
    pub status: String,
    pub row_count: u64,
    pub warning: Option<String>,
    pub conflict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombineInputRow {
    pub basename: String,
    pub source: String,
    pub sample_bits: u32,
    pub interval_seconds: u32,
    pub fold: Option<u32>,
    pub first_timestamp: String,
    pub last_timestamp: String,
    pub rows: u64,
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombineResult {
    pub stem: String,
    pub input_count: u32,
    pub total_rows: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportsSnapshot {
    pub preview: Option<ReportPreview>,
    pub report_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombineSnapshot {
    pub inputs: Vec<CombineInputRow>,
    pub compatible: bool,
    pub incompatibility: Option<String>,
    pub result: Option<CombineResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStateDto {
    pub collection: CollectionSnapshot,
    pub file_job: FileJobState,
    pub reports: ReportsSnapshot,
    pub combine: CombineSnapshot,
    pub theme: ThemePreference,
    pub preferences_warning: Option<String>,
    pub diagnostics: Vec<DiagnosticRecord>,
}

impl ReportsSnapshot {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            preview: None,
            report_ready: false,
        }
    }
}

impl CombineSnapshot {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            inputs: Vec::new(),
            compatible: false,
            incompatibility: None,
            result: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AppStateDto, CollectionEventDto, CollectionSnapshot, CollectionState, CombineSnapshot,
        FileJobState, ReportsSnapshot, ThemePreference,
    };

    #[test]
    fn collection_event_json_is_tagged_camel_case_without_entropy() {
        let event = CollectionEventDto::SampleCommitted {
            session_id: "s1".into(),
            sequence: 2,
            sample_index: 1,
            sample_count: 1,
            elapsed_label: "00:00:01".into(),
            ones_proportion_label: "0.5000".into(),
            cumulative_z: 0.5,
            cumulative_z_label: "+0.50".into(),
        };
        let value = serde_json::to_value(&event).expect("json");
        assert_eq!(value["kind"], "sampleCommitted");
        assert_eq!(value["sessionId"], "s1");
        assert_eq!(value["sampleIndex"], 1);
        assert_eq!(value["cumulativeZ"], 0.5);
        assert_eq!(value["cumulativeZLabel"], "+0.50");
        let dump = value.to_string().to_ascii_lowercase();
        assert!(!dump.contains("entropy"));
        assert!(!dump.contains("seed"));
    }

    #[test]
    fn snapshot_json_uses_camel_case() {
        let dto = AppStateDto {
            collection: CollectionSnapshot {
                state: CollectionState::Idle,
                status_label: "Idle".into(),
                candidates: Vec::new(),
                selected_token: None,
                family_warning: None,
                sample_bits: 2048,
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
                last_event_sequence: 0,
                error_code: None,
                error_message: None,
                error_recovery: None,
            },
            file_job: FileJobState::GeneratingReport,
            reports: ReportsSnapshot::empty(),
            combine: CombineSnapshot::empty(),
            theme: ThemePreference::System,
            preferences_warning: None,
            diagnostics: Vec::new(),
        };
        let value = serde_json::to_value(&dto).expect("json");
        assert!(value.get("fileJob").is_some());
        assert!(value.get("file_job").is_none());
        assert_eq!(value["fileJob"], "generatingReport");
        assert_eq!(value["reports"]["reportReady"], false);
        assert_eq!(value["collection"]["statusLabel"], "Idle");
        assert!(value["collection"].get("selectedToken").is_some());
        assert!(value.get("diagnostics").is_some());
        assert!(value["diagnostics"].as_array().is_some_and(Vec::is_empty));
        assert!(value["collection"].get("lastEventSequence").is_some());
        assert_eq!(value["theme"], "system");
        assert!(value.get("preferencesWarning").is_some());
        assert!(value.get("selectedToken").is_none());
    }
}
