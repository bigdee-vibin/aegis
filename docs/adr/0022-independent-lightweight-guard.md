# ADR-0022: Independent, lightweight guard separate from the called model

**Status:** Proposed (2026-11-17), requested by the product owner.

**Context.** Aegis inspects content before it reaches a model. If the inspector were the same model (or shared context) as the model being called, a malicious prompt could steer both, and LOCAL ONLY would couple security to inference. The daemon already runs independent of any browser (ADR-0001, ADR-0003).

**Decision.**
1. Stage 1 inspection is deterministic and model-free; it alone enforces critical blocking.
2. Stage 2 is a small local, non-generative classifier (ADR-0015), a different instance from any model being called, in every mode. It may only raise severity (ADR-0005).
3. Local-model rewriting (P1) uses a model instance separate from the target, runs with no tools, treats content as data, and yields proposals requiring user approval.
4. Startup validation fails if guard and target resolve to the same model instance or share a context/session.
5. Resource budgets (disk, memory, latency) are set in S3.6 and enforced by tests; a missing guard on a cloud route follows OPEN-04 (fail closed for Medium+ findings).

**Alternatives.** Reuse the target model as guard (rejected: shared failure mode, breaks LOCAL ONLY isolation); cloud evaluator (rejected: sends raw prompts out, FR-CTX-5); LLM-only guard (rejected: injectable, non-deterministic).

**Consequences.** Extra local resource use; one more model artefact to licence and update; adds AT-19. System-wide interception remains out of v1.0 (FR-DEP-4).
