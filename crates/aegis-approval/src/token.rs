use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Mutex;

type HmacSha256 = Hmac<Sha256>;

const DOMAIN: &[u8] = b"aegis-approved-outbound-v1";

/// Why a token was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenError {
    /// The tag does not verify: forged, altered, or minted under another key.
    BadTag,
    /// The bytes about to be sent are not the bytes that were approved.
    PayloadMismatch,
    /// The token was approved for a different provider.
    ProviderMismatch,
    /// The token has expired.
    Expired,
    /// The token was already used.
    Replayed,
}

/// Permission to send exactly one payload to exactly one provider, once,
/// before an expiry. Fields are private: the only way to obtain one is
/// [`Minter::mint`], and any modification invalidates the tag.
#[derive(Debug, Clone)]
pub struct ApprovedOutbound {
    request_id: u64,
    provider: String,
    payload_sha256: [u8; 32],
    expires_at_ms: u64,
    nonce: u64,
    tag: [u8; 32],
}

impl ApprovedOutbound {
    /// Request this approval belongs to.
    #[must_use]
    pub fn request_id(&self) -> u64 {
        self.request_id
    }

    /// Provider this approval is for.
    #[must_use]
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// SHA-256 of the approved payload.
    #[must_use]
    pub fn payload_sha256(&self) -> &[u8; 32] {
        &self.payload_sha256
    }
}

fn message(
    request_id: u64,
    provider: &str,
    payload_sha256: &[u8; 32],
    expires_at_ms: u64,
    nonce: u64,
) -> Vec<u8> {
    let mut m = Vec::with_capacity(DOMAIN.len() + provider.len() + 80);
    m.extend_from_slice(DOMAIN);
    m.extend_from_slice(&request_id.to_be_bytes());
    m.extend_from_slice(&(provider.len() as u64).to_be_bytes());
    m.extend_from_slice(provider.as_bytes());
    m.extend_from_slice(payload_sha256);
    m.extend_from_slice(&expires_at_ms.to_be_bytes());
    m.extend_from_slice(&nonce.to_be_bytes());
    m
}

fn tag(key: &[u8; 32], msg: &[u8]) -> Option<[u8; 32]> {
    let mut mac = HmacSha256::new_from_slice(key).ok()?;
    mac.update(msg);
    Some(mac.finalize().into_bytes().into())
}

/// Mints tokens. Held only by the approval engine.
pub struct Minter {
    key: [u8; 32],
    next_nonce: Mutex<u64>,
}

impl Minter {
    /// Create a minter from a 32-byte secret. The daemon supplies OS-random
    /// bytes at start-up; the key never leaves the process.
    #[must_use]
    pub fn new(key: [u8; 32]) -> Self {
        Self {
            key,
            next_nonce: Mutex::new(1),
        }
    }

    /// The matching verifier, for the egress guard.
    #[must_use]
    pub fn verifier(&self) -> Verifier {
        Verifier {
            key: self.key,
            used: Mutex::new(HashSet::new()),
        }
    }

    /// Mint a token for one payload and provider. `None` only if the HMAC
    /// primitive rejects the key (cannot happen for a 32-byte key); callers
    /// must treat `None` as "not approved".
    #[must_use]
    pub fn mint(
        &self,
        request_id: u64,
        provider: &str,
        payload_sha256: [u8; 32],
        expires_at_ms: u64,
    ) -> Option<ApprovedOutbound> {
        let nonce = {
            let mut n = self.next_nonce.lock().unwrap_or_else(|e| e.into_inner());
            let current = *n;
            *n += 1;
            current
        };
        let t = tag(
            &self.key,
            &message(request_id, provider, &payload_sha256, expires_at_ms, nonce),
        )?;
        Some(ApprovedOutbound {
            request_id,
            provider: provider.to_string(),
            payload_sha256,
            expires_at_ms,
            nonce,
            tag: t,
        })
    }
}

/// Verifies tokens at the point of sending. Tracks used nonces.
pub struct Verifier {
    key: [u8; 32],
    used: Mutex<HashSet<u64>>,
}

impl Verifier {
    /// Check a token against the exact bytes about to be sent. Consumes the
    /// nonce on success, so each token authorises one send.
    ///
    /// # Errors
    /// Returns the first failing [`TokenError`].
    pub fn verify(
        &self,
        token: &ApprovedOutbound,
        payload: &[u8],
        provider: &str,
        now_ms: u64,
    ) -> Result<(), TokenError> {
        let expected = tag(
            &self.key,
            &message(
                token.request_id,
                &token.provider,
                &token.payload_sha256,
                token.expires_at_ms,
                token.nonce,
            ),
        )
        .ok_or(TokenError::BadTag)?;
        let mut mac = HmacSha256::new_from_slice(&self.key).map_err(|_| TokenError::BadTag)?;
        mac.update(&message(
            token.request_id,
            &token.provider,
            &token.payload_sha256,
            token.expires_at_ms,
            token.nonce,
        ));
        // Constant-time comparison of the presented tag.
        mac.verify_slice(&token.tag)
            .map_err(|_| TokenError::BadTag)?;
        let _ = expected;
        if token.provider != provider {
            return Err(TokenError::ProviderMismatch);
        }
        let actual: [u8; 32] = Sha256::digest(payload).into();
        if actual != token.payload_sha256 {
            return Err(TokenError::PayloadMismatch);
        }
        if now_ms >= token.expires_at_ms {
            return Err(TokenError::Expired);
        }
        let mut used = self.used.lock().unwrap_or_else(|e| e.into_inner());
        if !used.insert(token.nonce) {
            return Err(TokenError::Replayed);
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn sha(bytes: &[u8]) -> [u8; 32] {
        Sha256::digest(bytes).into()
    }

    fn pair() -> (Minter, Verifier) {
        let m = Minter::new([7u8; 32]);
        let v = m.verifier();
        (m, v)
    }

    #[test]
    fn exact_payload_provider_and_time_are_accepted_once() {
        let (m, v) = pair();
        let t = m.mint(1, "openai", sha(b"hello"), 1_000).unwrap();
        assert_eq!(v.verify(&t, b"hello", "openai", 10), Ok(()));
        assert_eq!(
            v.verify(&t, b"hello", "openai", 10),
            Err(TokenError::Replayed)
        );
    }

    #[test]
    fn mutated_payload_is_refused_and_does_not_consume_the_token() {
        // AT-09: only the approved bytes reach the provider.
        let (m, v) = pair();
        let t = m.mint(1, "openai", sha(b"approved text"), 1_000).unwrap();
        assert_eq!(
            v.verify(&t, b"approved text + extra", "openai", 10),
            Err(TokenError::PayloadMismatch)
        );
        assert_eq!(v.verify(&t, b"approved text", "openai", 10), Ok(()));
    }

    #[test]
    fn wrong_provider_and_expiry_are_refused() {
        let (m, v) = pair();
        let t = m.mint(1, "openai", sha(b"x"), 100).unwrap();
        assert_eq!(
            v.verify(&t, b"x", "other", 10),
            Err(TokenError::ProviderMismatch)
        );
        assert_eq!(v.verify(&t, b"x", "openai", 100), Err(TokenError::Expired));
        assert_eq!(
            v.verify(&t, b"x", "openai", 5_000),
            Err(TokenError::Expired)
        );
    }

    #[test]
    fn token_from_another_key_is_forged() {
        let (_, v) = pair();
        let other = Minter::new([9u8; 32]);
        let forged = other.mint(1, "openai", sha(b"x"), 1_000).unwrap();
        assert_eq!(
            v.verify(&forged, b"x", "openai", 10),
            Err(TokenError::BadTag)
        );
    }

    #[test]
    fn tampering_with_a_field_breaks_the_tag() {
        let (m, v) = pair();
        let mut t = m.mint(1, "openai", sha(b"x"), 1_000).unwrap();
        t.payload_sha256 = sha(b"y");
        assert_eq!(v.verify(&t, b"y", "openai", 10), Err(TokenError::BadTag));
        let mut t = m.mint(1, "openai", sha(b"x"), 1_000).unwrap();
        t.expires_at_ms = u64::MAX;
        assert_eq!(v.verify(&t, b"x", "openai", 10), Err(TokenError::BadTag));
        let mut t = m.mint(1, "openai", sha(b"x"), 1_000).unwrap();
        t.provider = "evil".into();
        assert_eq!(v.verify(&t, b"x", "evil", 10), Err(TokenError::BadTag));
    }

    #[test]
    fn nonces_are_unique() {
        let (m, v) = pair();
        let a = m.mint(1, "p", sha(b"x"), 1_000).unwrap();
        let b = m.mint(1, "p", sha(b"x"), 1_000).unwrap();
        assert_ne!(a.nonce, b.nonce);
        assert_eq!(v.verify(&a, b"x", "p", 1), Ok(()));
        assert_eq!(v.verify(&b, b"x", "p", 1), Ok(()));
    }
}
