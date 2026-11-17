//! Test helpers (story S0.3). Grows with the acceptance suite.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Counts calls reaching a mock upstream. Acceptance tests assert it stays zero
/// when a request must not leave the machine (AT: #1, #4, #6, #7).
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

/// Returns the canary strings found in `haystack`. Used to prove prompts never
/// reach persistent storage or logs (acceptance #12, #13).
#[must_use]
pub fn find_canaries<'a>(haystack: &[u8], canaries: &[&'a str]) -> Vec<&'a str> {
    canaries
        .iter()
        .copied()
        .filter(|c| !c.is_empty() && haystack.windows(c.len()).any(|w| w == c.as_bytes()))
        .collect()
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

    #[test]
    fn canary_scan_finds_only_present_strings() {
        let data = b"row: CANARY-123 end";
        assert_eq!(
            find_canaries(data, &["CANARY-123", "CANARY-999"]),
            vec!["CANARY-123"]
        );
    }
}
