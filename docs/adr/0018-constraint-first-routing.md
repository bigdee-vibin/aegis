# ADR-0018: Constraint-first routing with explanations

**Status:** Proposed

**Decision.** The router first *filters* by hard constraints (operating mode; cloud consent granted; provider sensitivity ceiling vs. inspected content; capability needs: tools, JSON schema, context length; health), then *ranks* survivors by profile preference (privacy → capability → latency/cost). Output includes every considered provider with a reason code for each rejection; the UI renders a plain-language explanation ("Used local Llama because the prompt contained a medical record and your Personal profile restricts cloud use"). Failure handling returns `Fail` or `ProposeFallback`; automatic fallback to a different *locality* is not a representable value. In AUTO, cloud routes always see only the approved sanitised payload; in HYBRID the split rule is user-defined; LOCAL ONLY has an empty cloud candidate set by construction.

**Consequences.** + Transparent, testable (table-driven) routing. − Quality heuristics (is local model good enough?) are approximations → expose override and learn only via user-approved suggestions (P2).
