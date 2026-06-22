//! Shared crate-internal helpers. Not part of the public API.

/// Percent-encodes a single path segment, escaping every byte outside the
/// unreserved set (`A-Z a-z 0-9 - _ . ~`). Reused by any resource that
/// interpolates a caller-supplied value into a path segment.
pub(crate) fn encode_path_segment(segment: &str) -> String {
    let mut encoded = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_only_reserved_characters() {
        // A valid SWIFT code passes through unchanged.
        assert_eq!(encode_path_segment("BOFAUS3NLMA"), "BOFAUS3NLMA");
        // Reserved characters are escaped so the path stays well-formed.
        assert_eq!(encode_path_segment("a/b c"), "a%2Fb%20c");
    }
}
