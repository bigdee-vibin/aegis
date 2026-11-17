//! Local OpenAI-compatible gateway (stories S1.2, S1.8).
//!
//! Pipeline for `POST /v1/chat/completions`:
//! authenticate (throttled) → concurrency limit → size limit and read timeout →
//! parse → validate → inspect (time-limited, fail closed) → policy → audit →
//! (only if allowed) upstream. Every failure before the upstream call ends the
//! request; nothing is forwarded on an inspection failure.

// `GatewayError` is large because it carries findings for the error body; it is
// created at most once per request, so boxing it would only add noise.
#![allow(clippy::result_large_err)]

mod auth;
mod error;
mod upstream;
mod validate;

pub use auth::{
    key_is_strong, AuthOutcome, ClientRegistry, FAILURE_WINDOW, MAX_FAILURES_PER_WINDOW,
};
pub use error::GatewayError;
pub use upstream::{NoUpstream, Upstream, UpstreamError, UpstreamFuture};
pub use validate::{validate_chat, ValidationError};

use aegis_audit::{category_counts, max_severity, AuditEvent, AuditSink, Endpoint};
use aegis_core::{Action, Finding, ReasonCode};
use aegis_inspect::{InspectionReport, Inspector};
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
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;

/// Request limits.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Maximum request body size in bytes.
    pub max_body_bytes: usize,
    /// Maximum time to receive the full body.
    pub body_timeout: Duration,
    /// Maximum time for inspection; exceeding it fails closed.
    pub inspect_timeout: Duration,
    /// Maximum requests processed concurrently.
    pub max_concurrent: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_body_bytes: 1024 * 1024,
            body_timeout: Duration::from_secs(15),
            inspect_timeout: Duration::from_secs(5),
            max_concurrent: 32,
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
    permits: Arc<Semaphore>,
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
            permits: Arc::new(Semaphore::new(limits.max_concurrent.max(1))),
            counter: AtomicU64::new(0),
        }
    }

    fn next_request_id(&self) -> u64 {
        let ms = unix_ms();
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        (ms << 20) | (n & 0xF_FFFF)
    }

    /// Write an audit event on a blocking thread (the sink does file I/O).
    async fn audit(&self, event: AuditEvent) -> Result<(), ()> {
        let sink = self.audit.clone();
        match tokio::task::spawn_blocking(move || sink.record(&event)).await {
            Ok(Ok(())) => Ok(()),
            _ => Err(()),
        }
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

struct Ctx<'a> {
    state: &'a GatewayState,
    request_id: u64,
    client_id: u32,
    started: Instant,
    input_bytes: u64,
}

impl Ctx<'_> {
    async fn record(
        &self,
        action: Action,
        reasons: &[ReasonCode],
        findings: &[Finding],
        forwarded: bool,
    ) -> Result<(), GatewayError> {
        let event = AuditEvent {
            ts_unix_ms: unix_ms(),
            request_id: self.request_id,
            client_id: self.client_id,
            endpoint: Endpoint::ChatCompletions,
            action,
            reasons: reasons.to_vec(),
            category_counts: category_counts(findings),
            max_severity: max_severity(findings),
            input_bytes: self.input_bytes,
            forwarded,
            latency_ms: u32::try_from(self.started.elapsed().as_millis()).unwrap_or(u32::MAX),
        };
        self.state.audit(event).await.map_err(|()| {
            GatewayError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "aegis_error",
                "audit_failed",
                "Request not processed: the audit record could not be written.",
            )
            .with_request(self.request_id)
        })
    }

    /// Audit a request rejected before inspection, then return its error. If
    /// the audit write itself fails the request is still rejected.
    async fn reject(&self, reason: ReasonCode, error: GatewayError) -> GatewayError {
        let _ = self.record(Action::Block, &[reason], &[], false).await;
        error
            .with_request(self.request_id)
            .with_reasons(vec![reason])
    }
}

async fn handle_chat(
    state: &GatewayState,
    headers: &HeaderMap,
    body: Body,
) -> Result<Response, GatewayError> {
    let started = Instant::now();
    let request_id = state.next_request_id();
    let mut ctx = Ctx {
        state,
        request_id,
        client_id: 0,
        started,
        input_bytes: 0,
    };

    match state.clients.authenticate(headers) {
        AuthOutcome::Client(id) => ctx.client_id = id,
        AuthOutcome::Denied => {
            return Err(ctx
                .reject(ReasonCode::AuthFailed, GatewayError::unauthorized())
                .await);
        }
        // Throttled attempts are neither processed nor audited: the audit log
        // must not become the next thing an attacker can flood.
        AuthOutcome::Throttled => return Err(GatewayError::rate_limited()),
    }

    let Ok(_permit) = state.permits.clone().try_acquire_owned() else {
        return Err(GatewayError::overloaded().with_request(request_id));
    };

    let read = tokio::time::timeout(
        state.limits.body_timeout,
        axum::body::to_bytes(body, state.limits.max_body_bytes),
    )
    .await;
    let bytes = match read {
        Ok(Ok(b)) => b,
        Ok(Err(_)) => {
            return Err(ctx
                .reject(
                    ReasonCode::PayloadTooLarge,
                    GatewayError::payload_too_large(),
                )
                .await);
        }
        Err(_) => {
            return Err(ctx
                .reject(
                    ReasonCode::InvalidRequest,
                    GatewayError::new(
                        StatusCode::REQUEST_TIMEOUT,
                        "invalid_request_error",
                        "request_timeout",
                        "The request body was not received in time.",
                    ),
                )
                .await);
        }
    };
    ctx.input_bytes = bytes.len() as u64;

    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
        return Err(ctx
            .reject(
                ReasonCode::InvalidRequest,
                GatewayError::invalid("Body is not valid JSON."),
            )
            .await);
    };

    let strings = match validate_chat(&value) {
        Ok(s) => s,
        Err(ValidationError::Invalid(msg)) => {
            return Err(ctx
                .reject(ReasonCode::InvalidRequest, GatewayError::invalid(msg))
                .await);
        }
        Err(ValidationError::UnsupportedParam(param)) => {
            return Err(ctx
                .reject(
                    ReasonCode::UnsupportedParameter,
                    GatewayError::new(
                        StatusCode::BAD_REQUEST,
                        "invalid_request_error",
                        "unsupported_parameter",
                        "This parameter is not supported by this Aegis build.",
                    )
                    .with_param(param),
                )
                .await);
        }
        Err(ValidationError::UnsupportedContent) => {
            return Err(ctx
                .reject(
                    ReasonCode::UnsupportedContent,
                    GatewayError::new(
                        StatusCode::BAD_REQUEST,
                        "aegis_blocked",
                        "unsupported_content",
                        "Request contains content Aegis cannot inspect (text only in this version).",
                    ),
                )
                .await);
        }
    };

    // Inspection is CPU-bound and must not stall the runtime, and must not be
    // able to run forever: a timeout or a panic is an inspection failure.
    let inspector = state.inspector.clone();
    let task = tokio::task::spawn_blocking(move || inspector.inspect(&strings));
    let report = match tokio::time::timeout(state.limits.inspect_timeout, task).await {
        Ok(Ok(report)) => report,
        Ok(Err(_)) | Err(_) => InspectionReport {
            findings: Vec::new(),
            stage1_failed: true,
        },
    };
    let decision = decide(&PolicyInput {
        findings: &report.findings,
        stage1_failed: report.stage1_failed,
    });

    match decision.action {
        Action::Allow | Action::Notify => {
            // Record before releasing: if the audit write fails, nothing leaves.
            ctx.record(decision.action, &decision.reasons, &report.findings, true)
                .await?;
        }
        Action::Review | Action::Block => {
            ctx.record(decision.action, &decision.reasons, &report.findings, false)
                .await?;
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
