//! Approval engine core (stories S2.3, S2.5, S2.6, S2.7; ADR-0006, ADR-0008).
//!
//! Two independent pieces:
//! - [`Review`]: a pure state machine. It never sees text, only SHA-256 hashes
//!   of candidate payloads, and never performs I/O: it returns [`Effect`]s the
//!   caller must carry out (re-inspect this payload, mint a token).
//! - [`Minter`] / [`Verifier`] / [`ApprovedOutbound`]: an unforgeable,
//!   hash-bound, single-use, expiring permission to send exactly one payload to
//!   one provider. Rust cannot restrict a constructor to one other crate, so
//!   the token carries an HMAC tag that only the holder of the [`Minter`] key
//!   can produce; the egress guard receives only a [`Verifier`].

mod review;
mod token;

pub use review::{
    Effect, Input, Payload, ReinspectOutcome, Review, ReviewConfig, ReviewError, State, Strength,
    TransmitContent,
};
pub use token::{ApprovedOutbound, Minter, TokenError, Verifier};
