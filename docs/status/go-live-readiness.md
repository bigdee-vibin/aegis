# Aegis v1.0 — Go-Live Readiness Tracker

**As of:** 2026-11-17 · **Verdict: NOT READY.** Only M0 and part of M1 exist. Statuses below are verified facts only; anything untested is "Not verified".

## Release gates (all must be Passed)

| # | Gate | Status | Evidence / blocker |
|---|---|---|---|
| G1 | AT-01…AT-18 pass on Windows, macOS, Linux in CI | Not passed | AT-01 passes locally on pinned scratch build only; CI never run |
| G2 | CI green on 3 OSes (build, clippy, fmt, cargo-deny, tests) | Passed once, on the merged M1 branch (cde2292: windows, ubuntu, macos-latest, msrv, cargo-deny, no-outbound-sockets) | Run on main in progress; later branches need their own runs |
| G3 | Independent tester + QA sign-off, zero open Critical/High | In progress | Tester running on PR #2/#3 |
| G4 | Detection metrics measured vs targets (PRD §11) on synthetic corpora | NOT MET | Measured 2026-11-17 on pinned local build (not CI): secrets 172/201 (85.6%), obfuscation credentials 69/123 (56%), overall credential recall 74.4% vs target >=99% top-30; 7/52 secret false positives; PII negatives 0/50 blocked. 21 provider formats missing. Fix in progress |
| G5 | Spikes S0.5/S0.6/S0.7 reported on all 3 OSes | Not started | Gate M4/M5 |
| G6 | Slices M2–M5 complete (review/approve, local+routing, inbound/vault, extension) | Not started | M1 only, partially verified |
| G7 | ≥1 site adapter at declared tier with canary; coverage labels accurate | Not started | Needs S0.7 |
| G8 | Review Center usable and WCAG 2.2 AA verified (screen reader, keyboard, focus trap) | Not verified | Design board only; SR testing not done |
| G9 | Fuzzing, external pen test, crypto review: no open High/Critical | Not started | M7 |
| G10 | Signed installers per OS, SBOM, NOTICE, reproducibility | Not started | M7 |
| G11 | Honest-coverage docs and limitations published | Not started | S7.6 |
| G12 | Privacy hygiene: no personal data in repo history, licences clean | Blocked | Owner email in 16 public commits; owner decision pending |
| G13 | Update flow opt-in verified; zero telemetry verified | Not started | S7.4 |
| G14 | Open product questions Q-1…Q-6 decided | Open | PRD §14 |

## Critical path (recommended order)
1. Close tester findings (current cycle) → fix → QA verify.
2. Make CI green on 3 OSes with unpinned deps (gate for everything).
3. M0 spikes S0.3, S0.5, S0.6, S0.7 in parallel.
4. M2 (review/transform/transmit) → M3 (Ollama, LOCAL ONLY) → M4 (inbound, injection, vault) → M5 (extension).
5. Desktop dashboard from the design board (M6), then hardening and release (M7).
Dates are not committed until spikes report.

## Rule
No go-live recommendation until G1–G14 are Passed with evidence linked here.
