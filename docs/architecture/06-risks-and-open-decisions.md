# Aegis — Technical Risks, Open Decisions, Security Trade-offs

## 1. Technical risks

| ID | Risk | Likelihood | Impact | Mitigation / owner |
|---|---|---|---|---|
| R-01 | Detection recall/precision insufficient → leaks or alert fatigue | High | Critical | Layered detection; mandatory deterministic rules; published metrics; corpora gates; tune defaults to profile; never claim completeness |
| R-02 | Site adapters are fragile (DOM/network changes, anti-automation, ToS) | High | High | Tier model, canaries with auto-downgrade, small curated set, legal review of interception (S0.7 spike first) |
| R-03 | Inbound enforcement impossible for some sites (can't hold the stream) | High | Medium | Tier C = detect-only, shown honestly (ADR-0012) |
| R-04 | Buffered responses hurt UX and trip client timeouts | Medium | Medium | Keep-alives, configurable limits; document; revisit "fast mode" as opt-in |
| R-05 | Linux secure storage inconsistency (no Secret Service, Wayland/headless) | Medium | High | Master-password mode required when absent; no plaintext fallback |
| R-06 | NM installation across browsers/OS/Snap/Flatpak; MV3 worker lifecycle | High | Medium | Repair tool; documented unsupported packaging; reconnect/backoff |
| R-07 | Local model quality/latency/licensing; multilingual gaps | Medium | Medium | Pluggable classifiers; supported-language statement; eval gates |
| R-08 | Responses API drift (new item types, server tools) | High | Medium | Strict parsing — unknown item types rejected rather than passed through; conformance fixtures |
| R-09 | OS identity binding for loopback TCP is racy/best-effort | Medium | Low–Medium | Defence in depth only; control plane uses true peer creds |
| R-10 | Approval fatigue → users rubber-stamp | High | High | Allowances for ≤ Medium non-mandatory; clear diffs; severity-tiered friction; UX testing |
| R-11 | Transform quality degrades task meaning | Medium | High | Preservation checker; show actual diff; local-model rewrites treated as proposals |
| R-12 | Supply-chain (crates, npm, models, rule packs, updates) | Medium | Critical | `cargo vet/deny`, lockfiles, pinned hashes, signed packs, reproducible builds |
| R-13 | SQLCipher/ONNX build & packaging complexity | Medium | Medium | Vendored builds, CI matrix early (S0.5) |
| R-14 | Memory exposure of plaintext (review buffers, mappings) | Medium | High | zeroize, short lifetimes, no swap-sensitive caches where possible (`mlock` best-effort), no core dumps |
| R-15 | Scope breadth vs "working boundary first" | High | High | Strict milestone gating; P2 frozen until M7 |

## 2. Open decisions

> **All ten resolved on 2026-11-17 — see [ADR-0019](../adr/0019-open-decisions-resolution.md).** The table below is kept as the original question record. Key outcomes: OPEN-01 = ChatGPT + Claude + Gemini (tier per spike), OPEN-02 = opt-in updates, OPEN-05 = per-request override, OPEN-08 = Apache-2.0; the rest adopted the recommendations below.

| ID | Question | Architect's recommendation | Blocking |
|---|---|---|---|
| OPEN-01 | Which AI websites are the first adapters? | Pick 1–2 with highest user share *and* feasible interception per S0.7 spike | S5.4 |
| OPEN-02 | Update checks vs "zero outbound by default" | Opt-in at first run, default off; manual check available (ADR-0016) | S7.4 |
| OPEN-03 | Local NER/classifier model choice & licence; supported languages in v1 | English-first; evaluate 2–3 candidates in S3.6; state language limits | S3.6 |
| OPEN-04 | Default behaviour when Stage-2 classifier is unavailable on cloud routes | Block if any Stage-1 finding ≥ Medium or profile requires contextual coverage; otherwise proceed with notice | S3.6 |
| OPEN-05 | May users override "uninspected content" blocking on cloud routes? | Yes, per request, explicit warning, logged as metadata; default block | S4.4 |
| OPEN-06 | Test-key/dummy-secret allowances for developers (Coding profile) | Allow exact well-known dummy patterns only; never entropy-based exemptions | S1.4 |
| OPEN-07 | Retention default for pseudonym mappings | 24 h TTL or end of conversation; user-configurable | S4.5 |
| OPEN-08 | Licensing / open-source posture (affects community adapters, rule packs) | Decide before S0.1 publishes any code; Apache-2.0 for core is the default suggestion | S0.1 |
| OPEN-09 | Name/port defaults, extension ID strategy (needed for NM `allowed_origins`) | Reserve ID early via a dev key | S5.1 |
| OPEN-10 | Should HYBRID's local/cloud split be per-profile or global? | Per-profile | S3.2 |

## 3. Security trade-offs we are knowingly making

| Trade-off | Chosen side | Cost accepted |
|---|---|---|
| Buffered vs streaming responses | Security (full inspection) | Latency, client timeouts |
| Block vs warn on unsupported content | Block on cloud routes | Reduced multimodal/Responses features |
| Daemon as single enforcement point | Central TCB | Availability dependency; fail-closed |
| Deterministic rules over LLM judgment | Predictability & non-injectability | Higher false-positive friction |
| Per-user process vs system service | Compatibility with per-user key stores | Protection begins at login |
| Honest tiers vs "works everywhere" | Honesty | Smaller advertised website coverage |
| Short hashes in audit | Traceability of approved payloads | Tiny theoretical correlation risk; can be turned off |
| Update checks off by default | Zero-telemetry claim | Slower patch uptake |
| Same-user-malware out of scope | Feasibility | Users with compromised accounts get no guarantee |

## 4. Requirement conflicts surfaced (please confirm resolution)

1. **"Buffer entire response before release" vs browser sites.** Enforceable at the gateway and Tier B only; Tier C is detect-only after render. (ADR-0007/0012)
2. **"Zero outbound telemetry by default" vs "notify on updates".** Resolved as opt-in checks. (ADR-0016)
3. **"Preserve unsupported-parameter errors" vs "Responses API with server state".** Server-side state/tools are rejected explicitly rather than supported-but-uninspected. (ADR-0013)
4. **"Text-only v1" vs real traffic containing files/images.** Block-by-default on cloud routes with an explicit `uninspected` label. (ADR-0013)
5. **"Default to notification rather than silent modification" vs sensitivity-gated blocking.** Interpreted as: modification never happens without approval; blocking of mandatory-critical content is not a modification and happens without approval.
