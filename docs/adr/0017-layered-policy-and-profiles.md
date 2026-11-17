# ADR-0017: Layered policy; profiles cannot override mandatory rules

**Status:** Approved input, detailed here

**Decision.** `EffectivePolicy = Mandatory ⊕ Profile@version ⊕ ClientOverride`.
- *Mandatory* (compiled-in defaults + signed pack): critical-secret blocking, fail-closed matrix, LOCAL ONLY semantics, no-reconstruction-of-credentials, metadata-only logging. Immutable at runtime.
- *Profiles* (Personal, Coding, Research, Business, Custom): versioned documents with schema-limited knobs (sensitivity thresholds, enabled rule packs, approval strength, routing preferences, allowances). A knob may relax a default only if the rule is annotated `profile_relaxable`.
- *Client override*: can only tighten (e.g. stricter `on_review`).
- Profiles are independent (no inheritance chains in v1), versioned (immutable versions, new edits create a new version, audit records profile id+version), importable/exportable.
- Merge is validated at load: a violating profile fails to activate and the daemon keeps the last-good profile and tells the user.

**Consequences.** + No UI setting can weaken mandatory protection; reproducible decisions. − Power users may want to relax things (e.g. allow a test key) → provided as per-request, logged, *non-secret-class* allowances only, or a documented "test-key pattern" exclusion list limited to well-known dummy values.
