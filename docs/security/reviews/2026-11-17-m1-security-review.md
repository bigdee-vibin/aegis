# Security review: M1 gateway slice (branch m0-m1-scaffold @ c2d6147)

**Method:** independent read-only static review by an AI reviewer agent. Nothing was compiled or run; every reproduction is by reading and UNTESTED. Tracking: each finding needs a fix commit plus a regression test, then QA verification.

| # | Sev | Finding | Location | Fix direction |
|---|---|---|---|---|
| 1 | HIGH | Decoded views silently dropped when view cap/byte budget hit, so detection fails open (22+ decoy base64 tokens before a real key, or >256 KiB decodable tokens) | aegis-inspect/src/normalise.rs:654-673; inspector.rs:383-396 | Truncation must set stage1_failed (block) or mark content uninspected; test the cap fails closed |
| 2 | HIGH | password/secret/api-key rules miss UPPER_SNAKE and prefixed names (DB_PASSWORD=, AWS_SECRET_ACCESS_KEY=, GITHUB_TOKEN=) because `\b` and `_` are word chars; no AWS secret-access-key rule | aegis-detect-secrets/src/lib.rs:134 | Drop leading `\b`, allow prefixes/suffixes, add AWS secret rule + entropy fallback |
| 3 | MED | Invisible chars surviving NFKC not stripped (U+034F, variation selectors, tag chars, Hangul fillers, U+061C...); confusables not folded | normalise.rs:526-548 | Strip Cf/Mn/Me and default-ignorables; fold confusables |
| 4 | MED | No cross-part / cross-message joins, so split secrets pass (threat-model T15 claims mitigation) | inspector.rs:381-400; validate.rs:314-338 | Scan concatenations of adjacent parts; or document the limit |
| 5 | MED | Base64 decoder misaligned by glued prefixes, defeated by whitespace/line wrap; >=10% non-printable decodes discarded | normalise.rs:518,556-560,608-617 | Try alignment offsets, join wrapped runs, treat undecodable blobs as uninspected |
| 6 | MED | Hex with separators, base32, HTML entities, ROT13 etc. not handled | normalise.rs:605-629 | Add base32/entities; document accepted limits |
| 7 | MED | Rules anchored with `\b` miss when adjacent to word chars (x_sk-ant-..., AKIA...EXAMPLEfoo) | detect-secrets/src/lib.rs:43-120 | Remove left/right `\b`, use explicit lengths |
| 8 | LOW | plausible_secret_value rejects real passwords (leading $ * %, containing ()[]) | lib.rs:143-160 | Allow unless placeholder/template form |
| 9 | LOW | API key: unkeyed SHA-256 (ADR-0010 requires HMAC/Argon2id), env var of 16 chars accepted, no throttling | gateway/src/auth.rs:23-41; daemon/src/main.rs:158-186 | Keyed hash, entropy check, failed-auth throttle |
| 10 | LOW | Auth failures, 413, invalid JSON not audited | gateway/src/lib.rs:123-168 | Audit rejects (no key material) |
| 11 | LOW | Audit file: no fsync, default permissions, blocking write inside async handler | audit/src/lib.rs:125-139 | sync_data, 0600/ACL, spawn_blocking |
| 12 | LOW | Only 1 MiB cap: no timeout (FailureCode::Timeout never produced), no header/read timeout, no concurrency limit | gateway/src/lib.rs:129-134; daemon/src/main.rs:192 | Fail-closed timeout, limits |
| 13 | LOW | Content validation looser than I8/ADR-0013 (media fields beside type:text, stream:"true", previous_response_id/store not rejected) | validate.rs:289-307 | Per-part key allowlist; reject non-bool stream |
| 14 | LOW | deny.toml unverified/incomplete; sockets possible via std::net/tokio; CI actions pinned by mutable tag; no MSRV job | deny.toml; .github/workflows/ci.yml | Forbidden-API lint, pin by SHA, MSRV job |

**Checked, no issue:** no unsafe code; detector panics and empty detector set fail closed; policy pure/monotonic; gateway ordering (auth, size, parse, validate, inspect, policy, audit, upstream); all strings and keys inspected; constant-time key compare; audit events typed with no free text; loopback-only bind; no egress code in M1.
**Not assessed:** compilation, tests, clippy, cargo-deny, RustSec advisories, detector recall on corpora, Windows ACL/named pipes, ADRs for code not yet written.
