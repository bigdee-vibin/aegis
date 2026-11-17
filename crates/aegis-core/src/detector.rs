use crate::Finding;

/// Why a detector could not complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureCode {
    /// Internal error.
    Internal,
    /// Exceeded its time budget.
    Timeout,
}

/// Result of one detector run over one text view.
#[derive(Debug, Clone, PartialEq)]
pub enum DetectorOutcome {
    /// Completed; zero or more findings.
    Ok(Vec<Finding>),
    /// Could not complete. Callers must treat this as an inspection failure,
    /// never as "nothing found".
    Failed(FailureCode),
}

/// A deterministic or model-backed detector (api contracts §3).
///
/// Findings returned from `scan` use `Span::part_index == 0`; the inspection
/// pipeline rewrites it to the real part index.
pub trait Detector: Send + Sync {
    /// Stable detector identifier.
    fn id(&self) -> &'static str;

    /// Scan one normalised text view.
    fn scan(&self, text: &str) -> DetectorOutcome;
}
