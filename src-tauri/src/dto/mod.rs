//! IPC DTOs. Keep these independent of `rngkit-*` library types.

mod error;
mod state;

pub use error::{DiagnosticRecord, ErrorCode, SafeError};
pub use state::{
    AppStateDto, CollectionSnapshot, CollectionState, CombineInputRow, CombineResult,
    CombineSnapshot, FileJobState, ReportPreview, ReportsSnapshot, SourceCandidateDto,
};
