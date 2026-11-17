use axum::http::{header::AUTHORIZATION, HeaderMap};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Registered gateway clients. Only SHA-256 digests of API keys are stored.
///
/// API keys are 256-bit random values, so a fast hash is adequate here; the
/// keyed/Argon2 scheme of ADR-0010 applies once keys move into the encrypted
/// config store.
#[derive(Debug, Clone, Default)]
pub struct ClientRegistry {
    keys: Vec<(u32, [u8; 32])>,
}

impl ClientRegistry {
    /// Empty registry (rejects everything).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a client key.
    pub fn add_key(&mut self, client_id: u32, raw_key: &str) {
        self.keys.push((client_id, Sha256::digest(raw_key).into()));
    }

    /// Whether any client is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Authenticate from an `Authorization: Bearer <key>` header.
    #[must_use]
    pub fn authenticate(&self, headers: &HeaderMap) -> Option<u32> {
        let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
        let (scheme, token) = value.split_once(' ')?;
        if !scheme.eq_ignore_ascii_case("bearer") {
            return None;
        }
        let digest: [u8; 32] = Sha256::digest(token.trim()).into();
        let mut found = None;
        for (id, stored) in &self.keys {
            // No early exit: every key is compared.
            if bool::from(digest.ct_eq(stored)) {
                found = Some(*id);
            }
        }
        found
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn headers(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(AUTHORIZATION, value.parse().unwrap());
        h
    }

    #[test]
    fn accepts_registered_key_only() {
        let mut r = ClientRegistry::new();
        r.add_key(7, "aegis_test_key");
        assert_eq!(r.authenticate(&headers("Bearer aegis_test_key")), Some(7));
        assert_eq!(r.authenticate(&headers("bearer aegis_test_key")), Some(7));
        assert_eq!(r.authenticate(&headers("Bearer other")), None);
        assert_eq!(r.authenticate(&headers("Basic aegis_test_key")), None);
        assert_eq!(r.authenticate(&HeaderMap::new()), None);
    }
}
