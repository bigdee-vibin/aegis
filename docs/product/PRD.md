# Aegis AI Firewall — Product Requirements Document (v1.0 draft 1)

**Status:** Draft for owner review · **Date:** 2026-11-17 · **Owner:** Product owner · **PM:** Delivery lead
**Related:** [Architecture](../architecture/01-system-architecture.md) · [Threat model](../security/02-threat-model.md) · [Backlog](../architecture/04-backlog.md) · [ADRs](../adr/README.md) · [Audit](../status/2026-11-17-initial-audit.md)

Conventions: **[C]** confirmed decision · **[R]** PM recommendation awaiting confirmation · **[Q]** open question. Priorities: **P0** must for v1.0 privacy MVP, **P1** should (v1.0 if capacity), **P2** later. Nothing here is verified by tests yet; no effectiveness claim is made until measured (§11).

## 1. Vision and problem

**Aegis — your privacy firewall for the AI era.** A local-first layer that controls what personal and sensitive information reaches AI providers, inspects what comes back, and routes requests between local and cloud models with privacy taking precedence. [C]

**Problem.** People paste personal data, credentials, business and defence-related material into AI tools with no visibility or control, and have no honest way to know what a given site or app actually protects.

**Audience (v1.0):** individual AI users. **Primary mission:** privacy protection. [C]

## 2. Goals and non-goals

**Goals:** (G1) prevent critical secrets leaving the device; (G2) make every sensitive-data decision visible and user-controlled; (G3) preserve task meaning when sanitising; (G4) inspect inbound responses; (G5) enforce LOCAL ONLY absolutely; (G6) be honest about coverage.

**Non-goals for v1.0:** team/enterprise management; cloud-hosted Aegis service; telemetry; universal website coverage; protection against malware running as the same OS user; multimodal (image/file) inspection (blocked on cloud routes instead, ADR-0013); token optimisation, learning suggestions, desktop-app connectors (P2).

## 3. Users and key scenarios

| Persona | Scenario |
|---|---|
| Developer using ChatGPT/IDE tools | Pastes code with an API key → blocked; dummy test keys allowed per Coding profile |
| Professional using AI for documents | Personal data detected → review original vs proposed side by side → approve transform, then approve transmit |
| Privacy-conscious user | LOCAL ONLY mode; zero cloud calls even if Ollama is down |
| Defence-adjacent user | Operational/personnel/restricted-document context flagged for review, not auto-classified as classified |

## 4. Confirmed decisions (summary)

Chrome extension + OpenAI-compatible gateway in parallel as delivery direction; notify by default, never silently transform; outbound and inbound protection; automatic routing with manual override; deep context (conversation, documents, relevant history) with approval of changes; learning suggests only; metadata-only logging; multiple independent profiles; sensitivity-based approval; local-only reconstruction; Ollama runtime; central encrypted credential vault; hybrid client authentication; both `/v1/responses` and `/v1/chat/completions`; localhost default with explicit secured LAN; curated site adapters; buffer-before-forward inspection; dark-first with accessible light theme. Decisions OPEN-01…10 resolved in ADR-0019; **v1.0 OS scope = Windows, macOS, Linux** (ADR-0020). [C]

## 5. Functional requirements

### 5.1 Detection (FR-DET)
| ID | Requirement | P |
|---|---|---|
| FR-DET-1 | Detect personal identity info, credentials/secrets, financial, confidential, technical, personal/sensitive records, contextual identity, military/defence information | P0 |
| FR-DET-2 | Stage 1 deterministic detectors (provider key formats, PEM/private keys, JWTs, checksums for cards/IBAN/IDs, email, phone) always run and alone suffice to block critical secrets | P0 |
| FR-DET-3 | Every finding carries category, severity, location, confidence and a plain-language reason | P0 |
| FR-DET-4 | Stage 2 local classifier (English-first, permissive licence) adds contextual findings; may only raise severity, never lower (ADR-0005) | P1 |
| FR-DET-5 | Military/defence handled as context categories (operational detail, restricted documents, infrastructure, personnel, technology); general military discussion is not auto-classified as classified | P1 |
| FR-DET-6 | Dummy/test keys exempted only via exact well-known patterns in Coding profile; no entropy or user-regex exemptions for credential class (OPEN-06) | P0 |
| FR-DET-7 | Prompt-injection detector; untrusted content (uploads, retrieved data, tool results) cannot alter Aegis policy | P0 |
| FR-DET-8 | If Stage 2 is unavailable on a cloud route: block when any Stage-1 finding ≥ Medium or the profile needs contextual coverage; else proceed with a visible "contextual check skipped" notice (OPEN-04) | P0 |

### 5.2 Policy and approval (FR-POL)
| ID | Requirement | P |
|---|---|---|
| FR-POL-1 | Risk levels Low / Moderate / High / Critical with actions: notify / explain+offer sanitise / require review before transmission / block | P0 |
| FR-POL-2 | Aegis never modifies user content without approval. Blocking mandatory-critical content is not a modification and needs no approval [C] | P0 |
| FR-POL-3 | Two distinct approvals: approve transformation; approve transmission. Transmit approval is bound to the exact content hash | P0 |
| FR-POL-4 | Review shows original and proposed content side by side with what changed and why | P0 |
| FR-POL-5 | User edits to proposed content are re-inspected before transmission | P0 |
| FR-POL-6 | Rejected requests are not submitted; held requests time out with `approval_timeout` | P0 |
| FR-POL-7 | Layered policy: mandatory critical rules cannot be relaxed by any profile (ADR-0017) | P0 |
| FR-POL-8 | Approval allowances for ≤ Medium non-mandatory findings to limit fatigue (R-10) | P1 |
| FR-POL-9 | Uninspected content (images/files) on cloud routes blocked by default; per-request explicit override with persistent `UNINSPECTED` warning, never profile-level (OPEN-05) | P0 |

### 5.3 Transformation and context (FR-CTX)
| ID | Requirement | P |
|---|---|---|
| FR-CTX-1 | Redaction, anonymisation and stable local pseudonyms that preserve technical details, constraints, dates, numbers, negations and exceptions | P0 (deterministic) / P1 (contextual) |
| FR-CTX-2 | Preservation checker; ask the user when semantic equivalence is uncertain | P0 |
| FR-CTX-3 | Classify identity, task, technical, environmental, historical and sensitive context | P1 |
| FR-CTX-4 | Context from current conversation, documents and relevant history held in memory/session only; context changes need approval | P1 |
| FR-CTX-5 | Raw prompts are never sent to external evaluators for privacy scoring | P0 |
| FR-CTX-6 | Local-model rewrites are proposals, never applied silently | P1 |

### 5.4 Reconstruction (FR-REC)
| ID | Requirement | P |
|---|---|---|
| FR-REC-1 | Mappings encrypted at rest, stored only locally; reconstruction is local only | P0 |
| FR-REC-2 | Never map or reconstruct passwords, API keys, private keys or other blocked secrets | P0 |
| FR-REC-3 | Retention: 24 h or conversation end, whichever first; user-configurable (longer per profile up to 30 days) [Q: see §14 Q-3]; expiry crypto-shreds mappings | P0 |
| FR-REC-4 | Reconstructed responses go only to a verified local client; never to LAN clients unless explicitly allowed per application [R] | P0 |

### 5.5 Inbound inspection (FR-INB)
| ID | Requirement | P |
|---|---|---|
| FR-INB-1 | Gateway buffers the full response, inspects, then releases (synthetic SSE for streaming clients) (ADR-0007) | P0 |
| FR-INB-2 | Sensitive data and injection in responses detected; secret in any chunk blocks all | P0 |
| FR-INB-3 | Tool/function-call arguments and results inspected; risky calls need approval | P1 |
| FR-INB-4 | Browser sites: hold-and-inspect only at Tier B; Tier C is detect-only after render and is labelled as such | P1 |

### 5.6 Profiles and modes (FR-PRF, FR-MOD)
| ID | Requirement | P |
|---|---|---|
| FR-PRF-1 | Multiple independent profiles: Coding, Research, Personal, Business, Custom; suggestions allowed, activation by user only | P1 (a default profile in P0) |
| FR-PRF-2 | Profiles are versioned; cannot relax mandatory rules | P1 |
| FR-MOD-1 | LOCAL ONLY: no cloud inference, no silent fallback; Ollama failure yields a proposed fallback that needs explicit user choice | P0 |
| FR-MOD-2 | ONLINE ONLY, HYBRID (split per profile, OPEN-10) and AUTO modes | P1 |
| FR-MOD-3 | Router is constraint-first: privacy then capability, quality, cost, latency, availability; shows explanation; manual override respects policy | P1 |
| FR-MOD-4 | Distinguish local security processing from local model inference everywhere in UI and docs | P0 |

### 5.7 Gateway and providers (FR-GW, FR-PRV)
| ID | Requirement | P |
|---|---|---|
| FR-GW-1 | `POST /v1/chat/completions` and `POST /v1/responses`, streaming, OpenAI-style errors; unsupported parameters return explicit errors, not silent drops | P0 (chat, non-stream) / P1 |
| FR-GW-2 | Per-application API keys plus supported local identity verification; localhost default; LAN only when explicitly enabled and authenticated | P0 |
| FR-GW-3 | Request validation, size limits, rate limits; unknown Responses item types rejected; `previous_response_id`/server-side state/tools rejected (ADR-0013) | P0 |
| FR-PRV-1 | Provider adapters behind one canonical request/response model: Ollama, OpenAI, OpenAI-compatible, OpenRouter (potential) with capability detection | P1 |
| FR-PRV-2 | Provider credentials in a central encrypted vault (DPAPI / Keychain / Secret Service or master password); never visible to clients, browser code, logs or models | P0 |
| FR-PRV-3 | First-use cloud consent per provider/integration; revocable | P1 |
| FR-PRV-4 | Single egress component sends only content carrying an approval capability (ADR-0006) | P0 |

### 5.8 Browser (FR-BRW)
| ID | Requirement | P |
|---|---|---|
| FR-BRW-1 | Chrome MV3 extension + Native Messaging to local companion; Chrome, Edge, Brave | P0 |
| FR-BRW-2 | Curated adapters, first ChatGPT, Claude, Gemini; each with capability manifest, fixtures and live canary; ships at the tier it can honestly support | P0 (≥1) / P1 |
| FR-BRW-3 | Coverage shown as Protected / Partially protected / Not protected; unsupported sites warn accurately | P0 |
| FR-BRW-4 | Companion disconnected, crashed or message malformed: pending external requests blocked (fail-closed); malformed messages rejected | P0 |
| FR-BRW-5 | Review UI in popup/side panel, never injected into page DOM | P0 |

### 5.9 Data, logging, updates (FR-DAT)
| ID | Requirement | P |
|---|---|---|
| FR-DAT-1 | Persist metadata only: timestamp, application, profile, model/provider, risk level, categories, transformation type, approval status, token counts, latency, error category (SQLite/SQLCipher) | P0 |
| FR-DAT-2 | Never persist prompts, conversations, responses, uploaded content, plaintext mappings, credentials | P0 |
| FR-DAT-3 | Audit API accepts typed events only; canary scanning of DB and logs in CI | P0 |
| FR-DAT-4 | Zero telemetry; update checks opt-in, default off, manual "Check now", signed manifest, user approves download (OPEN-02) | P1 |

### 5.10 UI (FR-UI)
| ID | Requirement | P |
|---|---|---|
| FR-UI-1 | Review Center is the signature experience: findings, why it matters, what will be shared, proposed change, consequences of approve/reject, original vs transformed transmitted | P0 (extension/headless) / P1 (desktop) |
| FR-UI-2 | Navigation: Overview, Review Center, Security Inspector, Context & Profiles, Model Router, Analytics, Integrations, Settings | P1 |
| FR-UI-3 | Dark-first, accessible light theme; WCAG 2.2 AA; keyboard operable; reduced-motion; responsive | P0 |
| FR-UI-4 | Simple by default, advanced controls on demand; no decorative complexity in security decisions | P1 |

## 6. Non-functional requirements

| ID | Requirement |
|---|---|
| NFR-1 Security | Fail closed on detector failure, timeout, companion loss; secrets zeroised; no secrets in logs/browser/model context; supply-chain controls (lockfiles, `cargo deny`, signed packs) |
| NFR-2 Privacy | Local-first; no outbound traffic except approved provider calls and opt-in update checks |
| NFR-3 Performance [R] | Added latency p95 ≤ 300 ms for Stage 1 on ≤ 4k tokens, excluding buffering; targets confirmed after baseline |
| NFR-4 Reliability | Companion restart recovery; provider timeout handling; no data loss in approval queue within a session |
| NFR-5 Platforms | Windows, macOS, Linux (ADR-0020); Linux master-password mode when no Secret Service |
| NFR-6 Accessibility | WCAG 2.2 AA, keyboard, reduced motion |
| NFR-7 Licensing | Apache-2.0 core (OPEN-08), SBOM/NOTICE for bundled models |
| NFR-8 Honesty | Never claim universal protection or effectiveness without measured results |

## 7. Operating modes (reference)

| Mode | Cloud inference | Fallback | Notes |
|---|---|---|---|
| LOCAL ONLY | None | Never automatic | Zero external inference calls |
| ONLINE ONLY | Approved providers | n/a | Aegis inspection still applies |
| HYBRID | Per-profile split | Policy-controlled | Local security processing always on |
| AUTO | Chosen by router | Proposed, not silent | Privacy > capability > quality > cost > latency |

## 8. Canonical acceptance suite (AT-01 … AT-18)

Replaces the 16-item list in the architecture docs (audit finding G-1). All must pass on Windows, macOS and Linux in CI before 1.0. Status reflects verified results: none yet.

| ID | Test | Requirements | Backlog | Milestone | Status |
|---|---|---|---|---|---|
| AT-01 | API key in prompt is blocked/removed before external transmission; counting mock cloud sees 0 requests | FR-DET-2, FR-POL-2 | S1.7 | M1 | Not started |
| AT-02 | Personal info detected; correct profile policy applied | FR-DET-1/3, FR-POL-1 | S2.1, S2.2 | M2 | Not started |
| AT-03 | High-risk context requires review where configured | FR-POL-1, FR-CTX-3 | S2.2, S4.6 | M2/M4 | Not started |
| AT-04 | Low-risk content follows notify-only | FR-POL-1/2 | S2.2 | M2 | Not started |
| AT-05 | Rejected requests are not submitted | FR-POL-6 | S2.7 | M2 | Not started |
| AT-06 | Edited sanitised content is inspected again | FR-POL-5 | S2.5 | M2 | Not started |
| AT-07 | LOCAL ONLY yields zero external inference calls (network-deny) | FR-MOD-1 | S3.3 | M3 | Not started |
| AT-08 | Ollama failure in LOCAL ONLY never triggers cloud fallback | FR-MOD-1 | S3.4 | M3 | Not started |
| AT-09 | Cloud providers receive only approved payloads | FR-PRV-4, FR-POL-3 | S2.6 | M2 | Not started |
| AT-10 | Uploaded prompt-injection content cannot override policy | FR-DET-7 | S4.2 | M4 | Not started |
| AT-11 | Sensitive data in model responses detected | FR-INB-1/2 | S4.1 | M4 | Not started |
| AT-12 | Reconstruction mappings encrypted and local | FR-REC-1/2 | S4.5 | M4 | Not started |
| AT-13 | Session expiry clears temporary mappings | FR-REC-3 | S4.5 | M4 | Not started |
| AT-14 | Companion failure blocks pending external requests safely | FR-BRW-4 | S1.8, S5.2 | M1/M5 | Not started |
| AT-15 | Malformed messages rejected | FR-BRW-4, FR-GW-3 | S0.6, S5.2 | M5 | Not started |
| AT-16 | Learning suggestions inactive until approved | FR-PRF-1 | P2 story (to add) | Post-1.0 stub; test with stub at M7 | Not started |
| AT-17 | Audit storage holds no prompt or full-response content | FR-DAT-1/2/3 | S1.6 | M1 | Not started |
| AT-18 | Unsupported websites show accurate coverage warning | FR-BRW-3 | S5.3 | M5 | Not started |

Note: the former architecture numbering differs; a mapping table must be added to 01 §12 (action item).

## 9. Priorities (MoSCoW)

**Must (v1.0):** all P0 above: Stage-1 detection, policy and approval, review flow (extension + headless/CLI), critical blocking, LOCAL ONLY enforcement, encrypted mappings and local reconstruction, metadata-only audit, fail-closed, secure NM, ≥ 1 tested site adapter, chat-completions gateway, vault.
**Should:** profiles, Ollama and cloud adapters, routing, Stage-2 classifier, desktop dashboard, Responses API and streaming conformance, tool-call inspection, three adapters.
**Could:** analytics depth, context debugger, update manager.
**Won't (v1.0):** token optimiser, learning engine, desktop-app connectors, more browsers, multimodal inspection.

**Priority fixes vs backlog:** Ollama adapter and router (S3.1/S3.2) must be P0 because AT-07/08 depend on them, or AT-07/08 are defined against a mock Ollama in P0 [Q-4].

## 10. Release milestones (from backlog)

M0 foundations and spikes → M1 secret blocked at gateway → M2 review/transform/transmit → M3 local & routing safety → M4 inbound/tools/injection/vault → M5 browser extension → M6 dashboard and P1 breadth → M7 hardening, signing, packaging, pen test. Dates are not committed until the spikes S0.5–S0.7 report on all three OSes.

## 11. Success metrics (proposed targets, to be baselined)

| Metric | Proposed target [R] | Measured by |
|---|---|---|
| Critical secret leakage in test corpus | 0 | AT-01 + secret corpus (S0.4) |
| Secret recall, top 30 key formats | ≥ 99% | S1.4 |
| PII recall, structured (cards, IDs, email, phone) | ≥ 95%, with precision target set after baseline | S2.1 |
| Contextual detection recall/precision | Measured and published, no promise before baseline | S3.6 |
| LOCAL ONLY violations | 0 across network-deny runs | AT-07/08 |
| Audit content leaks | 0 canary hits in DB and logs | AT-17 |
| Transformation semantic preservation | Corpus pass rate set after baseline | S2.4 checker |
| Approval completion / rejection rates, review time | Tracked locally, no targets in v1.0 | Local metadata |
| Added latency p95 | ≤ 300 ms Stage 1 | Perf tests |
| Coverage-label accuracy | 100% of declared tiers verified by tests | Adapter canaries |
| Usability of Review Center | Task success in moderated tests; target set before M6 | UX study |

No effectiveness statement is published unless backed by these measurements.

## 12. Dependencies and risks

Key dependencies: spikes S0.5 (SQLCipher + OS key stores), S0.6 (Native Messaging), S0.7 (site interception); local classifier choice (S3.6); legal review of site interception. Full risk register: [06-risks-and-open-decisions](../architecture/06-risks-and-open-decisions.md) (R-01…R-15). Top risks for v1.0: detection recall (R-01), adapter fragility (R-02), approval fatigue (R-10), scope breadth now that three OSes are confirmed (R-15, R-05).

## 13. Out of scope and honest limits

Aegis does not protect against malware under the same OS user, does not inspect content on unsupported sites or in unsupported formats, and does not guarantee detection of every sensitive item. UI and docs state these limits.

## 14. Open questions

| ID | Question | PM recommendation |
|---|---|---|
| Q-1 | Gateway-first sequencing (gateway M1–M4, extension M5) vs literal parallel delivery | Keep gateway-first for the TCB; run extension spikes S0.6/S0.7 in parallel from M0 |
| Q-2 | Confirm the 18-test list as canonical | Yes (this document) |
| Q-3 | Pseudonym retention up to 30 days vs "session expiry clears mappings" | Default to session/24 h; allow longer only as explicit per-profile opt-in with warning |
| Q-4 | Ollama in P0 vs mock Ollama for AT-07/08 | Promote S3.1/S3.2 to P0 for the real-model path |
| Q-5 | Reconstruction output to LAN-connected clients | Disallow by default |
| Q-6 | Review Center UX for desktop app vs extension in P0 | Extension panel + CLI in P0; desktop in P1 |

## 15. Change log

- 2026-11-17: draft 1 created from the product handoff, ADR-0001…0020 and the initial audit.
