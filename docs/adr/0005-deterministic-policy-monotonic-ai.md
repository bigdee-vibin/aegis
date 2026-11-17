# ADR-0005: Deterministic policy engine; AI classifiers are monotonic

**Status:** Proposed

**Context.** The brief requires deterministic enforcement of critical policies and AI only for contextual detection. Model outputs are non-deterministic and attackable (injection can persuade a model to say "this is safe").

**Decision.**
1. `decide()` is a pure function: no I/O, AI, clock or randomness.
2. Stage-1 deterministic detectors always run and are sufficient for all mandatory rules.
3. Stage-2 AI output merges **max-severity per span** with Stage-1; it can add findings or raise severity, never remove/lower. No model output is ever interpreted as an instruction or policy decision.
4. Classifier unavailability is a coverage fact handled by the fail-closed matrix, not a silent pass.

**Consequences.**
+ Provable properties (monotonicity) with property tests; AI failure or compromise can only cause *more* friction, not leaks.
− False positives from AI add friction → needs confidence thresholds per profile, "why flagged" explanations, and user-owned allowances for non-mandatory findings.
− A model cannot "clear" a false positive from Stage 1; fixes must go through rule updates or explicit user allowance.

**Alternatives.** LLM-as-judge for final decisions (non-deterministic, injectable); rejected.
