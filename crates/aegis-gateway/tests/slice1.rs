//! Slice 1 end-to-end tests (story S1.7): a secret in a prompt is blocked
//! before any upstream call, and nothing sensitive reaches audit storage.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use aegis_audit::{AuditSink, FailingAuditSink, JsonlFileSink, MemoryAuditSink};
use aegis_core::{Action, Detector, DetectorOutcome, ReasonCode};
use aegis_detect_secrets::SecretScanner;
use aegis_gateway::{router, ClientRegistry, GatewayState, Limits, Upstream, UpstreamFuture};
use aegis_inspect::Inspector;
use aegis_testkit::{find_canaries, RequestCounter};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

const KEY: &str = "aegis_test_client_key";

struct CountingUpstream(RequestCounter);

impl Upstream for CountingUpstream {
    fn complete<'a>(&'a self, _request: &'a Value) -> UpstreamFuture<'a> {
        self.0.record();
        Box::pin(async { Ok(json!({"id": "mock", "object": "chat.completion", "choices": []})) })
    }
}

struct PanicDetector;
impl Detector for PanicDetector {
    fn id(&self) -> &'static str {
        "panic"
    }
    fn scan(&self, _: &str) -> DetectorOutcome {
        panic!("injected detector panic");
    }
}

struct Harness {
    app: axum::Router,
    upstream_calls: RequestCounter,
    audit: Arc<MemoryAuditSink>,
}

fn fake_openai_key() -> String {
    format!("sk-proj-{}", "aB3dE6gH9jK2mN5pQ8sT1vW4".repeat(2))
}

fn harness_with(
    detectors: Vec<Arc<dyn Detector>>,
    audit: Arc<dyn AuditSink>,
    memory: Arc<MemoryAuditSink>,
    limits: Limits,
) -> Harness {
    let upstream_calls = RequestCounter::new();
    let mut clients = ClientRegistry::new();
    clients.add_key(1, KEY);
    let state = GatewayState::new(
        clients,
        Inspector::new(detectors),
        audit,
        Arc::new(CountingUpstream(upstream_calls.clone())),
        limits,
    );
    Harness {
        app: router(Arc::new(state)),
        upstream_calls,
        audit: memory,
    }
}

fn harness() -> Harness {
    let memory = Arc::new(MemoryAuditSink::default());
    harness_with(
        vec![Arc::new(SecretScanner::new().unwrap())],
        memory.clone(),
        memory,
        Limits::default(),
    )
}

async fn post_raw(app: &axum::Router, key: Option<&str>, body: Vec<u8>) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method("POST")
        .uri("/v1/chat/completions")
        .header("content-type", "application/json");
    if let Some(k) = key {
        req = req.header("authorization", format!("Bearer {k}"));
    }
    let resp = app
        .clone()
        .oneshot(req.body(Body::from(body)).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn post(h: &Harness, body: Value) -> (StatusCode, Value) {
    post_raw(&h.app, Some(KEY), serde_json::to_vec(&body).unwrap()).await
}

fn chat(content: &str) -> Value {
    json!({"model": "any", "messages": [{"role": "user", "content": content}]})
}

fn code(body: &Value) -> &str {
    body["error"]["code"].as_str().unwrap_or("")
}

#[tokio::test]
async fn api_key_in_prompt_is_blocked_before_upstream() {
    let h = harness();
    let key = fake_openai_key();
    let (status, body) = post(&h, chat(&format!("Debug this, my key is {key}"))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(code(&body), "critical_secret");
    assert_eq!(
        h.upstream_calls.count(),
        0,
        "nothing may reach the upstream"
    );
    assert!(
        !body.to_string().contains(&key),
        "response must not echo the secret"
    );
    let events = h.audit.snapshot();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, Action::Block);
    assert!(!events[0].forwarded);
    assert_eq!(events[0].reasons, vec![ReasonCode::CriticalSecret]);
}

#[tokio::test]
async fn clean_prompt_is_forwarded_and_audited() {
    let h = harness();
    let (status, body) = post(&h, chat("What is the capital of France?")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], "mock");
    assert_eq!(h.upstream_calls.count(), 1);
    let events = h.audit.snapshot();
    assert_eq!(events[0].action, Action::Allow);
    assert!(events[0].forwarded);
}

#[tokio::test]
async fn secrets_are_found_wherever_they_hide() {
    use base64::Engine;
    let key = fake_openai_key();
    let b64 = base64::engine::general_purpose::STANDARD.encode(&key);
    let zero_width = key.replacen("sk-", "sk-\u{200B}", 1);
    let fullwidth = key.replacen("sk-", "ｓｋ－", 1);
    let bodies = vec![
        // system and tool messages
        json!({"messages":[{"role":"system","content":key},{"role":"user","content":"hi"}]}),
        json!({"messages":[{"role":"user","content":"hi"},{"role":"tool","tool_call_id":"c1","content":key}]}),
        // text content parts
        json!({"messages":[{"role":"user","content":[{"type":"text","text":key}]}]}),
        // tool call arguments
        json!({"messages":[{"role":"assistant","tool_calls":[{"id":"c1","type":"function","function":{"name":"f","arguments":format!("{{\"k\":\"{key}\"}}")}}]},{"role":"user","content":"hi"}]}),
        // unknown top-level field
        json!({"messages":[{"role":"user","content":"hi"}],"metadata":{"note":key}}),
        // obfuscated
        chat(&format!("data: {b64}")),
        chat(&zero_width),
        chat(&fullwidth),
        chat(&format!("a%20b {}", key.replace('-', "%2D"))),
    ];
    for (i, body) in bodies.into_iter().enumerate() {
        let h = harness();
        let (status, resp) = post(&h, body).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "case {i} was not blocked: {resp}"
        );
        assert_eq!(
            code(&resp),
            "critical_secret",
            "case {i} blocked for the wrong reason: {resp}"
        );
        assert_eq!(
            resp["error"]["aegis"]["reasons"][0], "critical_secret",
            "case {i} reason code missing"
        );
        assert_eq!(h.upstream_calls.count(), 0, "case {i} leaked upstream");
    }
}

#[tokio::test]
async fn private_key_and_password_are_blocked() {
    for text in [
        "-----BEGIN OPENSSH PRIVATE KEY-----\nabc",
        "db password = Tr0ub4dor&3xyz",
        "postgres://admin:s3cretPass@db.internal/app",
    ] {
        let h = harness();
        let (status, _) = post(&h, chat(text)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "not blocked: {text}");
        assert_eq!(h.upstream_calls.count(), 0);
    }
}

#[tokio::test]
async fn authentication_is_required() {
    let h = harness();
    let body = serde_json::to_vec(&chat("hello")).unwrap();
    for key in [None, Some("wrong-key")] {
        let (status, resp) = post_raw(&h.app, key, body.clone()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(code(&resp), "invalid_api_key");
    }
    assert_eq!(h.upstream_calls.count(), 0);
}

#[tokio::test]
async fn oversize_body_is_rejected() {
    let memory = Arc::new(MemoryAuditSink::default());
    let h = harness_with(
        vec![Arc::new(SecretScanner::new().unwrap())],
        memory.clone(),
        memory,
        Limits {
            max_body_bytes: 1024,
            ..Limits::default()
        },
    );
    let (status, resp) = post(&h, chat(&"a".repeat(4096))).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(code(&resp), "payload_too_large");
    assert_eq!(h.upstream_calls.count(), 0);
}

#[tokio::test]
async fn malformed_requests_are_rejected() {
    let h = harness();
    let (status, _) = post_raw(&h.app, Some(KEY), b"not json".to_vec()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = post(&h, json!({"messages": []})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(h.upstream_calls.count(), 0);
}

#[tokio::test]
async fn unsupported_content_and_parameters_are_refused_not_dropped() {
    let h = harness();
    let image = json!({"messages":[{"role":"user","content":[{"type":"image_url","image_url":{"url":"https://example.invalid/x.png"}}]}]});
    let (status, resp) = post(&h, image).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(code(&resp), "unsupported_content");

    let stream = json!({"stream": true, "messages":[{"role":"user","content":"hi"}]});
    let (status, resp) = post(&h, stream).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(code(&resp), "unsupported_parameter");
    assert_eq!(resp["error"]["param"], "stream");
    assert_eq!(h.upstream_calls.count(), 0);
}

#[tokio::test]
async fn detector_panic_fails_closed() {
    let memory = Arc::new(MemoryAuditSink::default());
    let h = harness_with(
        vec![Arc::new(PanicDetector)],
        memory.clone(),
        memory,
        Limits::default(),
    );
    let (status, resp) = post(&h, chat("totally harmless")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(code(&resp), "inspection_failed");
    assert_eq!(h.upstream_calls.count(), 0);
}

#[tokio::test]
async fn no_detectors_configured_fails_closed() {
    let memory = Arc::new(MemoryAuditSink::default());
    let h = harness_with(vec![], memory.clone(), memory, Limits::default());
    let (status, resp) = post(&h, chat("hello")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(code(&resp), "inspection_failed");
    assert_eq!(h.upstream_calls.count(), 0);
}

#[tokio::test]
async fn audit_failure_blocks_the_request() {
    let h = harness_with(
        vec![Arc::new(SecretScanner::new().unwrap())],
        Arc::new(FailingAuditSink),
        Arc::new(MemoryAuditSink::default()),
        Limits::default(),
    );
    let (status, resp) = post(&h, chat("hello")).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(code(&resp), "audit_failed");
    assert_eq!(
        h.upstream_calls.count(),
        0,
        "cannot release what cannot be recorded"
    );
}

#[tokio::test]
async fn audit_file_never_contains_prompt_text() {
    let dir = std::env::temp_dir().join(format!("aegis-slice1-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("audit.jsonl");
    let sink = Arc::new(JsonlFileSink::open(&path).unwrap());
    let h = harness_with(
        vec![Arc::new(SecretScanner::new().unwrap())],
        sink,
        Arc::new(MemoryAuditSink::default()),
        Limits::default(),
    );
    let key = fake_openai_key();
    let canary = "CANARY-PROMPT-7731-ZEBRA";
    post(&h, chat(&format!("{canary} my key is {key}"))).await;
    post(&h, chat(&format!("{canary} and a clean request"))).await;

    let stored = std::fs::read(&path).unwrap();
    assert!(!stored.is_empty(), "audit events must have been written");
    let leaked = find_canaries(&stored, &[canary, &key, "ZEBRA", "my key is"]);
    assert!(leaked.is_empty(), "audit file leaked: {leaked:?}");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn blocked_request_over_a_real_socket() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let h = harness();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = h.app.clone();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let body = serde_json::to_vec(&chat(&format!("key {}", fake_openai_key()))).unwrap();
    let request = format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {KEY}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.write_all(&body).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    assert!(response.starts_with("HTTP/1.1 403"), "got: {response}");
    assert!(response.contains("critical_secret"));
    assert_eq!(h.upstream_calls.count(), 0);
}
