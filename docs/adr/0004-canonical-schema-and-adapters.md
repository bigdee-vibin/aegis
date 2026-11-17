# ADR-0004: Canonical schema + provider adapters

**Status:** Approved input, detailed here

**Context.** Two downstream wire formats (Responses, Chat Completions) and multiple providers with differing capabilities. Inspection must work identically regardless of wire format.

**Decision.** All traffic is parsed into `AegisRequest`/`AegisResponse` (`aegis-core`). Inspection, policy and routing operate only on canonical types. Front-ends (gateway handlers for the two APIs; extension) are *translators in*; providers are *translators out*. Unknown/unsupported parameters are carried in `passthrough` only if the target adapter declares support; otherwise the request fails with `aegis_unsupported_parameter` — never silently dropped. Untrusted provenance is a property of each `Message`.

**Consequences.**
+ One inspection implementation; new providers = one adapter + conformance tests.
+ Honest failure semantics.
− Canonical model must evolve with the APIs (versioned; golden fixtures from real API traces, scrubbed). Lossy edge cases (e.g. Responses `reasoning` items) need explicit handling or rejection.

**Alternatives.** Inspect raw JSON per API (duplicated logic, higher bypass risk through unmodelled fields).
