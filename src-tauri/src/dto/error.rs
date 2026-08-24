//! Serializable safe errors and redacted diagnostic records.

use serde::{Deserialize, Serialize};

/// Stable application error codes. Snake_case on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidConfiguration,
    InvalidTransition,
    ExpiredSelection,
    SourceUnavailable,
    SourceBusy,
    SourceDisconnected,
    SourceTimedOut,
    PermissionDenied,
    OutputExists,
    CorruptInput,
    UnsupportedInput,
    OperationConflict,
    UnexpectedFailure,
}

/// Frontend-safe error DTO. Never contains entropy, selectors, or error chains.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeError {
    pub code: ErrorCode,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recovery: Option<String>,
}

impl SafeError {
    fn new(code: ErrorCode, message: &'static str) -> Self {
        Self {
            code,
            message: message.into(),
            operation_id: None,
            recovery: None,
        }
    }

    #[must_use]
    pub fn invalid_configuration(message: &'static str) -> Self {
        Self::new(ErrorCode::InvalidConfiguration, message)
    }

    #[must_use]
    pub fn invalid_transition(message: &'static str) -> Self {
        Self::new(ErrorCode::InvalidTransition, message)
    }

    #[must_use]
    pub fn expired_selection() -> Self {
        Self::new(
            ErrorCode::ExpiredSelection,
            "That source is no longer valid. Refresh sources and select again.",
        )
        .with_recovery("Refresh sources and select again.")
    }

    #[must_use]
    pub fn operation_conflict(message: &'static str) -> Self {
        Self::new(ErrorCode::OperationConflict, message)
    }

    #[must_use]
    pub fn source_unavailable() -> Self {
        Self::new(
            ErrorCode::SourceUnavailable,
            "The selected source became unavailable.",
        )
        .with_recovery("Select another source and try again.")
    }

    #[must_use]
    pub fn source_busy() -> Self {
        Self::new(ErrorCode::SourceBusy, "The selected source is busy.")
            .with_recovery("Wait, then try again.")
    }

    #[must_use]
    pub fn source_disconnected() -> Self {
        Self::new(
            ErrorCode::SourceDisconnected,
            "The selected source disconnected.",
        )
        .with_recovery("Reconnect the device, refresh sources, and select it again.")
    }

    #[must_use]
    pub fn source_timed_out() -> Self {
        Self::new(ErrorCode::SourceTimedOut, "The selected source timed out.")
            .with_recovery("Refresh sources and try again.")
    }

    #[must_use]
    pub fn output_exists() -> Self {
        Self::new(
            ErrorCode::OutputExists,
            "A session with that name already exists.",
        )
    }

    #[must_use]
    pub fn report_exists() -> Self {
        Self::new(
            ErrorCode::OutputExists,
            "An XLSX file already exists for this input.",
        )
    }

    #[must_use]
    pub fn unexpected_failure() -> Self {
        Self::new(
            ErrorCode::UnexpectedFailure,
            "The operation failed unexpectedly.",
        )
        .with_recovery("Copy diagnostics if you need to report this.")
    }

    #[must_use]
    pub fn channel_lost() -> Self {
        Self::new(
            ErrorCode::UnexpectedFailure,
            "The live session connection was lost. The session was stopped.",
        )
        .with_recovery("Start another session if you want to collect again.")
    }

    #[must_use]
    pub fn permission_denied(message: &'static str) -> Self {
        Self::new(ErrorCode::PermissionDenied, message)
    }

    #[must_use]
    pub fn corrupt_input(message: &'static str) -> Self {
        Self::new(ErrorCode::CorruptInput, message)
    }

    #[must_use]
    pub fn unsupported_input(message: &'static str) -> Self {
        Self::new(ErrorCode::UnsupportedInput, message)
    }

    #[must_use]
    pub fn with_operation_id(mut self, sequence: u64) -> Self {
        self.operation_id = Some(format!("op-{sequence}"));
        self
    }

    #[must_use]
    pub fn with_recovery(mut self, recovery: &'static str) -> Self {
        self.recovery = Some(recovery.into());
        self
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn recovery(&self) -> Option<&str> {
        self.recovery.as_deref()
    }

    pub(crate) fn into_message(self) -> String {
        self.message
    }
}

impl std::fmt::Display for SafeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SafeError {}

/// Bounded, redacted diagnostic retained in process memory only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticRecord {
    pub app_version: String,
    pub library_revision: String,
    pub operation_id: String,
    pub code: ErrorCode,
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::{ErrorCode, SafeError};
    use serde_json::Value;

    #[test]
    fn safe_error_serializes_camel_case_without_chains() {
        let error = SafeError::source_unavailable()
            .with_operation_id(1)
            .with_recovery("Select another source and try again.");
        let value = serde_json::to_value(&error).expect("json");
        assert_eq!(value["code"], "source_unavailable");
        assert_eq!(value["operationId"], "op-1");
        assert!(value.get("source").is_none());
        assert!(value.get("causes").is_none());
        let dump = value.to_string().to_ascii_lowercase();
        assert!(!dump.contains("entropy"));
        assert!(!dump.contains("seed"));
    }

    #[test]
    fn unexpected_failure_uses_a_canonical_safe_message() {
        let value = serde_json::to_value(SafeError::unexpected_failure()).expect("json");
        assert_eq!(value["code"], "unexpected_failure");
        assert_eq!(value["message"], "The operation failed unexpectedly.");
    }

    #[test]
    fn error_codes_are_snake_case() {
        let value = serde_json::to_value(ErrorCode::OperationConflict).expect("json");
        assert_eq!(value, Value::String("operation_conflict".into()));
    }
}
