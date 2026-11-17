# ADR-0019: Resolution of OPEN-01 … OPEN-10

**Status:** Accepted (2026-11-17). "Owner" = decided by the product owner; "Architect default" = recommendation adopted as the working default, revisable by ADR.

| ID | Decision | Decided by | Effect |
|---|---|---|---|
| OPEN-01 | First adapters: **ChatGPT, Claude, Gemini**. Order and achievable tier per site are set by spike S0.7. Expectation: ChatGPT/Claude may reach Tier B or C; Gemini likely Tier C (DOM-only) due to its batched RPC transport. A site that cannot meet its target tier ships at the tier it can honestly support, shown in the UI. | Owner | S5.4 scope = 3 adapters; each ships with a capability manifest, fixtures and live canary. Three adapters at M5 is an ambitious scope (R-02, R-15): ship in order of spike feasibility, and do not delay M5 exit for the third. |
| OPEN-02 | Update checks **opt-in, default off**; manual "Check now" always available; signed manifest, no identifiers, user approves before download. | Owner | ADR-0016 moves to Accepted. #16 must hold with checks off. |
| OPEN-03 | English-first for v1; evaluate 2–3 local NER/classifier candidates in S3.6; licence must be permissive; supported languages stated in UI/docs. | Architect default | S3.6 evaluation criteria fixed. |
| OPEN-04 | Stage-2 unavailable on a cloud route: **block** if any Stage-1 finding ≥ Medium or the profile requires contextual coverage; otherwise proceed with a visible "contextual check skipped" notice. | Architect default | Policy defaults and fail-closed matrix confirmed. |
| OPEN-05 | Uninspected content on cloud routes: **default block; per-request explicit override** with persistent `UNINSPECTED` warning; override recorded as metadata only. Not available at profile level. | Owner | ADR-0013 Accepted; Review Center needs an override flow (UX). |
| OPEN-06 | Coding profile may exempt **exact well-known dummy/test key patterns** only; no entropy-based or user-regex exemptions for credential-class findings. | Architect default | S1.4 includes a signed dummy-pattern list. |
| OPEN-07 | Pseudonym mapping retention: **24 h TTL or conversation end**, whichever first; user-configurable (shorter only for mandatory layer; longer allowed per profile up to 30 days). | Architect default | S4.5 parameters. |
| OPEN-08 | **Apache-2.0** for core, rule-pack framework and adapter SDK. Bundled models/rule data keep their own licences, tracked in a NOTICE/SBOM. | Owner | S0.1 adds LICENSE, NOTICE, DCO/CLA decision (CLA vs DCO still to pick; default DCO). |
| OPEN-09 | Generate a **dev extension key now** to fix the extension ID for NM `allowed_origins`; production ID reserved when the store listing is created. | Architect default | S0.6/S5.1 use the dev ID; keep the key out of the repo. |
| OPEN-10 | HYBRID local/cloud split is **per-profile**. | Architect default | S3.2 / S6.4 design. |

**Consequences.** All ten blockers are cleared for M0. Items marked Architect default are not product-owner-approved; they can be changed without rework before the stories listed consume them.
