//! Development and test snapshots. Never used by production IPC.

use crate::dto::{
    AppStateDto, CollectionSnapshot, CollectionState, CombineInputRow, CombineResult,
    CombineSnapshot, ErrorCode, FileJobState, ReportPreview, ReportsSnapshot, SourceCandidateDto,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevScenario {
    Idle,
    Discovering,
    Ready,
    Collecting,
    Stopping,
    Completed,
    Failed,
    ReportsPreview,
    ReportsConflict,
    CombineCompatible,
    CombineIncompatible,
}

impl DevScenario {
    pub fn parse(id: &str) -> Result<Self, crate::errors::SafeError> {
        match id {
            "idle" => Ok(Self::Idle),
            "discovering" => Ok(Self::Discovering),
            "ready" => Ok(Self::Ready),
            "collecting" => Ok(Self::Collecting),
            "stopping" => Ok(Self::Stopping),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "reportsPreview" => Ok(Self::ReportsPreview),
            "reportsConflict" => Ok(Self::ReportsConflict),
            "combineCompatible" => Ok(Self::CombineCompatible),
            "combineIncompatible" => Ok(Self::CombineIncompatible),
            _ => Err(crate::errors::SafeError::invalid_configuration(
                "Unknown development scenario.",
            )),
        }
    }
}

#[must_use]
pub fn bitb_candidate() -> SourceCandidateDto {
    SourceCandidateDto {
        token: "mock-bitb-1".into(),
        family_label: "BitBabbler".into(),
        variant: Some("White".into()),
        ordinal: 1,
        requires_fold: true,
    }
}

#[must_use]
pub fn pseudo_candidate() -> SourceCandidateDto {
    SourceCandidateDto {
        token: "mock-pseudo-1".into(),
        family_label: "PseudoRNG".into(),
        variant: None,
        ordinal: 1,
        requires_fold: false,
    }
}

#[must_use]
pub fn scenario_snapshot(scenario: DevScenario) -> AppStateDto {
    match scenario {
        DevScenario::Idle => snapshot(
            idle_collection(),
            FileJobState::Idle,
            empty_reports(),
            empty_combine(),
        ),
        DevScenario::Discovering => snapshot(
            collection(CollectionState::Discovering, "Discovering sources"),
            FileJobState::Idle,
            empty_reports(),
            empty_combine(),
        ),
        DevScenario::Ready => snapshot(
            ready_collection(CollectionState::Ready, "Ready"),
            FileJobState::Idle,
            empty_reports(),
            empty_combine(),
        ),
        DevScenario::Collecting => snapshot(
            live_collection(CollectionState::Collecting, "Collecting"),
            FileJobState::Idle,
            empty_reports(),
            empty_combine(),
        ),
        DevScenario::Stopping => snapshot(
            live_collection(CollectionState::Stopping, "Stopping"),
            FileJobState::Idle,
            empty_reports(),
            empty_combine(),
        ),
        DevScenario::Completed => snapshot(
            live_collection(CollectionState::Completed, "Completed"),
            FileJobState::Idle,
            empty_reports(),
            empty_combine(),
        ),
        DevScenario::Failed => {
            let mut collection = live_collection(CollectionState::Failed, "Failed");
            collection.error_code = Some(ErrorCode::SourceUnavailable);
            collection.error_message = Some("The selected source became unavailable.".into());
            snapshot(
                collection,
                FileJobState::Idle,
                empty_reports(),
                empty_combine(),
            )
        }
        DevScenario::ReportsPreview => snapshot(
            idle_collection(),
            FileJobState::Idle,
            ReportsSnapshot {
                preview: Some(native_preview(false)),
            },
            empty_combine(),
        ),
        DevScenario::ReportsConflict => snapshot(
            idle_collection(),
            FileJobState::Idle,
            ReportsSnapshot {
                preview: Some(native_preview(true)),
            },
            empty_combine(),
        ),
        DevScenario::CombineCompatible => snapshot(
            idle_collection(),
            FileJobState::Idle,
            empty_reports(),
            CombineSnapshot {
                inputs: compatible_inputs(),
                compatible: true,
                incompatibility: None,
                result: Some(CombineResult {
                    stem: "20260822T120000_concat_bitb_s8_i1_f0".into(),
                    input_count: 2,
                    total_rows: 18,
                }),
            },
        ),
        DevScenario::CombineIncompatible => {
            let mut inputs = compatible_inputs();
            inputs[1].valid = false;
            inputs[1].error = Some("Timestamp range overlaps the previous input.".into());
            snapshot(
                idle_collection(),
                FileJobState::Idle,
                empty_reports(),
                CombineSnapshot {
                    inputs,
                    compatible: false,
                    incompatibility: Some(
                        "Overlapping timestamp ranges are rejected, including equal boundaries."
                            .into(),
                    ),
                    result: None,
                },
            )
        }
    }
}

fn snapshot(
    collection: CollectionSnapshot,
    file_job: FileJobState,
    reports: ReportsSnapshot,
    combine: CombineSnapshot,
) -> AppStateDto {
    AppStateDto {
        collection,
        file_job,
        reports,
        combine,
    }
}

fn empty_reports() -> ReportsSnapshot {
    ReportsSnapshot::empty()
}

fn empty_combine() -> CombineSnapshot {
    CombineSnapshot::empty()
}

fn idle_collection() -> CollectionSnapshot {
    collection(CollectionState::Idle, "Idle")
}

fn collection(state: CollectionState, status_label: &str) -> CollectionSnapshot {
    CollectionSnapshot {
        state,
        status_label: status_label.into(),
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
    }
}

fn ready_collection(state: CollectionState, status_label: &str) -> CollectionSnapshot {
    let mut snapshot = collection(state, status_label);
    snapshot.candidates = vec![bitb_candidate(), pseudo_candidate()];
    snapshot.selected_token = Some(bitb_candidate().token);
    snapshot.fold = Some(0);
    snapshot.output_root_label = Some("Chosen folder".into());
    snapshot
}

fn live_collection(state: CollectionState, status_label: &str) -> CollectionSnapshot {
    let mut snapshot = ready_collection(state, status_label);
    snapshot.sample_count = 12;
    snapshot.elapsed_label = "00:00:12".into();
    snapshot.ones_proportion_label = "0.5104".into();
    snapshot.cumulative_z_label = "+0.72".into();
    snapshot.session_stem = Some("20260822T101500_bitb_s8_i1_f0".into());
    snapshot.session_id = Some("s1".into());
    snapshot.last_event_sequence = 12;
    snapshot
}

fn native_preview(conflict: bool) -> ReportPreview {
    ReportPreview {
        kind_label: "Native session".into(),
        origin: "Collected session".into(),
        source: "BitBabbler".into(),
        sample_bits: 8,
        interval_seconds: 1,
        fold: Some(0),
        status: "Completed".into(),
        row_count: 12,
        warning: None,
        conflict,
    }
}

fn compatible_inputs() -> Vec<CombineInputRow> {
    vec![
        CombineInputRow {
            basename: "20260101T010000_bitb_s8_i1.csv".into(),
            source: "BitBabbler".into(),
            sample_bits: 8,
            interval_seconds: 1,
            fold: Some(0),
            first_timestamp: "2026-01-01T01:00:00Z".into(),
            last_timestamp: "2026-01-01T01:00:10Z".into(),
            rows: 10,
            valid: true,
            error: None,
        },
        CombineInputRow {
            basename: "20260101T020000_bitb_s8_i1.csv".into(),
            source: "BitBabbler".into(),
            sample_bits: 8,
            interval_seconds: 1,
            fold: Some(0),
            first_timestamp: "2026-01-01T02:00:00Z".into(),
            last_timestamp: "2026-01-01T02:00:08Z".into(),
            rows: 8,
            valid: true,
            error: None,
        },
    ]
}
