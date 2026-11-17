# ADR-0007: Full-buffer response inspection with synthetic streaming

**Status:** Approved input (buffer complete response before inspection/release), detailed here

**Context.** Secrets/PII or injected tool calls can appear anywhere in a response, including the final tokens; per-chunk inspection can miss tokens split across chunks and cannot recall bytes already released.

**Decision.** Upstream calls are collected fully (provider streaming may be used internally to bound memory, but nothing is released). After inspection, if the client requested `stream: true`, the gateway emits a *synthetic* stream in the client's expected SSE event format (chunking at token/word granularity with small pacing). Keep-alive SSE comments are sent while waiting. Response size and time limits apply (`max_response_bytes`, `max_wait`); exceeding limits → `provider_error`/block, never partial release.

**Consequences.**
+ Acceptance #11 holds by design; reconstruction of placeholders happens before release.
− Time-to-first-token equals total generation time; long generations may hit client timeouts → documented, configurable keep-alives, UI explains.
− Memory use per concurrent request; bounded by limits.
− On browser Tier C adapters, inbound cannot be held; see ADR-0012.

**Alternatives.** Sliding-window chunk inspection with hold-back buffer — lower latency, but residual leak window and complexity; may be offered later as an explicit "fast mode" with weaker guarantees (not default).
