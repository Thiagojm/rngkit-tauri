//! Safe error types live in [`crate::dto`]. Diagnostic redaction lives in
//! [`crate::diagnostics`].

pub use crate::diagnostics::redact_detail;
pub use crate::dto::{DiagnosticRecord, ErrorCode, SafeError};
