# Aegis — Initial Repository & Implementation Audit

**Date:** 2026-11-17 · **Author:** Product/Delivery lead · **Status of evidence:** inspected directly (read-only); nothing tested because no code exists.

## 0. Access and limits (read first)

| Item | Result |
|---|---|
| GitHub connector (MCP) | **Failed to connect** (`400 Authorization header is badly formatted`). Not used. |
| `gh` CLI | Works. Authenticated as `bigdee-vibin`. Used read-only. |
| Remote repo | `bigdee-vibin/aegis` — **public**, branch `main`, 1 commit (`5edf4af Initial commit`). |
| Figma | No access in this session. Design review is against `aegis-design/` files only. |
| Execution | No build/test run: there is nothing to build. |

Nothing was pushed, committed, or changed on GitHub. All files written so far are local.

## 1. Repository inventory

**Remote (`bigdee-vibin/aegis`, GitHub-confirmed):** one file, `README.md` (7 bytes, `# aegis`). 0 issues, 0 PRs, 1 branch, no CI, no license, no code.

**Local, not under git, not pushed** (`...\AII privacy\aegis`, ~102 KB, 27 files):

| Path | Content |
|---|---|
| `README.md` | Status: "architecture review (no production code yet)" |
| `docs/architecture/01,03,04,05,06` | System architecture, API contracts, backlog (M0–M7), deployment, risks/open decisions |
| `docs/security/02-threat-model.md` | Threat model |
| `docs/adr/0001–0019` | 19 ADRs (ADR-0019 resolves OPEN-01…10) |

**Local design assets** (`...\AII privacy\aegis-design`, outside the repo folder): `aegis.css` (219 lines; dark/light tokens matching the agreed palette, buttons, badges, diff, dialog, drawer, popup, toast, table, etc.) and `icons.js` (Lucide-style icon subset). No HTML screens, no `design/MASTER.md`.

**Not present anywhere:** PRD, source code, tests, CI, LICENSE/NOTICE, test matrix, integration matrix, API spec (OpenAPI), Figma export.

## 2. Existing implementation assessment

| Area | Status |
|---|---|
| Production code | **Not started** (0 lines). |
| Architecture | Substantial, internally consistent paper design. **Not yet validated by any spike** (S0.5–S0.7 undone). |
| Security design | Threat model + invariants + 19 ADRs exist. Strong choices: daemon-centric enforcement, deterministic policy with monotonic AI, capability-token egress guard, full-buffer inspection, fail-closed. |
| UI | Design tokens/components CSS only. No screens. |
| Repo hygiene | Nothing published; local docs are unversioned (no git history, no backup). |

Retain: all of it. Replace: nothing. Risk: it is all unverified claims until spikes and the first vertical slice exist.

## 3. PRD-to-code traceability (status = verified implementation only)

| Requirement area | Architecture/backlog coverage | Code | Tests | Status |
|---|---|---|---|---|
| Secret detection + critical block (AT1) | S1.4, S1.5, S1.7 | none | none | Not started |
| PII detection + policy (AT2) | S2.1, S2.2 | none | none | Not started |
| Review/approval, two distinct approvals (AT3–5) | S2.3–S2.7, ADR-0008 | none | none | Not started |
| LOCAL ONLY, no cloud fallback (AT7–8) | S3.3, S3.4, ADR-0018 | none | none | Not started |
| Egress only approved payload (AT9) | S2.6, ADR-0006 | none | none | Not started |
| Prompt injection resilience (AT10) | S4.2, ADR-0005 | none | none | Not started |
| Inbound inspection (AT11) | S4.1, ADR-0007 | none | none | Not started |
| Encrypted reconstruction + TTL (AT12–13) | S4.5, ADR-0014 | none | none | Not started |
| Fail-closed companion/malformed msgs (AT14–15) | S1.8, S5.2 | none | none | Not started |
| Learning suggestions inactive (AT16) | P2 | none | none | Not started |
| Metadata-only audit (AT17) | S1.6 | none | none | Not started |
| Unsupported-site warning (AT18) | S5.3 | none | none | Not started |

(AT numbers = the 18 tests in the PRD handoff. See gap G-1 on numbering.)

## 4. Architecture gaps and risks

| ID | Finding | Severity | Recommendation |
|---|---|---|---|
| G-1 | **Acceptance-suite mismatch:** architecture/backlog use **16** criteria (S7.1 "16/16"); the product handoff defines **18**. Mapping is undocumented. | High | Build one canonical AT-01…AT-18 table; update 01 §12 and S7.1. First task. |
| G-2 | **OS scope contradiction:** handoff says v1.0 OS support is unresolved with Windows as focus; architecture/backlog assume Windows + macOS + Linux in CI, NM installers, packaging. This triples M0/M5/M7 effort and Linux secure storage (R-05) is flagged high-risk. | High | Product owner decision needed (see §10). Recommend Windows-only v1.0 gate, with OS abstraction retained. |
| G-3 | **No PRD file exists.** Requirements live only in the handoff text; traceability has no stable IDs. | High | Write PRD with requirement IDs (FR-/NFR-); in progress next. |
| G-4 | **Backlog priority drift:** handoff P0 includes Ollama-dependent tests (AT7, AT8) and the local-only story; backlog marks Ollama (S3.1, S3.2) P1 while S3.3/S3.4 are P0. P0 tests depend on P1 stories. | Medium | Reclassify S3.1/S3.2 to P0 or redefine AT7/8 P0 as "mock Ollama". |
| G-5 | **Gateway timing:** handoff confirms Chrome extension and API gateway in parallel; the backlog delivers the gateway first (M1–M4) and the extension at M5. Architecturally sound (gateway is the TCB) but not "parallel". | Medium | Confirm sequencing with owner; the spike S0.6/S0.7 can run in parallel with M1. |
| G-6 | **Browser sites cannot be routed or fully buffered:** Tier C is detect-only post-render. Acknowledged in ADR-0007/0012; must be explicit in user-facing copy and PRD. | Medium | Keep honest tiers; add to PRD as a constraint, not a bug. |
| G-7 | **Stable pseudonyms vs session expiry:** ADR-0019 sets 24h TTL or conversation end, up to 30 days per profile. Handoff test AT13 says "session expiry clears temporary mappings." Longer retention contradicts a plain reading. | Medium | Confirm 30-day option is acceptable or cap at session. |
| G-8 | **Dependency on unproven spikes:** S0.5 (SQLCipher+DPAPI), S0.6 (Native Messaging), S0.7 (site interception) gate large parts of the design. | High | Run the three spikes before further design investment. |
| G-9 | **Scope breadth (R-15):** three adapters, three OSes, multiple providers. | High | Gate M5 on spike results; do not delay for third adapter (ADR-0019 already says so). |
| G-10 | **Public repo.** Threat model and architecture detail would be published on first push. | Medium | Owner decision on publishing now vs private first. |

## 5. UI/UX implementation assessment

- Tokens match the agreed palette (`#0B1118`, `#2DD4BF`, etc.); a light theme exists with darker accent text for contrast. Good foundation.
- Present: focus-visible styling, `.sr` utility, diff, dialog, drawer, popup, queue-item, toast components.
- Missing: any screen. Review Center (the signature experience), the 8 navigation pages, onboarding, extension popup states, uninspected-override flow (OPEN-05), reduced-motion rules (not found in CSS), contrast verification (light and dark never measured), responsive breakpoints.
- Cannot compare against Figma (no access). Treat design as unreviewed.

## 6. Integration coverage assessment

| Integration | Status |
|---|---|
| Chrome Native Messaging | Planned (spike S0.6 not run) |
| ChatGPT / Claude / Gemini adapters | Planned; tier per spike S0.7 (not run). No claim of protection. |
| OpenAI-compatible gateway | Planned (contracts drafted in 03) |
| Ollama | Planned |
| OpenAI / compatible / OpenRouter | Planned |
| Credential vault, SQLite/SQLCipher | Planned; unproven |

All are **Planned**. None is Implemented, Tested or Protected.

## 7. QA and security testing gap analysis

Missing: test matrix file, synthetic corpora (S0.4), mock cloud / network-deny harness (S0.3), canary scanner, CI, fuzz targets, accuracy/false-positive benchmarks, performance baselines, accessibility audit plan, UI tests. The architecture's test design (canary scan of DB+logs, "zero outbound" harness, monotonicity property tests) is good; none exists yet. No success-metric targets are defined numerically except secret recall ≥99% on a corpus (S1.4).

## 8. Prioritised GitHub backlog (proposed, not created)

Issues have not been created on GitHub. Proposed first batch:

1. **P0** Canonical acceptance table AT-01…18 with requirement IDs (G-1).
2. **P0** PRD v1 with FR/NFR IDs (G-3).
3. **P0** Decide v1.0 OS scope; ADR-0020 (G-2).
4. **P0** S0.1 monorepo scaffold, LICENSE (Apache-2.0 per OPEN-08), NOTICE, CI.
5. **P0** S0.3 testkit (mock cloud, network-deny) and S0.4 corpora.
6. **P0** Spikes S0.5, S0.6, S0.7 (parallel).
7. **P0** Test matrix and integration matrix files.
8. **P0** M1 slice: S1.1–S1.8 (secret blocked, mock cloud sees 0 requests).
9. **P1** Review Center design (S0.8, S6.2) and onboarding.
10. **P1** Reclassify Ollama stories (G-4).

Milestones: M0–M7 as in backlog.

## 9. Recommended next milestone

**M0 + M1**, in this order: (a) owner decisions G-1/G-2/G-10, (b) scaffold repo and CI, (c) three spikes in parallel, (d) Slice 1: `curl` with an API key in the prompt gets an OpenAI-shaped block error while a counting mock cloud receives zero requests, with metadata-only audit proven by a canary scan. That is the smallest demonstrable proof of the core promise.

## 10. Decisions needed from the product owner

1. **v1.0 OS scope** (G-2): Windows-only, or Windows+macOS+Linux?
2. **Publish to the public repo now?** (G-10) and which license-ready docs.
3. **Acceptance suite:** confirm the 18 handoff tests as canonical (G-1).
4. Confirm gateway-first sequencing (G-5) and the 30-day pseudonym cap (G-7).
