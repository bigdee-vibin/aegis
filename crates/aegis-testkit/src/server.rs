use crate::RequestCounter;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Router;
use serde_json::{json, Value};
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

const MAX_RECORDED_BODY: usize = 8 * 1024 * 1024;

/// What a recorded request looked like on the wire.
#[derive(Debug, Clone)]
pub struct RecordedRequest {
    /// HTTP method.
    pub method: String,
    /// Request path (no query).
    pub path: String,
    /// Header name/value pairs, names lower-cased.
    pub headers: Vec<(String, String)>,
    /// Exact body bytes received.
    pub body: Vec<u8>,
}

impl RecordedRequest {
    /// Value of a header, if present.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// How the mock answers requests.
#[derive(Debug, Clone)]
pub enum Behaviour {
    /// 200 with this JSON body.
    Respond(Value),
    /// Empty body with this status code.
    Status(u16),
    /// Accept the request, never answer.
    Hang,
    /// 200 with a body that is not valid JSON.
    Malformed,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Cloud,
    Ollama,
}

struct Shared {
    kind: Kind,
    requests: Mutex<Vec<RecordedRequest>>,
    behaviour: Mutex<Behaviour>,
    counter: RequestCounter,
}

/// A loopback HTTP server that records every request it receives.
pub struct MockServer {
    addr: SocketAddr,
    shared: Arc<Shared>,
    task: JoinHandle<()>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

async fn handle(State(shared): State<Arc<Shared>>, req: Request<Body>) -> Response {
    let (parts, body) = req.into_parts();
    let bytes = axum::body::to_bytes(body, MAX_RECORDED_BODY)
        .await
        .map(|b| b.to_vec())
        .unwrap_or_default();
    let path = parts.uri.path().to_string();
    lock(&shared.requests).push(RecordedRequest {
        method: parts.method.to_string(),
        path: path.clone(),
        headers: parts
            .headers
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_ascii_lowercase(),
                    String::from_utf8_lossy(v.as_bytes()).into_owned(),
                )
            })
            .collect(),
        body: bytes,
    });
    shared.counter.record();

    if shared.kind == Kind::Ollama && path == "/api/tags" {
        return axum::Json(json!({ "models": [] })).into_response();
    }
    let behaviour = lock(&shared.behaviour).clone();
    match behaviour {
        Behaviour::Respond(value) => axum::Json(value).into_response(),
        Behaviour::Status(code) => StatusCode::from_u16(code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
        Behaviour::Hang => std::future::pending().await,
        Behaviour::Malformed => (StatusCode::OK, "{ this is not json").into_response(),
    }
}

impl MockServer {
    async fn start(kind: Kind, default: Value) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let shared = Arc::new(Shared {
            kind,
            requests: Mutex::new(Vec::new()),
            behaviour: Mutex::new(Behaviour::Respond(default)),
            counter: RequestCounter::new(),
        });
        let app = Router::new().fallback(handle).with_state(shared.clone());
        let task = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        Ok(Self { addr, shared, task })
    }

    /// A stand-in for a cloud provider (OpenAI-shaped chat completion reply).
    ///
    /// # Errors
    /// Returns an I/O error if no loopback port can be bound.
    pub async fn cloud() -> io::Result<Self> {
        Self::start(
            Kind::Cloud,
            json!({
                "id": "mock-cloud", "object": "chat.completion",
                "choices": [{ "index": 0, "finish_reason": "stop",
                    "message": { "role": "assistant", "content": "mock cloud reply" } }]
            }),
        )
        .await
    }

    /// A stand-in for Ollama (`/api/chat` reply, empty `/api/tags`).
    ///
    /// # Errors
    /// Returns an I/O error if no loopback port can be bound.
    pub async fn ollama() -> io::Result<Self> {
        Self::start(
            Kind::Ollama,
            json!({
                "model": "mock-model", "done": true,
                "message": { "role": "assistant", "content": "mock local reply" }
            }),
        )
        .await
    }

    /// Address the server listens on.
    #[must_use]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// `http://127.0.0.1:<port>`.
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Change how subsequent requests are answered.
    pub fn set_behaviour(&self, behaviour: Behaviour) {
        *lock(&self.shared.behaviour) = behaviour;
    }

    /// Number of requests received.
    #[must_use]
    pub fn request_count(&self) -> usize {
        self.shared.counter.count()
    }

    /// Shared counter, for code that wants to hold one without the server.
    #[must_use]
    pub fn counter(&self) -> RequestCounter {
        self.shared.counter.clone()
    }

    /// Every request received, in order, with exact bodies.
    #[must_use]
    pub fn requests(&self) -> Vec<RecordedRequest> {
        lock(&self.shared.requests).clone()
    }

    /// Stop listening, as if the service crashed: later connections are
    /// refused. Waits until the listener is closed.
    pub async fn stop(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// A listener that stands in for "the internet". Any TCP connection to it is
/// recorded and closed immediately, with no protocol spoken.
pub struct ConnectionTripwire {
    addr: SocketAddr,
    connections: Arc<AtomicUsize>,
    task: JoinHandle<()>,
}

impl ConnectionTripwire {
    /// Start the tripwire on a free loopback port.
    ///
    /// # Errors
    /// Returns an I/O error if no loopback port can be bound.
    pub async fn start() -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let connections = Arc::new(AtomicUsize::new(0));
        let seen = connections.clone();
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                seen.fetch_add(1, Ordering::SeqCst);
                drop(stream);
            }
        });
        Ok(Self {
            addr,
            connections,
            task,
        })
    }

    /// Address to configure as a provider endpoint that must never be used.
    #[must_use]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// `http://127.0.0.1:<port>`.
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// TCP connections accepted so far.
    #[must_use]
    pub fn connection_count(&self) -> usize {
        self.connections.load(Ordering::SeqCst)
    }
}

impl Drop for ConnectionTripwire {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::http::{post_json, request};
    use crate::sha256_hex;
    use std::time::Duration;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn cloud_records_exact_bytes_and_headers() {
        let cloud = MockServer::cloud().await.unwrap();
        assert_eq!(cloud.request_count(), 0);
        let body = br#"{"messages":[{"role":"user","content":"hello"}]}"#;
        let resp = post_json(
            cloud.addr(),
            "/v1/chat/completions",
            &[("Authorization", "Bearer test-key")],
            body,
        )
        .await
        .unwrap();
        assert_eq!(resp.status, 200);
        assert!(resp.text().contains("mock cloud reply"));
        let seen = cloud.requests();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].path, "/v1/chat/completions");
        assert_eq!(seen[0].header("authorization"), Some("Bearer test-key"));
        assert_eq!(sha256_hex(&seen[0].body), sha256_hex(body));
        assert_eq!(cloud.counter().count(), 1);
    }

    #[tokio::test]
    async fn ollama_answers_chat_and_tags() {
        let ollama = MockServer::ollama().await.unwrap();
        let tags = request(ollama.addr(), "GET", "/api/tags", &[], b"")
            .await
            .unwrap();
        assert!(tags.text().contains("models"));
        let chat = post_json(ollama.addr(), "/api/chat", &[], b"{}")
            .await
            .unwrap();
        assert!(chat.text().contains("mock local reply"));
        assert_eq!(ollama.request_count(), 2);
    }

    #[tokio::test]
    async fn stopped_server_refuses_connections() {
        let ollama = MockServer::ollama().await.unwrap();
        let addr = ollama.addr();
        assert!(post_json(addr, "/api/chat", &[], b"{}").await.is_ok());
        ollama.stop().await;
        let err = post_json(addr, "/api/chat", &[], b"{}").await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::ConnectionRefused);
    }

    #[tokio::test]
    async fn failure_behaviours() {
        let server = MockServer::ollama().await.unwrap();
        server.set_behaviour(Behaviour::Status(500));
        let r = post_json(server.addr(), "/api/chat", &[], b"{}")
            .await
            .unwrap();
        assert_eq!(r.status, 500);

        server.set_behaviour(Behaviour::Malformed);
        let r = post_json(server.addr(), "/api/chat", &[], b"{}")
            .await
            .unwrap();
        assert_eq!(r.status, 200);
        assert!(serde_json::from_slice::<Value>(&r.body).is_err());

        server.set_behaviour(Behaviour::Hang);
        let hung = tokio::time::timeout(
            Duration::from_millis(300),
            post_json(server.addr(), "/api/chat", &[], b"{}"),
        )
        .await;
        assert!(hung.is_err(), "a hanging server must not answer");
        assert_eq!(server.request_count(), 3, "the hung request still counts");
    }

    #[tokio::test]
    async fn tripwire_counts_raw_connections() {
        let trip = ConnectionTripwire::start().await.unwrap();
        assert_eq!(trip.connection_count(), 0);
        let mut s = tokio::net::TcpStream::connect(trip.addr()).await.unwrap();
        s.write_all(b"x").await.unwrap();
        for _ in 0..50 {
            if trip.connection_count() == 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(trip.connection_count(), 1);
    }

    #[tokio::test]
    async fn tripwire_stays_at_zero_when_unused() {
        let trip = ConnectionTripwire::start().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(trip.connection_count(), 0);
    }
}
