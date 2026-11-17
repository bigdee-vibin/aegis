//! Metadata-only audit (story S1.6, ADR-0009).
//!
//! [`AuditEvent`] contains only enums, integers and fixed-size counters: there
//! is deliberately no string field, so prompt or response text cannot be
//! recorded even by mistake. The JSON-lines sink is an M1 stand-in; the
//! SQLCipher-backed store replaces it after spike S0.5, keeping this type.

use aegis_core::{Action, Category, Finding, ReasonCode, Severity};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

/// Which API surface handled the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Endpoint {
    /// `POST /v1/chat/completions`.
    ChatCompletions,
}

/// One audit record. No free text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unix time in milliseconds.
    pub ts_unix_ms: u64,
    /// Request identifier.
    pub request_id: u64,
    /// Authenticated client registry id.
    pub client_id: u32,
    /// API surface.
    pub endpoint: Endpoint,
    /// Policy action taken.
    pub action: Action,
    /// Reason codes.
    pub reasons: Vec<ReasonCode>,
    /// Finding counts per category, indexed by `Category::index`.
    pub category_counts: [u16; Category::COUNT],
    /// Highest finding severity, if any.
    pub max_severity: Option<Severity>,
    /// Request body size in bytes.
    pub input_bytes: u64,
    /// Whether the request was released toward an upstream provider.
    pub forwarded: bool,
    /// Time spent in Aegis before the decision, in milliseconds.
    pub latency_ms: u32,
}

/// Per-category finding counts, saturating at `u16::MAX`.
#[must_use]
pub fn category_counts(findings: &[Finding]) -> [u16; Category::COUNT] {
    let mut counts = [0u16; Category::COUNT];
    for f in findings {
        let slot = &mut counts[f.category.index()];
        *slot = slot.saturating_add(1);
    }
    counts
}

/// Highest severity among findings.
#[must_use]
pub fn max_severity(findings: &[Finding]) -> Option<Severity> {
    findings.iter().map(|f| f.severity).max()
}

/// Audit write failed. Carries no detail on purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditError;

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("audit write failed")
    }
}

impl std::error::Error for AuditError {}

/// Destination for audit events.
pub trait AuditSink: Send + Sync {
    /// Durably record an event. Failure must make the caller fail closed.
    ///
    /// # Errors
    /// Returns [`AuditError`] if the event could not be recorded.
    fn record(&self, event: &AuditEvent) -> Result<(), AuditError>;
}

/// In-memory sink for tests.
#[derive(Debug, Default)]
pub struct MemoryAuditSink {
    events: Mutex<Vec<AuditEvent>>,
}

impl MemoryAuditSink {
    /// Events recorded so far.
    #[must_use]
    pub fn snapshot(&self) -> Vec<AuditEvent> {
        self.events
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|e| e.into_inner().clone())
    }
}

impl AuditSink for MemoryAuditSink {
    fn record(&self, event: &AuditEvent) -> Result<(), AuditError> {
        let mut guard = self.events.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(event.clone());
        Ok(())
    }
}

/// Append-only JSON-lines file sink.
#[derive(Debug)]
pub struct JsonlFileSink {
    file: Mutex<File>,
}

impl JsonlFileSink {
    /// Open (creating if needed) the audit file for appending.
    ///
    /// # Errors
    /// Returns the I/O error if the file cannot be opened.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            file: Mutex::new(file),
        })
    }
}

impl AuditSink for JsonlFileSink {
    fn record(&self, event: &AuditEvent) -> Result<(), AuditError> {
        let mut line = serde_json::to_vec(event).map_err(|_| AuditError)?;
        line.push(b'\n');
        let mut file = self.file.lock().map_err(|_| AuditError)?;
        file.write_all(&line).map_err(|_| AuditError)?;
        file.flush().map_err(|_| AuditError)
    }
}

/// A sink that always fails; used to test fail-closed behaviour.
#[derive(Debug, Default)]
pub struct FailingAuditSink;

impl AuditSink for FailingAuditSink {
    fn record(&self, _event: &AuditEvent) -> Result<(), AuditError> {
        Err(AuditError)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use aegis_core::Span;

    fn event() -> AuditEvent {
        AuditEvent {
            ts_unix_ms: 1,
            request_id: 2,
            client_id: 3,
            endpoint: Endpoint::ChatCompletions,
            action: Action::Block,
            reasons: vec![ReasonCode::CriticalSecret],
            category_counts: [0, 1, 0, 0, 0, 0, 0, 0],
            max_severity: Some(Severity::Critical),
            input_bytes: 10,
            forwarded: false,
            latency_ms: 1,
        }
    }

    #[test]
    fn jsonl_round_trips() {
        let dir = std::env::temp_dir().join(format!("aegis-audit-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("audit.jsonl");
        let sink = JsonlFileSink::open(&path).unwrap();
        sink.record(&event()).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let parsed: AuditEvent = serde_json::from_str(text.trim()).unwrap();
        assert_eq!(parsed, event());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn counts_and_max_severity() {
        let f = |c, s| Finding {
            category: c,
            subtype: "x".into(),
            span: Span {
                part_index: 0,
                start: 0,
                end: 1,
            },
            severity: s,
            confidence: 1.0,
            detector: "d".into(),
            rule_version: "0".into(),
        };
        let findings = [
            f(Category::CredentialSecret, Severity::Critical),
            f(Category::CredentialSecret, Severity::Low),
            f(Category::Identity, Severity::Medium),
        ];
        let c = category_counts(&findings);
        assert_eq!(c[Category::CredentialSecret.index()], 2);
        assert_eq!(c[Category::Identity.index()], 1);
        assert_eq!(max_severity(&findings), Some(Severity::Critical));
        assert_eq!(max_severity(&[]), None);
    }

    #[test]
    fn failing_sink_fails() {
        assert!(FailingAuditSink.record(&event()).is_err());
    }
}
