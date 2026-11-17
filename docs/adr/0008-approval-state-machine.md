# ADR-0008: Approval state machine and re-inspection

**Status:** Proposed

**Context.** Requirements: notify by default, explicit approval before transformations, separate transform vs transmit approval, sensitivity-based approvals, re-inspect user edits, never silently override.

**Decision.** Each request is a persisted-in-memory state machine (see system architecture §4). Rules:
- Any change to candidate payload bytes (transform accepted, user edit, local rewrite) → mandatory `Reinspecting` using the full pipeline.
- `ApproveTransmit` carries the SHA-256 of the exact displayed final bytes; the daemon mints `ApprovedOutbound` only on a match.
- Approval strength by severity: Low = notify; Medium = one-click transmit approval; High = explicit confirm with final-bytes view and per-finding acknowledgement; Critical on mandatory list = not approvable.
- Approvals are per-request by default; "remember for this client/site/category" allowances exist only for non-mandatory, ≤ Medium findings, are profile-scoped, visible and revocable, and expire.
- Held HTTP requests time out with `approval_timeout`; non-interactive API keys may be configured `on_review: block`.
- Review content lives in memory only and is zeroised.

**Consequences.** + Approve-X-send-Y impossible; user edits can't smuggle new secrets. − Friction; mitigated by allowances and good defaults. − Held connections need keep-alive/timeouts handling.

**Alternatives.** Single combined approval (conflates consent to rewrite with consent to send); rejected per requirements.
