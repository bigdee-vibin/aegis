//! Regression tests for the independent M1 security review and tester report.
//! Test names carry the finding number they pin down.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use aegis_audit::{AuditSink, FailingAuditSink, MemoryAuditSink};
use aegis_core::{Action, Detector, DetectorOutcome, ReasonCode};
use aegis_detect_secrets::SecretScanner;
use aegis_gateway::{
    router, ClientRegistry, GatewayState, Limits, Upstream, UpstreamFuture, MAX_FAILURES_PER_WINDOW,
};
use aegis_inspect::Inspector;
use aegis_testkit::RequestCounter;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;

const KEY: &str = "aegis_test_client_key";

struct CountingUpstream(RequestCounter);
impl Upstream for CountingUpstream {
    fn complete<'a>(&'a self, _request: &'a Value) -> UpstreamFuture<'a> {
        self.0.record();
        Box::pin(async { Ok(json!({"id": "mock", "choices": []})) })
    }
}

/// A detector that really panics (index out of bounds), not an injected `panic!`.
struct RealPanic;
impl Detector for RealPanic {
    fn id(&self) -> &'static str {
        "real-panic"
    }
    fn scan(&self, text: &str) -> DetectorOutcome {
        let empty: Vec<u8> = Vec::new();
        let _ = empty[text.len()];
        DetectorOutcome::Ok(vec![])
    }
}

/// A detector that is slow, to exercise the inspection timeout and concurrency limit.
struct Slow(Duration);
impl Detector for Slow {
    fn id(&self) -> &'static str {
        "slow"
    }
    fn scan(&self, _: &str) -> DetectorOutcome {
        std::thread::sleep(self.0);
        DetectorOutcome::Ok(vec![])
    }
}

struct H {
    app: axum::Router,
    calls: RequestCounter,
    audit: Arc<MemoryAuditSink>,
}

fn build(detectors: Vec<Arc<dyn Detector>>, limits: Limits, failing_audit: bool) -> H {
    let calls = RequestCounter::new();
    let memory = Arc::new(MemoryAuditSink::default());
    let sink: Arc<dyn AuditSink> = if failing_audit {
        Arc::new(FailingAuditSink)
    } else {
        memory.clone()
    };
    let mut clients = ClientRegistry::new();
    clients.add_key(1, KEY);
    let state = GatewayState::new(
        clients,
        Inspector::new(detectors),
        sink,
        Arc::new(CountingUpstream(calls.clone())),
        limits,
    );
    H {
        app: router(Arc::new(state)),
        calls,
        audit: memory,
    }
}

fn secrets() -> Vec<Arc<dyn Detector>> {
    vec![Arc::new(SecretScanner::new().unwrap())]
}

fn h() -> H {
    build(secrets(), Limits::default(), false)
}

async fn send(app: &axum::Router, key: Option<&str>, body: Vec<u8>) -> (StatusCode, Value) {
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

async fn post(h: &H, body: Value) -> (StatusCode, Value) {
    send(&h.app, Some(KEY), serde_json::to_vec(&body).unwrap()).await
}

fn chat(content: &str) -> Value {
    json!({"messages": [{"role": "user", "content": content}]})
}

fn code(v: &Value) -> &str {
    v["error"]["code"].as_str().unwrap_or("")
}

fn real_key() -> String {
    format!("sk-ant-api03-{}", "aB3dE6gH9jK2mN5pQ8sT1vW4yZ7")
}

fn b64(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s)
}

#[tokio::test]
async fn finding_1_many_decoy_tokens_before_a_real_key_fail_closed() {
    let decoys: Vec<String> = (0..80)
        .map(|i| b64(&format!("decoy payload number {i:04} padding text")))
        .collect();
    let text = format!("{} {}", decoys.join(" "), b64(&real_key()));
    let h = h();
    let (status, resp) = post(&h, chat(&text)).await;
    assert_ne!(status, StatusCode::OK, "must not be forwarded");
    assert_eq!(code(&resp), "inspection_failed", "{resp}");
    assert_eq!(h.calls.count(), 0);
}

#[tokio::test]
async fn finding_1_more_than_the_decode_budget_of_tokens_fails_closed() {
    let bigs: Vec<String> = ["abcdefghij", "klmnopqrst", "uvwxyzabcd"]
        .iter()
        .map(|s| b64(&s.repeat(15_000)))
        .collect();
    let text = format!("{} {}", bigs.join(" "), b64(&real_key()));
    let h = h();
    let (status, resp) = post(&h, chat(&text)).await;
    assert_ne!(status, StatusCode::OK);
    assert_eq!(code(&resp), "inspection_failed", "{resp}");
    assert_eq!(h.calls.count(), 0);
}

#[tokio::test]
async fn finding_1_a_few_harmless_tokens_are_still_forwarded() {
    let h = h();
    let text = format!(
        "{} {}",
        b64("hello there, this is fine"),
        b64("so is this one, really")
    );
    let (status, _) = post(&h, chat(&text)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(h.calls.count(), 1);
}

#[tokio::test]
async fn finding_2_prefixed_assignment_names_are_blocked_end_to_end() {
    for text in [
        "DB_PASSWORD=Tr0ub4dor&3xyz",
        "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        "GITHUB_TOKEN=a1B2c3D4e5f6G7h8I9j0",
        "export MY_API_KEY=q8w7e6r5t4y3abcd",
    ] {
        let h = h();
        let (status, resp) = post(&h, chat(text)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "not blocked: {text}");
        assert_eq!(code(&resp), "critical_secret");
        assert_eq!(h.calls.count(), 0);
    }
}

#[tokio::test]
async fn finding_4_secret_split_across_adjacent_messages_is_blocked() {
    let body = json!({"messages":[
        {"role":"user","content":"my key is sk-ant-api03-aB3dE6gH9jK2"},
        {"role":"user","content":[{"type":"text","text":"mN5pQ8sT1vW4yZ7 thanks"}]}
    ]});
    let h = h();
    let (status, resp) = post(&h, body).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{resp}");
    assert_eq!(code(&resp), "critical_secret");
    assert_eq!(h.calls.count(), 0);
}

#[tokio::test]
async fn finding_5_and_6_more_encodings_are_blocked() {
    let key = real_key();
    let hex_sep: String = key.bytes().map(|b| format!("0x{b:02x},")).collect();
    let entities: String = key.chars().map(|c| format!("&#{};", c as u32)).collect();
    let long = b64(&format!(
        "config line {key} end of config line, padding padding"
    ));
    let wrapped = format!("{}\n{}\n{}", &long[..44], &long[44..88], &long[88..]);
    for (name, text) in [
        ("hex with separators", hex_sep),
        ("html entities", entities),
        ("line-wrapped base64", wrapped),
        ("glued base64 prefix", format!("tok{}", b64(&key))),
        ("unicode hyphen", key.replace("sk-ant", "sk\u{2010}ant")),
        ("cyrillic homoglyph", key.replacen('a', "а", 1)),
        (
            "combining grapheme joiner",
            key.replacen("sk-", "sk-\u{034F}", 1),
        ),
    ] {
        let h = h();
        let (status, resp) = post(&h, chat(&text)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{name} not blocked: {resp}");
        assert_eq!(code(&resp), "critical_secret", "{name}");
        assert_eq!(h.calls.count(), 0, "{name} leaked upstream");
    }
}

#[tokio::test]
async fn finding_7_glued_words_do_not_hide_a_key() {
    let key = real_key();
    for text in [
        format!("x_{key}"),
        format!("prefix{key}suffix"),
        "AKIAIOSFODNN7EXAMPLEfoo".to_string(),
    ] {
        let h = h();
        let (status, _) = post(&h, chat(&text)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{text}");
    }
}

#[tokio::test]
async fn tester_3_a_real_detector_panic_blocks() {
    let h = build(vec![Arc::new(RealPanic)], Limits::default(), false);
    let (status, resp) = post(&h, chat("harmless text")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(code(&resp), "inspection_failed");
    assert_eq!(h.calls.count(), 0);
}

#[tokio::test]
async fn tester_4_audit_failure_blocks_for_both_allow_and_block_decisions() {
    // Allow decision: nothing may be released if it cannot be recorded.
    let h = build(secrets(), Limits::default(), true);
    let (status, resp) = post(&h, chat("a harmless question")).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(code(&resp), "audit_failed");
    assert_eq!(h.calls.count(), 0);
    // Block decision: still blocked, never forwarded, and still an error.
    let (status, resp) = post(&h, chat(&format!("key {}", real_key()))).await;
    assert_ne!(status, StatusCode::OK);
    assert!(
        ["audit_failed", "critical_secret"].contains(&code(&resp)),
        "{resp}"
    );
    assert_eq!(h.calls.count(), 0);
}

#[tokio::test]
async fn finding_9_failed_auth_is_throttled_and_valid_keys_still_work() {
    let h = h();
    let body = serde_json::to_vec(&chat("hello")).unwrap();
    for _ in 0..MAX_FAILURES_PER_WINDOW {
        let (status, _) = send(&h.app, Some("wrong"), body.clone()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let (status, resp) = send(&h.app, Some("wrong"), body.clone()).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(code(&resp), "rate_limited");
    let (status, _) = send(&h.app, Some(KEY), body).await;
    assert_eq!(status, StatusCode::OK, "valid key must not be locked out");
}

#[tokio::test]
async fn finding_10_rejected_requests_are_audited_without_content() {
    let h = h();
    let oversize = build(
        secrets(),
        Limits {
            max_body_bytes: 512,
            ..Limits::default()
        },
        false,
    );
    let _ = send(&h.app, Some("wrong"), b"{}".to_vec()).await; // auth failure
    let _ = send(&h.app, Some(KEY), b"not json CANARY-REJECT".to_vec()).await; // invalid JSON
    let _ = send(
        &oversize.app,
        Some(KEY),
        serde_json::to_vec(&chat(&"a".repeat(4096))).unwrap(),
    )
    .await; // 413
    let reasons: Vec<ReasonCode> = h
        .audit
        .snapshot()
        .iter()
        .flat_map(|e| e.reasons.clone())
        .collect();
    assert!(reasons.contains(&ReasonCode::AuthFailed), "{reasons:?}");
    assert!(reasons.contains(&ReasonCode::InvalidRequest), "{reasons:?}");
    let big = oversize.audit.snapshot();
    assert!(big
        .iter()
        .any(|e| e.reasons.contains(&ReasonCode::PayloadTooLarge)));
    assert!(h.audit.snapshot().iter().all(|e| e.action == Action::Block));
    assert_eq!(h.calls.count(), 0);
}

#[tokio::test]
async fn finding_12_slow_inspection_times_out_closed() {
    let h = build(
        vec![Arc::new(Slow(Duration::from_millis(400)))],
        Limits {
            inspect_timeout: Duration::from_millis(50),
            ..Limits::default()
        },
        false,
    );
    let (status, resp) = post(&h, chat("hello")).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(code(&resp), "inspection_failed");
    assert_eq!(h.calls.count(), 0);
}

#[tokio::test]
async fn finding_12_concurrency_is_limited() {
    let h = build(
        vec![Arc::new(Slow(Duration::from_millis(300)))],
        Limits {
            max_concurrent: 1,
            inspect_timeout: Duration::from_secs(5),
            ..Limits::default()
        },
        false,
    );
    let a = post(&h, chat("first"));
    let b = post(&h, chat("second"));
    let ((sa, ra), (sb, rb)) = tokio::join!(a, b);
    let overloaded = [(sa, &ra), (sb, &rb)]
        .iter()
        .filter(|(s, r)| *s == StatusCode::SERVICE_UNAVAILABLE && code(r) == "overloaded")
        .count();
    assert_eq!(
        overloaded, 1,
        "exactly one request must be shed: {ra} / {rb}"
    );
    assert_eq!(h.calls.count(), 1);
}

#[tokio::test]
async fn finding_13_stream_strings_and_media_beside_text_are_refused() {
    let h = h();
    let (status, resp) = post(
        &h,
        json!({"stream": "true", "messages": [{"role": "user", "content": "hi"}]}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(code(&resp), "unsupported_parameter");
    let (status, resp) = post(
        &h,
        json!({"messages": [{"role": "user", "content": [{"type": "text", "text": "hi", "image_url": {"url": "x"}}]}]}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(code(&resp), "unsupported_content");
    assert_eq!(h.calls.count(), 0);
}
