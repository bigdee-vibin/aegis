# ADR-0006: Capability-token egress guard

**Status:** Proposed

**Context.** "No silent cloud fallback" and "only approved sanitised content reaches cloud" are the product's central promises. Conventions and code review are not enough; a refactor could add a stray HTTP call.

**Decision.** Create `aegis-egress`, the only crate allowed to depend on a general HTTP client. Its `send()` requires an `ApprovedOutbound { request_id, provider_id, payload_sha256, expires_at, nonce }` that can only be constructed inside `aegis-approval` (sealed constructor). At send time it re-hashes the serialized body, checks provider consent (first-use consent record), expiry and single-use nonce. Local providers use a separate loopback-only client type that cannot connect to non-loopback addresses. CI enforces the dependency rule (`cargo deny` ban on HTTP clients elsewhere; dependency-graph test). Network-deny integration harness verifies behaviour in LOCAL ONLY.

**Consequences.**
+ Acceptance #6, #7, #8 are structural; mutation-after-approval is detected.
− Slightly awkward ergonomics for developers (intentional friction); update fetching needs its own token type.
− Does not stop an attacker who modifies the binary or a malicious dependency adding sockets — mitigated by supply-chain controls and the CI egress test.

**Alternatives.** Host firewall rules only (platform-specific, user-modifiable, not testable in unit tests); convention + lints only.
