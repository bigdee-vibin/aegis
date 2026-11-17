# Architect status (engineering)

**As of 2026-11-17.** Facts only; "verified" means a command was run, with the environment stated. Nothing here claims CI results: CI results come from the PM session.

## Branches (all pushed to origin; no force-push to main)

| Branch | Contents | Base |
|---|---|---|
| `m0-m1-scaffold` | M0 workspace + docs + Review Center contract v1 + M1 gateway/secret scanner/audit (PR #3) | origin/main |
| `s0-3-testkit` | S0.3 mock cloud/Ollama servers, connection tripwire, test HTTP client | m0-m1-scaffold |
| `s2-1-pii-detectors` | S2.1 structured PII detectors | s0-3-testkit |
| `docs-architect-sync` | AT-19, S3.6a, contract v1.1, ADR-0023, this note | s2-1-pii-detectors |

## Done and verified locally

Environment: Windows 11, Rust 1.100, `x86_64-pc-windows-gnu`, scratch copy with older tokio 1.43.1 / axum 0.8.1 / mio 1.0.3 pins (this machine has no `dlltool`/MSVC).
- `cargo test --workspace`: 71 passed, 0 failed (at `s2-1-pii-detectors`). `clippy -D warnings` and `fmt --check` clean.
- M1 slice 1: key in prompt blocked before upstream (mock count 0), obfuscated secrets blocked, fail-closed on detector failure / audit failure, audit file canary-clean (JSONL stand-in).
- S2.1: email, phone, card (Luhn), IBAN (mod 97), SSN, Aadhaar (Verhoeff), PAN; inline test cases only.

## Not done / not verified

- CI on Windows (MSVC), macOS, Linux; `cargo-deny`; unpinned dependency build. Results must come from the PM session.
- AT-17 stays **not passed** until the SQLCipher store (S0.5). AT-07/08 are **partial** (tripwire + structural only; no OS-level network-deny, S0.3b).
- S0.5, S0.6, S0.7 spikes: not started (no C compiler here for SQLCipher; browser/OS spikes need other OSes).
- Recall/precision of detectors: not measured; the shared S0.4 corpus is not yet consumed.
- PII detectors are not wired into the daemon (without an approval engine, Review-level findings fail closed).
- Approval engine, egress token, transform/preservation (S2.3–S2.7, S2.4): not started.
- Original-coordinate spans for decoded findings (needed by the Review Center contract): not done.

## Next (in order)

1. Approval state machine + hash-bound token + reinspection + rejection/timeout (S2.3/2.5/2.6/2.7).
2. Transform + preservation checker (S2.4).
3. Guard-separation check (S3.6a, AT-19).
4. Consume S0.4 corpora for detector metrics.
