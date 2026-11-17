# Engineering build handoff (architect session)

**Written 2026-11-17.** Everything needed to continue the build without the original session. Facts only; "verified" names the environment. Rules from the owner and PM are at the bottom.

## 1. Where the code is

| Branch (origin) | What it holds | Status |
|---|---|---|
| `main` (f75d7f2 at last check; 9b6b879 was the M1 merge) | M0 scaffold, M1 gateway slice, security-review fixes, design/QA/PRD docs | CI green on windows-latest, ubuntu-22.04, macos-latest, msrv 1.90, cargo-deny, no-outbound-sockets (reported by PM, unpinned deps) |
| `pr/approval-engine` (08e0b95) | `aegis-approval`: Review state machine + HMAC hash-bound single-use tokens | CI green (reported by PM). Not wired into gateway/egress |
| `docs-architect-sync` | S0.3 testkit, S2.1 PII detectors, AT-19 row, S3.6a, contract v1.1 (ADR-0023), `architect-status.md` | Built on the OLD m0-m1 history: **must be rebased onto origin/main** (conflicts expected in `Cargo.toml` tokio features and docs) |
| `s2-1-pii-detectors`, `s0-3-testkit` | the same commits individually | superseded by the rebase above |
| `fix/secret-recall` (this branch) | provider-format rules, placeholder filtering, spaced/line-wrapped recovery, corpus metrics test | see section 3 |

Earlier branches `s2-3-approval-engine`, `m0-m1-scaffold`, `fix/m1-security-review` are obsolete or merged; do not push to them. `s2-1b-corpus-metrics` failed CI because its `Cargo.lock` was not updated after adding dev-dependencies (the msrv job uses `--locked`); its content is now in `fix/secret-recall` with the lock fixed. **Always run `cargo metadata --locked` after changing any `Cargo.toml` and commit `Cargo.lock`.** The canonical resume file is `docs/status/BUILD-STATUS.md` on main; keep this note consistent with it.

## 2. Building locally on this Windows machine

No MSVC and no `dlltool` here, so the default build fails. CI is the truth. For local checks only, build a throwaway copy with older versions (never commit the pins); see `docs/dev/local-build-without-msvc.md`. Recipe used: copy the repo (excluding `.git`, `target`, `Cargo.lock`), set `tokio = "=1.43.1"`, `axum = "=0.8.1"`, `tower = "=0.5.2"`, run `cargo generate-lockfile` then `cargo update -p mio --precise 1.0.3`, then `cargo +stable-x86_64-pc-windows-gnu fmt --check`, `clippy --workspace --all-targets -- -D warnings`, `test --workspace`. Rust 1.100 is installed under `%USERPROFILE%\.cargo`.

Corpora: `python -I tests/corpora/generate.py` (branch `s0-4-corpora`, owned by the PM; data is generated, never committed). Run the metrics test with `AEGIS_CORPORA_DIR=<corpora>/data cargo test -p aegis-inspect --test corpus_secrets -- --nocapture`.

## 3. Detection recall: measured results (corpora v1.0.0, record level, pinned scratch build, not CI)

| Measure | Before | Now (this branch) | PRD target |
|---|---|---|---|
| secrets_positive (plain forms) | 172/201 (85.6%) | **201/201** | >= 99% top-30 |
| obfuscation, credential records | 69/123 (56%) | **103/123 (83.7%)** | set a separate threshold |
| secrets_negative expected allow (default profile) | 7/52 false positives | **0/49** (3 coding-profile twins excluded) | 0 |
| pii_negative secret false positives | 0/50 | 0/50 | 0 |

Still missed (20 obfuscation records): HTTP `Authorization: Basic <base64 user:pass>` (add a header rule, and a Bearer-token rule), string concatenation in code (`"ghp_a" + "b"`: add a literal-joining view), secrets split across messages (parts are `{role,text}` objects; reassembly across prose is a known limit and is reported, not claimed). Four variants per format is a small sample: raise the generator's per-format count before claiming 99%.
Not implemented: the OPEN-06 exact-dummy-key exemption (profiles arrive with M6); the corpus metrics test is opt-in and CI does not yet generate the corpus or gate on recall.

## 4. What remains, in priority order

1. **Detection (PM priority 1):** Basic/Bearer header rules; concat view; per-format variant count; CI step that generates the corpus and fails below thresholds (`AEGIS_MIN_PLAIN_RECALL`, `AEGIS_MIN_OBFUSCATION_RECALL`, `AEGIS_MAX_SECRET_FALSE_POSITIVES` are already read by the test; guard the CI step on `tests/corpora/generate.py` existing). Then the PII corpus against `aegis-detect-pii` (needs the rebase in section 1).
2. **Rebase and push** `docs-architect-sync` content and `pr/approval-engine` onto origin/main, one push each, wait for all-3-OS CI before pushing again.
3. **Wire approval engine:** gateway Review path to `Review`, egress crate to `Verifier` (token required to send), daemon supplies OS-random Minter key bytes (needs a randomness source that builds on all targets), reinspection loop, hold/timeout of API requests. Maps to AT-05, AT-06, AT-09.
4. **S2.4** deterministic transform + preservation checker; original-coordinate spans for decoded findings (Review Center contract 10.10); **S2.9** reason catalogue.
5. **S3.1/S3.2** (P0) Ollama adapter + router; **S3.6a** guard-separation startup check (AT-19, ADR-0022).
6. Spikes: **S0.5** SQLCipher + OS key stores on 3 OSes (needs a C compiler; AT-17 stays not passed until then), **S0.6** Native Messaging, **S0.7** site interception, **S0.3b** OS-level network-deny (AT-07/08 are only PARTIAL until then).
7. M4+ (inbound inspection, vault, extension, dashboard) per `docs/architecture/04-backlog.md`.

## 5. Known gaps and deviations (do not hide these)

- AT-17 passes only against the temporary JSONL audit stand-in; not passed until SQLCipher.
- AT-07/AT-08 partial: mock/tripwire plus structural tests, no OS-level network-deny.
- API keys use unkeyed SHA-256 (ADR-0010 says HMAC/Argon2id): mitigated by key-strength check and throttle; keyed hashing needs OS randomness.
- Windows audit-file ACL not hardened (0600 applied on Unix only); no header-read timeout; ROT13/reversal/compression/prose-split secrets are accepted limits.
- Review Center contract: finding locations are in view coordinates, not original text, until S2.4.
- Not verified anywhere: macOS/Linux for `fix/secret-recall`, the Unix-only 0600 audit test, unpinned build of this branch.

## 6. Rules in force

- Owner: the main repo lives on GitHub because local files and sessions on this PC will be deleted; push every checkpoint to a branch.
- Identity: commit as `bigdee-vibin <bigdee-vibin@users.noreply.github.com>` (the owner's personal email must not appear in public history); sign off with `-s`.
- Never force-push `main`; open PRs only through the PM or the compare link; one push, then wait for CI to finish; no secrets, real prompts or dev keys in commits.
- Security Highs first; report only verified results (command, OS, commit); do not claim M1/AT results until CI is green on all three OSes.
- Do not edit `docs/qa/` (QA agent) or renumber ADRs (ADR-0022 is taken; ADR-0023 is contract v1.1).
