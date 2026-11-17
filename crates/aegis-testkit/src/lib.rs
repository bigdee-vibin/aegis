//! Test helpers (story S0.3).
//!
//! - [`RequestCounter`] and [`find_canaries`]: counting and content-leak checks.
//! - [`MockServer`]: a loopback HTTP server that stands in for a cloud provider
//!   or Ollama, records exactly what it receives, and can misbehave on demand.
//! - [`ConnectionTripwire`]: a loopback listener standing in for "the
//!   internet"; any TCP connection to it is an outbound attempt.
//! - [`http`]: a tiny HTTP/1.1 client for tests (the workspace bans general
//!   HTTP client crates outside `aegis-egress`).
//!
//! # What this does and does not prove
//!
//! Pointing a provider at a mock or tripwire proves that *Aegis's own routing
//! and guard logic* sends nothing to a provider it should not. It cannot catch
//! a stray socket opened by unrelated code to an address nobody configured.
//! OS-level deny harnesses (Linux network namespace, Windows firewall rule,
//! macOS pf) need elevation and are tracked separately as S0.3b.

mod canary;
pub mod http;
mod server;

pub use canary::{find_canaries, sha256_hex};
pub use server::{Behaviour, ConnectionTripwire, MockServer, RecordedRequest};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Counts calls reaching a mock upstream. Acceptance tests assert it stays zero
/// when a request must not leave the machine (AT-01, AT-05, AT-07, AT-08).
#[derive(Debug, Clone, Default)]
pub struct RequestCounter(Arc<AtomicUsize>);

impl RequestCounter {
    /// New counter at zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one upstream call.
    pub fn record(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }

    /// Calls recorded so far.
    #[must_use]
    pub fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_counts_across_clones() {
        let c = RequestCounter::new();
        let c2 = c.clone();
        assert_eq!(c.count(), 0);
        c2.record();
        assert_eq!(c.count(), 1);
    }
}
