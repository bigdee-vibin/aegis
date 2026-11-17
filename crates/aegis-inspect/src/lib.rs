//! Inspection pipeline (stage 1): normalise each text part into views, run
//! every detector over every view, and fail closed on any detector failure.
//!
//! Spans in findings refer to the *view* in which the match occurred (original,
//! normalised, or a decoded variant), not necessarily the original bytes. That
//! is sufficient for blocking decisions; mapping back to original coordinates
//! for transformation is part of S2.4.

mod inspector;
mod normalise;

pub use inspector::{InspectionReport, Inspector};
pub use normalise::views;
