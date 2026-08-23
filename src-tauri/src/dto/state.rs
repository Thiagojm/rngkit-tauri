//! Camel-case application DTOs. These are independent of library types.

use serde::{Deserialize, Serialize};

use super::ErrorCode;

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
}

impl ReportsSnapshot {
    #[must_use]
    pub fn empty() -> Self {
        Self { preview: None }
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
        AppStateDto, CollectionSnapshot, CollectionState, CombineSnapshot, FileJobState,
        ReportsSnapshot,
    };

    #[test]
    fn snapshot_json_uses_camel_case() {
        let dto = AppStateDto {
            collection: CollectionSnapshot {
                state: CollectionState::Idle,
                status_label: "Idle".into(),
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
                last_event_sequence: 0,
                error_code: None,
                error_message: None,
            },
            file_job: FileJobState::GeneratingReport,
            reports: ReportsSnapshot::empty(),
            combine: CombineSnapshot::empty(),
        };
        let value = serde_json::to_value(&dto).expect("json");
        assert!(value.get("fileJob").is_some());
        assert!(value.get("file_job").is_none());
        assert_eq!(value["fileJob"], "generatingReport");
        assert_eq!(value["collection"]["statusLabel"], "Idle");
        assert!(value["collection"].get("selectedToken").is_some());
        assert!(value["collection"].get("lastEventSequence").is_some());
    }
}
