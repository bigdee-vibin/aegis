use sha2::{Digest, Sha256};

/// Returns the canary strings found in `haystack`. Used to prove prompts never
/// reach persistent storage or logs (AT-12, AT-17).
#[must_use]
pub fn find_canaries<'a>(haystack: &[u8], canaries: &[&'a str]) -> Vec<&'a str> {
    canaries
        .iter()
        .copied()
        .filter(|c| !c.is_empty() && haystack.windows(c.len()).any(|w| w == c.as_bytes()))
        .collect()
}

/// Lowercase hex SHA-256, for asserting hash-bound payloads (AT-09).
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canary_scan_finds_only_present_strings() {
        let data = b"row: CANARY-123 end";
        assert_eq!(
            find_canaries(data, &["CANARY-123", "CANARY-999", ""]),
            vec!["CANARY-123"]
        );
    }

    #[test]
    fn sha256_matches_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
