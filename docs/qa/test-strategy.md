# Aegis Test Strategy (v0.1 draft)

**Status:** Draft for QA and owner review. **Sources:** PRD v1.0 draft 1, backlog (M0 to M7), threat model. Targets marked [R] are PM recommendations pending baseline, as in the PRD. Nothing here is verified; no effectiveness claim is made until measured (PRD NFR-8).

Companion documents: [test-matrix.md](test-matrix.md), [integration-coverage-matrix.md](integration-coverage-matrix.md).

## 1. Objectives

1. Prove the privacy-critical invariants hold: no critical secret leaves the device, no silent transformation, no silent cloud fallback, no content persisted, egress only of approved bytes.
2. Measure detection quality honestly and publish results, including misses.
3. Verify that coverage labels shown to users (Protected / Partially protected / Not protected) match what tests demonstrate.
4. Verify behaviour on Windows, macOS and Linux (ADR-0020).

## 2. Test layers

| Layer | Scope | Examples | Gate |
|---|---|---|---|
| L1 Unit | Detectors, checksum validators, normaliser, policy tables, approval state machine | Luhn/IBAN, key formats, decision tables | Every commit |
| L2 Property | Invariants over generated inputs | Policy monotonicity and layering (profiles cannot relax mandatory rules), Stage 2 can only raise severity, approval hash binding | Every commit |
| L3 Component and contract | Crate/service boundaries; canonical schema and golden fixtures; provider conformance against recorded fixtures | Chat and Responses fixtures round-trip (S0.2), adapter conformance (S3.5) | Every commit |
| L4 Integration | Multi-component flows with mocks from `aegis-testkit`: counting mock cloud, mock Ollama, canary scanner | Gateway to policy to egress; vault and SQLCipher round trips; NM relay to daemon | Every PR |
| L5 End-to-end and acceptance | AT-01..AT-18 as one `tests/acceptance` suite | Key in prompt blocked with cloud count 0 | Every PR for implemented ATs; all 18 must pass on three OSes before 1.0 |
| L6 Security and adversarial | Fuzzing, adversarial corpora, fault injection, network-deny, canary scans | cargo-fuzz of parsers, IPC and NM framing; obfuscation and injection corpora with regression thresholds | Nightly and per release |
| L7 Browser and adapter | Fixture tests plus live canaries per site adapter; extension behaviour with companion failure | Adapter tier verification | Fixtures per PR; live canary scheduled |
| L8 Performance and reliability | Latency, load, soak, restart recovery | Stage 1 p95 target [R] 300 ms on up to 4k tokens | Nightly and per release |
| L9 UX and accessibility | Automated a11y, manual WCAG 2.2 AA audit, keyboard, screen reader, reduced motion, moderated usability | Review Center | Per milestone (M6) and release |
| L10 External | Penetration test and crypto review (S7.5) | Scope: gateway, NM, vault, egress | Before 1.0 |

Principle: invariant tests (L2, L5, L6) are the priority. Features that cannot be shown to preserve invariants are not shipped.

## 3. Environments

| Environment | Windows | macOS | Linux |
|---|---|---|---|
| CI runners | Windows 11 or Server current | macOS current and previous major (TBD) | Ubuntu LTS current (TBD, add Fedora if resources allow) |
| Key store under test | DPAPI | Keychain | Secret Service (GNOME Keyring/KWallet) and no-Secret-Service master-password mode |
| Network-deny harness (S0.3) | WFP or in-process socket guard | pf or in-process socket guard | netns or in-process socket guard |
| Browsers (Native Messaging, extension) | Chrome, Edge, Brave | Chrome, Edge, Brave | Chrome, Edge (where available), Brave |
| Ollama | Local install pinned version (TBD) | Same | Same |
| Install and uninstall | Per-OS installer, signed (M7) | Notarised package | Package format TBD |

Notes:
- Matrix of OS x browser x key store is large; the minimum release matrix is every OS with Chrome and each key-store mode, plus Edge and Brave for Native Messaging install and uninstall (S5.1).
- Headless Linux CI must exercise the no-Secret-Service path explicitly (NFR-5).
- Mocks stand in for cloud providers in CI. Live provider and live site tests are limited to scheduled canaries with synthetic data and dedicated test accounts, never real personal data.
- Tests that need real clouds must be opt-in, tagged, and run only on credentials created for testing.

## 4. Test data and corpora

All corpora are synthetic. No real personal data, real credentials or real defence material may be used or committed (backlog S0.4). Each corpus has a labelling specification, documented generation method, version and checksum.

| Corpus | Contents | Used by | Owner/status |
|---|---|---|---|
| Secrets | Top 30 provider key formats, PEM and private keys, JWTs, dummy and well-known test keys, near-miss lookalikes, high-entropy non-secrets | FT-DET-01/07, ST-LEAK-01, RT-ADV-01 | Needed at M0/M1 |
| Structured PII | Cards (Luhn valid/invalid), IBANs, national IDs by region with checksums, emails, phones, names, addresses; multi-region | FT-DET-02, AT-02, RT-ADV-01 | Needed at M2 |
| Contextual | Documents and conversations where sensitivity depends on context; English first (FR-DET-4) | FT-DET-05, FT-CTX-03 | Needed at M3 |
| Defence-related | Synthetic operational, personnel, infrastructure, technology and restricted-document style text; general military discussion as negative class; no real classified or operational information | FT-DET-06, PT-DEF-01 | Needed at M4; PM to define labelling |
| Prompt injection | Direct, indirect (documents, retrieved data, tool results), policy-override attempts, exfiltration links | AT-10, ST-INJ-01/02 | Needed at M4 |
| Obfuscation and evasion | Base64, homoglyph, zero-width, split secrets, multi-part and multi-role | ST-EVA-01, ST-BYP-04 | Needed at M1 onward |
| Malicious uploads and payloads | Oversize, nested, ReDoS, compression-like, polyglot, unsupported types | ST-UPL-01..03 | Needed at M4 |
| Canary strings | Unique markers inserted into prompts, responses, uploads | ST-LEAK-02, AT-17, PT-PER-01 | Needed at M1 |
| Protocol fixtures | Recorded Chat and Responses requests, responses and SSE streams; NM message samples including malformed | FT-GW-*, IT-OAI-01, AT-15 | Needed at M0 |
| Site fixtures | Saved DOM and network shapes per adapter | IT-ADP-* | Needed at M5 |
| Semantic-preservation | Texts with numbers, dates, negations and exceptions before and after transformation | FT-CTX-01/02 | Needed at M2 |

Corpus rules: version-controlled, reviewed for accidental real data, regeneration reproducible, and metrics always report corpus version. Held-out sets are kept separate from tuning sets to avoid inflated recall claims.

## 5. Tooling

Tooling follows the architecture and backlog; where not specified it is a proposal needing sign-off. Nothing needs installing as part of this document.

| Need | Tooling (proposed unless named in backlog) |
|---|---|
| Build, unit and property tests | Cargo test, property-testing crate, golden-file fixtures (backlog S0.1, S0.2, S1.5) |
| Lint and supply chain | clippy, fmt, `cargo deny`, lockfiles, dependency-graph lint (only `aegis-egress` may pull a general HTTP client), SBOM (S0.1, S7.3) |
| Fuzzing | cargo-fuzz (S7.2) |
| Mocks and harnesses | `aegis-testkit`: request-counting mock cloud, mock Ollama, network-deny harness, canary scanner (S0.3) |
| Extension and UI | Browser automation framework (TBD), accessibility scanner (TBD), manual screen-reader sessions per OS |
| Performance | Benchmark harness in CI with stored baselines (TBD) |
| Reporting | Test matrix updated from CI results; each row links its evidence; coverage and corpus metrics published per release |
| Defect tracking | Repository issue tracker, labelled by severity (section 8) and test ID |

## 6. Entry and exit criteria

**Entry to test a feature:** requirement ID exists, acceptance criteria defined, relevant corpus exists, mocks available.

**Test-matrix hygiene:** a row is only moved from `Not run` by recording an actual result and linking evidence. Results are never entered from memory or inferred.

## 7. Release criteria (1.0)

All of the following must hold; any miss blocks release unless the owner records an explicit, reasoned waiver (S1 and S2 cannot be waived for privacy invariants).

1. AT-01..AT-18 pass on Windows, macOS and Linux in CI (PRD section 8; S7.1).
2. All P0 test-matrix rows in functional, security, privacy and integration sections have a recorded result; none Blocked or Not run.
3. Zero open S1 Critical defects; zero open S2 High defects, or each has a documented, owner-approved mitigation that does not affect a privacy invariant.
4. Measured metrics meet or have been explicitly rebaselined by the owner, and are published as measured: secret leakage in corpus 0; LOCAL ONLY violations 0 across network-deny runs; audit canary hits 0; secret recall top 30 formats [R] >= 99%; structured PII recall [R] >= 95%; Stage 1 added latency p95 [R] <= 300 ms; coverage-label accuracy 100% of declared tiers verified. Contextual detection and semantic preservation are published as measured with no promise.
5. Fuzzing: no open crashes in parsers, IPC, NM framing, normaliser (S7.2).
6. External penetration test and crypto review complete with no open High or Critical findings (S7.5).
7. Fail-closed matrix (threat model section 4) fully exercised (ST-FC-01).
8. WCAG 2.2 AA audit completed on security-decision surfaces; moderated usability results recorded for Review Center.
9. At least one site adapter at its declared tier with passing fixtures and live canary; every other site shown as Not protected or its verified tier.
10. Artefacts signed on three OSes, SBOM and NOTICE produced; install, upgrade and uninstall verified (S7.3, S7.4).
11. Honesty documentation reviewed: coverage, limits (same-user malware, unsupported sites, undetected items), provider data handling (S7.6).
12. [integration-coverage-matrix.md](integration-coverage-matrix.md) reflects evidence: no integration labelled Protected without passing evidence.

## 8. Defect severity definitions

| Severity | Definition | Examples | Response |
|---|---|---|---|
| S1 Critical | Violates a privacy or security invariant, or exposes secrets or content; or data loss of protected material | A critical secret reaches the mock cloud; silent cloud fallback in LOCAL ONLY; approved-X-send-Y succeeds; content or credentials persisted or logged; fail-open on detector or companion failure; unauthenticated or LAN-exposed gateway by default; a coverage label that claims protection that does not exist | Blocks release and merge of related work; fix immediately; add regression test; consider disclosure |
| S2 High | Major function or security control fails with no privacy invariant breach, or there is a plausible route to S1 | Review flow cannot complete; detector misses an in-scope class below target; a significant WCAG 2.2 AA failure on a decision surface; adapter tier mislabelled conservatively; crash on malformed input without leak | Blocks release unless mitigated and approved; fix in current milestone |
| S3 Medium | Function degraded or confusing, workaround exists, no security impact | Misleading but non-protective UI wording; poor explanation text; minor performance miss against [R] target | Fix before release where capacity allows; may ship with note |
| S4 Low | Cosmetic or minor inconvenience | Layout glitch, typo | Scheduled |

Severity is set by QA with security input; disputes escalate to the owner. Priority (P0/P1/P2) describes requirement importance and is separate from defect severity. Any defect in which user content or a secret could appear where it must not is at least S1 regardless of likelihood.

## 9. Regression policy

1. Every fixed defect gets an automated regression test before closing, linked to its issue; S1 and S2 regressions join the permanent acceptance or adversarial suite.
2. Detector and rule changes run the full corpora; recall or precision falling below the recorded baseline for any class fails the build. Baselines may only be lowered by owner-approved change recorded in the repository.
3. Any change to policy, egress, approval, vault, audit, gateway auth or Native Messaging triggers the full security and privacy suites (L5, L6) on all three OSes.
4. Adapter changes and scheduled live canaries: a failing canary immediately downgrades the declared tier in the manifest and UI until fixed.
5. Provider API drift: fixtures refreshed on a schedule and on provider announcements; conformance failures treated as S2 until triaged.
6. Dependency, model or rule-pack updates re-run corpora and supply-chain checks.
7. Flaky tests are quarantined only with an issue and owner; a quarantined invariant test blocks release.
8. Nightly runs: fuzzing, soak, adversarial corpora. Per-release run: the entire matrix with results recorded.
9. Test-matrix and integration-matrix updates accompany each release; stale `Not run` rows for implemented features block release.

## 10. Roles and responsibilities

| Role | Responsibility |
|---|---|
| QA lead | Owns this strategy and the matrices; severity triage; release recommendation |
| Security lead | Adversarial corpora, threat-model alignment, pen test scope |
| Feature owners | Unit, property and component tests; regression tests for their defects |
| PM | Priority, waivers escalation, requirement changes, open questions (PRD section 14) |
| UX | Usability studies, accessibility sign-off |
| Product owner | Final waiver and release decision |

## 11. Risks to testing

- Detection recall is inherently below 100% (T1); corpora cannot represent real-world variety. Mitigation: publish limits, grow adversarial corpora, avoid claims beyond measurement.
- Site adapters are fragile by nature (R-02): live canaries and honest tiering.
- Three-OS scope increases CI cost and flakiness (R-15): prioritise invariants, keep the network-deny harness reliable on all three.
- Synthetic data may miss real formats; regional PII coverage must be stated by region.
- Open decisions affect tests: Q-3 retention, Q-4 Ollama in P0 (AT-07/08 against mock or real), Q-5 LAN reconstruction, OPEN-04 and OPEN-05 behaviours. Test rows follow the PM recommendations and must be updated when decisions are confirmed.
- AT-16 depends on a post-1.0 stub (learning suggestions); it is tested against a stub at M7.
