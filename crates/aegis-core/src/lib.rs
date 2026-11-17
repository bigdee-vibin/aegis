//! Canonical Aegis types shared by every crate (ADR-0004).
//!
//! This crate performs no I/O and has no dependency on policy, detectors,
//! providers or the network. It is the vocabulary of the system.

mod action;
mod finding;
mod mode;

pub use action::Action;
pub use finding::{Category, Finding, Severity, Span};
pub use mode::OperatingMode;
