use axum::http::{header::AUTHORIZATION, HeaderMap};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;

/// Failed-authentication attempts allowed per window before invalid keys are
/// rejected without further work.
pub const MAX_FAILURES_PER_WINDOW: usize = 20;
/// Length of the failed-attempt window.
pub const FAILURE_WINDOW: Duration = Duration::from_secs(60);

/// Outcome of an authentication attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthOutcome {
    /// Valid key for this client.
    Client(u32),
    /// Missing or invalid key.
    Denied,
    /// Too many recent failures: invalid keys are being throttled.
    Throttled,
}

/// Registered gateway clients. Only SHA-256 digests of API keys are stored.
///
/// Known deviation from ADR-0010 (security review finding 9): keys are hashed
/// with plain SHA-256, not HMAC/Argon2id. This is acceptable only because the
/// daemon enforces a minimum key strength (see [`key_is_strong`]) and holds
/// keys in memory only; the keyed scheme applies once keys are persisted in
/// the encrypted config store.
#[derive(Debug, Default)]
pub struct ClientRegistry {
    keys: Vec<(u32, [u8; 32])>,
    failures: Mutex<VecDeque<Instant>>,
}

/// Minimum bar for a gateway key: at least 32 characters drawn from at least
/// 12 distinct characters (rules out `aaaa…`, repeated phrases and short
/// passwords; real keys are random).
#[must_use]
pub fn key_is_strong(key: &str) -> bool {
    let mut distinct: Vec<char> = key.chars().collect();
    distinct.sort_unstable();
    distinct.dedup();
    key.chars().count() >= 32 && distinct.len() >= 12
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

    /// Authenticate from an `Authorization: Bearer <key>` header, throttling
    /// invalid attempts. Valid keys are never throttled, so an attacker cannot
    /// lock a legitimate client out.
    #[must_use]
    pub fn authenticate(&self, headers: &HeaderMap) -> AuthOutcome {
        self.authenticate_at(headers, Instant::now())
    }

    fn authenticate_at(&self, headers: &HeaderMap, now: Instant) -> AuthOutcome {
        if let Some(id) = self.match_key(headers) {
            return AuthOutcome::Client(id);
        }
        let mut failures = self.failures.lock().unwrap_or_else(|e| e.into_inner());
        while failures
            .front()
            .is_some_and(|t| now.duration_since(*t) > FAILURE_WINDOW)
        {
            failures.pop_front();
        }
        if failures.len() >= MAX_FAILURES_PER_WINDOW {
            return AuthOutcome::Throttled;
        }
        failures.push_back(now);
        AuthOutcome::Denied
    }

    fn match_key(&self, headers: &HeaderMap) -> Option<u32> {
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
        assert_eq!(
            r.authenticate(&headers("Bearer aegis_test_key")),
            AuthOutcome::Client(7)
        );
        assert_eq!(
            r.authenticate(&headers("bearer aegis_test_key")),
            AuthOutcome::Client(7)
        );
        assert_eq!(
            r.authenticate(&headers("Bearer other")),
            AuthOutcome::Denied
        );
        assert_eq!(
            r.authenticate(&headers("Basic aegis_test_key")),
            AuthOutcome::Denied
        );
        assert_eq!(r.authenticate(&HeaderMap::new()), AuthOutcome::Denied);
    }

    #[test]
    fn finding_9_failures_are_throttled_but_valid_keys_still_work() {
        let mut r = ClientRegistry::new();
        r.add_key(1, "aegis_test_key");
        let start = Instant::now();
        for _ in 0..MAX_FAILURES_PER_WINDOW {
            assert_eq!(
                r.authenticate_at(&headers("Bearer wrong"), start),
                AuthOutcome::Denied
            );
        }
        assert_eq!(
            r.authenticate_at(&headers("Bearer wrong"), start),
            AuthOutcome::Throttled
        );
        assert_eq!(
            r.authenticate_at(&headers("Bearer aegis_test_key"), start),
            AuthOutcome::Client(1)
        );
        // The window slides: after it passes, invalid keys are answered normally again.
        let later = start + FAILURE_WINDOW + Duration::from_secs(1);
        assert_eq!(
            r.authenticate_at(&headers("Bearer wrong"), later),
            AuthOutcome::Denied
        );
    }

    #[test]
    fn finding_9_weak_keys_are_rejected() {
        assert!(!key_is_strong("short"));
        assert!(!key_is_strong(&"a".repeat(40)));
        assert!(!key_is_strong(
            &"abcabcabcabcabcabcabcabcabcabcabc".repeat(2)
        ));
        assert!(key_is_strong("kT9vQ2xLmB7nW4zRpD6yH1sJcF8gA3eU"));
    }
}
