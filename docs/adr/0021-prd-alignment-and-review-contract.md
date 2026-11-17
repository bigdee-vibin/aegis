# ADR-0021: Align to the PRD acceptance suite; Review Center data contract

**Status:** Accepted (2026-11-17). Source of truth for requirements: `docs/product/PRD.md`.

**Context.** The product manager's PRD defines a canonical 18-item acceptance suite (AT-01…AT-18) that replaces the architect's earlier 16-item list, promotes the Ollama adapter and router to P0 (AT-07/08 need a real local model path), and asks for a Review Center data contract so the design session can build screens without inventing interfaces.

**Decisions.**
1. AT-01…AT-18 are canonical. Architecture §12 maps them to mechanisms and tests; S7.1 is the suite story. Two legacy behaviours not in the list (tool-call inspection; zero telemetry) stay as tests and are proposed to the PM as AT-19/AT-20.
2. S3.1 (Ollama adapter) and S3.2 (modes + router) are P0.
3. New story S2.9: a static catalogue of plain-language reasons and remediation text, so findings carry explanations without ever carrying matched text.
4. The Review Center data contract is published in `docs/architecture/03-api-contracts.md` §10 (`contract_version = 1`). Key properties: daemon-decides/UI-renders; full-snapshot updates with `revision`; credential spans pre-masked by the daemon with same-length bullets so the UI never receives a secret; UTF-16 offsets; transform and transmit approvals are separate steps and transmit binds to the exact payload SHA-256; coverage tier and `UNINSPECTED` override are explicit fields.
5. **Change control:** any change to §10 requires an ADR and a message to the design session. Neither side adds fields informally.

**Consequences.** Design can proceed against a fixed contract before M1 exits. The contract exposes engineering gaps now tracked: original-coordinate spans for decoded findings, the preservation checker (S2.4), router explanations (S3.2), and the reason catalogue (S2.9). The M1 pipeline reports spans in view coordinates and must be extended before the Review Center can highlight decoded findings.

**Alternatives.** Letting the UI derive risk levels, diffs and hashes (duplicates security logic, risks drift); sending raw credential text to the UI (violates the principle that clients are untrusted, ADR-0001).
