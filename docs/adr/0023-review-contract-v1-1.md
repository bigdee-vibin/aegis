# ADR-0023: Review Center contract v1.1

**Status:** Accepted (2026-11-17).

**Context.** The design session asked for three changes to the Review Center contract (architecture 03 §10): an explicit statement of which text will be transmitted, the original-text hash earlier, and a decision on a 30-second reject-undo. ADR-0021 requires an ADR plus a message to the design session for any contract change.

**Decisions.**
1. Add `ApprovalStep.transmit_content` (`original` | `transformed` | `null`). The daemon sets it; the UI does not infer it.
2. Add `ContentView.original_sha256`, available from the first snapshot. Computed over the exact original bytes before masking.
3. Drop reject-undo for v1: rejection is final. Holding or replaying a request after the user rejected it conflicts with FR-POL-6 (rejected requests are not submitted) and complicates the approval state machine and hash binding.
4. Versioning: `contract_version` stays 1; additive changes bump `contract_minor`. Consumers treat a missing `contract_minor` as 0. Breaking changes bump `contract_version`.

**Consequences.** The design board can show a WILL BE SENT banner with a hash in both the original and transformed cases without guessing. Engineering must hash the original payload at intake (cheap) and set `transmit_content` when building approval steps (S2.3). No undo affordance appears in v1 screens.
