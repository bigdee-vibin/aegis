use aegis_core::{Finding, ReasonCode};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// An error returned to the client in the OpenAI-compatible envelope
/// (api contracts §2). Messages are static strings; matched text is never
/// included.
#[derive(Debug)]
pub struct GatewayError {
    pub(crate) status: StatusCode,
    pub(crate) kind: &'static str,
    pub(crate) code: &'static str,
    pub(crate) message: &'static str,
    pub(crate) param: Option<&'static str>,
    pub(crate) request_id: Option<u64>,
    pub(crate) reasons: Vec<ReasonCode>,
    pub(crate) findings: Vec<Finding>,
}

impl GatewayError {
    pub(crate) fn new(
        status: StatusCode,
        kind: &'static str,
        code: &'static str,
        message: &'static str,
    ) -> Self {
        Self {
            status,
            kind,
            code,
            message,
            param: None,
            request_id: None,
            reasons: Vec::new(),
            findings: Vec::new(),
        }
    }

    pub(crate) fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "invalid_request_error",
            "invalid_api_key",
            "Missing or invalid API key.",
        )
    }

    pub(crate) fn rate_limited() -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limit_error",
            "rate_limited",
            "Too many failed attempts. Try again later.",
        )
    }

    pub(crate) fn overloaded() -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "aegis_error",
            "overloaded",
            "Aegis is at its concurrency limit. Try again shortly.",
        )
    }

    pub(crate) fn payload_too_large() -> Self {
        Self::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "invalid_request_error",
            "payload_too_large",
            "Request body exceeds the configured limit.",
        )
    }

    pub(crate) fn invalid(message: &'static str) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "invalid_request_error",
            "invalid_request",
            message,
        )
    }

    pub(crate) fn with_request(mut self, id: u64) -> Self {
        self.request_id = Some(id);
        self
    }

    pub(crate) fn with_param(mut self, param: &'static str) -> Self {
        self.param = Some(param);
        self
    }

    pub(crate) fn with_reasons(mut self, reasons: Vec<ReasonCode>) -> Self {
        self.reasons = reasons;
        self
    }

    pub(crate) fn with_findings(mut self, findings: Vec<Finding>) -> Self {
        self.findings = findings;
        self
    }

    fn body(&self) -> Value {
        let mut counts: BTreeMap<(&'static str, String), u32> = BTreeMap::new();
        for f in &self.findings {
            *counts
                .entry((f.category.as_str(), f.subtype.clone()))
                .or_insert(0) += 1;
        }
        let findings: Vec<Value> = counts
            .into_iter()
            .map(|((category, subtype), count)| {
                json!({ "category": category, "subtype": subtype, "count": count })
            })
            .collect();
        let reasons: Vec<&str> = self.reasons.iter().map(|r| r.as_str()).collect();
        json!({
            "error": {
                "message": self.message,
                "type": self.kind,
                "code": self.code,
                "param": self.param,
                "aegis": {
                    "request_id": self.request_id.map(|id| format!("{id:016x}")),
                    "reasons": reasons,
                    "findings": findings,
                }
            }
        })
    }
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body())).into_response()
    }
}
