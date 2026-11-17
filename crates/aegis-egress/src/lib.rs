//! Egress guard placeholder (ADR-0006).
//!
//! Story S2.6 implements the approved-outbound token and the HTTP client here.
//! Design refinement recorded for that story: Rust cannot restrict a
//! constructor to one other crate, so the token must be *unforgeable* rather
//! than merely hard to construct. The approval engine will hold a per-process
//! secret key and tag each token (request id, provider, payload SHA-256,
//! expiry, nonce); this crate receives only a verifier built from the same key
//! at daemon start-up and re-checks the tag and the payload hash at send time.
//!
//! Until then this crate deliberately contains no network code, and
//! `deny.toml` bans outbound HTTP clients everywhere else.
