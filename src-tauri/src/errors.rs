//! Diagnostic redaction. Safe error types live in [`crate::dto`].

pub use crate::dto::{DiagnosticRecord, ErrorCode, SafeError};

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
    token.contains(":\\") || token.contains("\\\\")
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
    use super::redact_detail;

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
        let redacted = redact_detail("open failed path=/dev/ttyUSB0 port=COM3");
        assert_eq!(redacted, "open failed [redacted] [redacted]");
    }
}
