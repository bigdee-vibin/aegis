use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

/// Why the upstream call did not produce a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpstreamError {
    /// No provider is configured or reachable.
    Unavailable,
    /// The provider returned an error.
    Failed,
}

/// Boxed future returned by [`Upstream::complete`].
pub type UpstreamFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Value, UpstreamError>> + Send + 'a>>;

/// Where approved requests go. M1 has no real providers: only mocks and
/// [`NoUpstream`]. Real adapters arrive in M3 behind the egress guard, and
/// inbound response inspection (I5) in M4 — until then nothing returned from
/// an upstream is inspected, so no real upstream may be wired in.
pub trait Upstream: Send + Sync {
    /// Complete a request that has been inspected and approved.
    fn complete<'a>(&'a self, request: &'a Value) -> UpstreamFuture<'a>;
}

/// Refuses every request. The daemon's default until providers exist.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoUpstream;

impl Upstream for NoUpstream {
    fn complete<'a>(&'a self, _request: &'a Value) -> UpstreamFuture<'a> {
        Box::pin(async { Err(UpstreamError::Unavailable) })
    }
}
