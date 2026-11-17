//! Inspection pipeline (stage 1): normalise each text part into views, run
//! every detector over every view, and fail closed on any detector failure or
//! truncated decoding.
//!
//! Spans in findings refer to the *view* in which the match occurred (original,
//! normalised, or a decoded variant), not necessarily the original bytes. That
//! is sufficient for blocking decisions; mapping back to original coordinates
//! for transformation is part of S2.4.

mod inspector;
mod normalise;

pub use inspector::{InspectionReport, Inspector};
pub use normalise::{views, ViewSet, MAX_DECODED_BYTES, MAX_DECODE_DEPTH, MAX_VIEWS};
