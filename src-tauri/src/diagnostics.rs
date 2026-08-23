//! Bounded in-memory diagnostics and copy-safe formatting.

use crate::dto::{DiagnosticRecord, ErrorCode};

const MAX_DETAIL_CHARS: usize = 240;

/// Replace path, port, hex, and forbidden-token fragments, then bound length.
#[must_use]
pub fn redact_detail(raw: &str) -> String {
    let mut parts = Vec::new();
    for token in raw.split_whitespace() {
        parts.push(if sensitive_token(token) {
            "[redacted]"
        } else {
            token
        });
    }
    let joined = parts.join(" ");
    truncate_chars(&joined, MAX_DETAIL_CHARS)
}

/// Explicitly sanitized copy text. Never includes entropy, seeds, selectors,
/// serials, device paths, or absolute legacy-input paths.
#[must_use]
pub fn format_copy(records: &[&DiagnosticRecord]) -> String {
    if records.is_empty() {
        return "RngKit diagnostics\nNo diagnostic records.".into();
    }
    let mut out = String::new();
    let first = records[0];
    out.push_str("RngKit ");
    out.push_str(&first.app_version);
    out.push('\n');
    out.push_str("library ");
    out.push_str(&first.library_revision);
    out.push('\n');
    for record in records {
        out.push('\n');
        out.push_str(&record.operation_id);
        out.push(' ');
        out.push_str(&code_slug(record.code));
        out.push('\n');
        out.push_str(&redact_detail(&record.detail));
        out.push('\n');
    }
    out
}

fn code_slug(code: ErrorCode) -> String {
    serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unexpected_failure".into())
}

fn sensitive_token(token: &str) -> bool {
    looks_like_windows_path(token)
        || looks_like_unix_path(token)
        || looks_like_com_port(token)
        || looks_like_hex_blob(token)
        || contains_forbidden_word(token)
}

fn looks_like_windows_path(token: &str) -> bool {
    let bytes = token.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
    {
        return true;
    }
    token.contains('\\')
}

fn looks_like_unix_path(token: &str) -> bool {
    token.contains('/') && token.len() > 1
}

fn looks_like_com_port(token: &str) -> bool {
    token
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|segment| {
            let upper = segment.to_ascii_uppercase();
            upper.starts_with("COM")
                && upper.len() > 3
                && upper.as_bytes()[3..].iter().all(u8::is_ascii_digit)
        })
}

fn looks_like_hex_blob(token: &str) -> bool {
    let stripped = token.trim_matches(|c: char| matches!(c, ':' | ',' | '.' | ';'));
    stripped.len() >= 16 && stripped.bytes().all(|b| b.is_ascii_hexdigit())
}

fn contains_forbidden_word(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    lower.contains("entropy")
        || lower.contains("seed")
        || lower.contains("serial")
        || lower.contains("selector")
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_owned();
    }
    input.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::{format_copy, redact_detail};
    use crate::dto::{DiagnosticRecord, ErrorCode};

    fn record(detail: &str) -> DiagnosticRecord {
        DiagnosticRecord {
            app_version: "0.1.0".into(),
            library_revision: crate::RNGKIT_CORE_REVISION.into(),
            operation_id: "op-1".into(),
            code: ErrorCode::UnexpectedFailure,
            detail: redact_detail(detail),
        }
    }

    #[test]
    fn redacts_paths_ports_seeds_and_bounds_length() {
        let raw = "open failed on C:\\Users\\dev\\device.bin COM3 /dev/ttyUSB0 seed=00ff \
                   serial=ABCDEF0123456789 entropy=00112233445566778899aabbccddeeff extra \
                   context that should be truncated once the redacted output grows past the \
                   two hundred forty character diagnostic limit because copied diagnostics \
                   stay bounded";
        let redacted = redact_detail(raw);
        assert!(!redacted.contains("C:\\"));
        assert!(!redacted.contains("/dev/"));
        assert!(!redacted.to_ascii_lowercase().contains("com3"));
        assert!(!redacted.to_ascii_lowercase().contains("seed"));
        assert!(!redacted.to_ascii_lowercase().contains("serial"));
        assert!(!redacted.to_ascii_lowercase().contains("entropy"));
        assert!(redacted.contains("[redacted]"));
        assert!(redacted.chars().count() <= 240);
    }

    #[test]
    fn keeps_safe_words() {
        assert_eq!(
            redact_detail("source became unavailable during read"),
            "source became unavailable during read"
        );
    }

    #[test]
    fn redacts_embedded_paths_and_ports() {
        let redacted = redact_detail(
            "open failed path=/dev/ttyUSB0 port=COM3 C:\\Users\\Jane Doe\\session.bin",
        );
        assert_eq!(
            redacted,
            "open failed [redacted] [redacted] [redacted] [redacted]"
        );
    }

    #[test]
    fn copy_text_never_contains_sensitive_fragments() {
        let samples = [
            "open failed C:\\Users\\dev\\rng.bin COM3 seed=00ff",
            "legacy D:/data/session.csv serial=ABCDEF0123456789",
            "device /dev/ttyUSB0 entropy=00112233445566778899aabbccddeeff",
            r"port \\.\COM12 selector=usb",
        ];
        for sample in samples {
            let rec = record(sample);
            let text = format_copy(&[&rec]);
            let lower = text.to_ascii_lowercase();
            assert!(text.contains("RngKit 0.1.0"));
            assert!(text.contains(crate::RNGKIT_CORE_REVISION));
            assert!(text.contains("op-1"));
            assert!(text.contains("unexpected_failure"));
            assert!(!text.contains(":\\"), "{text}");
            assert!(!text.contains("/dev/"), "{text}");
            assert!(!lower.contains("com3"), "{text}");
            assert!(!lower.contains("com12"), "{text}");
            assert!(!lower.contains("seed"), "{text}");
            assert!(!lower.contains("serial"), "{text}");
            assert!(!lower.contains("entropy"), "{text}");
            assert!(!lower.contains("selector"), "{text}");
        }
    }

    #[test]
    fn empty_copy_is_safe() {
        let text = format_copy(&[]);
        assert!(text.contains("No diagnostic records"));
        assert!(!text.to_ascii_lowercase().contains("entropy"));
    }
}
