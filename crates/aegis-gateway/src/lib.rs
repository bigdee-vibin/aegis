//! Local OpenAI-compatible gateway (stories S1.2, S1.8).
//!
//! Pipeline for `POST /v1/chat/completions`:
//! authenticate → size limit → parse → validate → inspect → policy →
//! audit → (only if allowed) upstream. Every failure before the upstream call
//! ends the request; nothing is forwarded on an inspection failure.

mod auth;
mod error;
mod upstream;
mod validate;

pub use auth::ClientRegistry;
pub use error::GatewayError;
pub use upstream::{NoUpstream, Upstream, UpstreamError, UpstreamFuture};
pub use validate::{validate_chat, ValidationError};

use aegis_audit::{category_counts, max_severity, AuditEvent, AuditSink, Endpoint};
use aegis_core::{Action, ReasonCode};
use aegis_inspect::Inspector;
use aegis_policy::{decide, PolicyInput};
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Request limits.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Maximum request body size in bytes.
    pub max_body_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_body_bytes: 1024 * 1024,
        }
    }
}

/// Shared gateway state.
pub struct GatewayState {
    clients: ClientRegistry,
    inspector: Inspector,
    audit: Arc<dyn AuditSink>,
    upstream: Arc<dyn Upstream>,
    limits: Limits,
    counter: AtomicU64,
}

impl GatewayState {
    /// Assemble gateway state.
    #[must_use]
    pub fn new(
        clients: ClientRegistry,
        inspector: Inspector,
        audit: Arc<dyn AuditSink>,
        upstream: Arc<dyn Upstream>,
        limits: Limits,
    ) -> Self {
        Self {
            clients,
            inspector,
            audit,
            upstream,
            limits,
            counter: AtomicU64::new(0),
        }
    }

    fn next_request_id(&self) -> u64 {
        let ms = unix_ms();
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        (ms << 20) | (n & 0xF_FFFF)
    }
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Build the gateway router.
pub fn router(state: Arc<GatewayState>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state)
}

async fn healthz() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn chat_completions(
    State(state): State<Arc<GatewayState>>,
    headers: HeaderMap,
    body: Body,
) -> Response {
    handle_chat(&state, &headers, body)
        .await
        .unwrap_or_else(IntoResponse::into_response)
}

async fn handle_chat(
    state: &GatewayState,
    headers: &HeaderMap,
    body: Body,
) -> Result<Response, GatewayError> {
    let started = Instant::now();
    let client_id = state
        .clients
        .authenticate(headers)
        .ok_or_else(GatewayError::unauthorized)?;
    let request_id = state.next_request_id();

    let bytes = axum::body::to_bytes(body, state.limits.max_body_bytes)
        .await
        .map_err(|_| GatewayError::payload_too_large().with_request(request_id))?;
    let input_bytes = bytes.len() as u64;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| GatewayError::invalid("Body is not valid JSON.").with_request(request_id))?;

    let record = |action: Action,
                  reasons: &[ReasonCode],
                  findings: &[aegis_core::Finding],
                  forwarded: bool|
     -> Result<(), GatewayError> {
        let event = AuditEvent {
            ts_unix_ms: unix_ms(),
            request_id,
            client_id,
            endpoint: Endpoint::ChatCompletions,
            action,
            reasons: reasons.to_vec(),
            category_counts: category_counts(findings),
            max_severity: max_severity(findings),
            input_bytes,
            forwarded,
            latency_ms: u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX),
        };
        state.audit.record(&event).map_err(|_| {
            GatewayError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "aegis_error",
                "audit_failed",
                "Request not processed: the audit record could not be written.",
            )
            .with_request(request_id)
        })
    };

    let strings = match validate_chat(&value) {
        Ok(s) => s,
        Err(ValidationError::Invalid(msg)) => {
            return Err(GatewayError::invalid(msg).with_request(request_id));
        }
        Err(ValidationError::UnsupportedParam(param)) => {
            let reasons = [ReasonCode::UnsupportedParameter];
            record(Action::Block, &reasons, &[], false)?;
            return Err(GatewayError::new(
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                "unsupported_parameter",
                "This parameter is not supported by this Aegis build.",
            )
            .with_param(param)
            .with_request(request_id)
            .with_reasons(reasons.to_vec()));
        }
        Err(ValidationError::UnsupportedContent) => {
            let reasons = [ReasonCode::UnsupportedContent];
            record(Action::Block, &reasons, &[], false)?;
            return Err(GatewayError::new(
                StatusCode::BAD_REQUEST,
                "aegis_blocked",
                "unsupported_content",
                "Request contains content Aegis cannot inspect (text only in this version).",
            )
            .with_request(request_id)
            .with_reasons(reasons.to_vec()));
        }
    };

    let report = state.inspector.inspect(&strings);
    let decision = decide(&PolicyInput {
        findings: &report.findings,
        stage1_failed: report.stage1_failed,
    });

    match decision.action {
        Action::Allow | Action::Notify => {
            // Record before releasing: if the audit write fails, nothing leaves.
            record(decision.action, &decision.reasons, &report.findings, true)?;
        }
        Action::Review | Action::Block => {
            record(decision.action, &decision.reasons, &report.findings, false)?;
            let (status, kind, code, message) =
                if decision.reasons.contains(&ReasonCode::InspectionFailed) {
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "aegis_error",
                        "inspection_failed",
                        "Request blocked: inspection could not be completed.",
                    )
                } else if decision.reasons.contains(&ReasonCode::CriticalSecret) {
                    (
                        StatusCode::FORBIDDEN,
                        "aegis_blocked",
                        "critical_secret",
                        "Request blocked: a credential or secret was detected.",
                    )
                } else {
                    (
                        StatusCode::FORBIDDEN,
                        "aegis_blocked",
                        "review_required",
                        "Request held for review, but approval is not available in this build.",
                    )
                };
            return Err(GatewayError::new(status, kind, code, message)
                .with_request(request_id)
                .with_reasons(decision.reasons)
                .with_findings(report.findings));
        }
    }

    match state.upstream.complete(&value).await {
        Ok(response) => Ok((StatusCode::OK, Json(response)).into_response()),
        Err(UpstreamError::Unavailable) => Err(GatewayError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "aegis_error",
            "no_provider",
            "No model provider is available.",
        )
        .with_request(request_id)),
        Err(UpstreamError::Failed) => Err(GatewayError::new(
            StatusCode::BAD_GATEWAY,
            "aegis_error",
            "provider_error",
            "The model provider returned an error.",
        )
        .with_request(request_id)),
    }
}
