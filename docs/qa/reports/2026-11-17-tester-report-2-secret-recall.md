# Tester report 2: independent measurement of secret detection (main @ 97c00b7, corpora 1.0.0)

Independent AI tester. Pinned scratch build on Windows (GNU toolchain; tokio 1.43.1, axum 0.8.1, mio 1.0.3), not CI-equivalent. fmt/clippy, macOS, Linux, the gateway end to end and profile logic were NOT run. All secrets are invented.

## Claim check (author's numbers reproduced exactly)
| Claim | Author | Tester |
|---|---|---|
| secrets_positive | 201/201 | 201/201 |
| obfuscation credential records | 103/123 | 103/123 |
| secrets_negative expected-allow (default profile) | 0/49 FP | 0/49 |
| pii_negative secret FPs | 0/50 | 0/50 |
| workspace tests | 76 | 76 pass |
Caveat: these figures were measured on the same corpus the fixes were tuned on (training-set recall). Overall record-level credential recall by tester's count: 93.8% ((201+103)/324).

## Unscored corpus records
- sec-neg-0055 (bare AWS secret key, no label, expected block) is a MISS the metrics test never counts.
- sec-neg-0050 and 0052 (documented AWS example keys) are flagged under the coding profile: the OPEN-06 exemption is not implemented (known gap).

## Missed obfuscation records (20)
Basic-auth base64 header (3), keys split across messages (9), JSON prefix/suffix halves (4), string concatenation (4).

## New defects found with ~180 new synthetic cases (80/153 positive-intent cases detected; 6/27 benign controls flagged)
| # | Weakness | Severity |
|---|---|---|
| D1 | NUL, BS, DEL, combining marks (U+0301), NBSP, thin and ideographic spaces mid-key bypass (interleaved NULs too); 14 other invisibles are handled | Medium |
| D2 | Markdown-escaped keys (`ghp\_...`, `sk\-ant\-...`) | Medium |
| D3 | Password inside JSON-escaped string (tool-call arguments) missed | Medium |
| D4 | `Authorization: Bearer <opaque>` / `Basic <b64>` missed (known) | Medium |
| D5 | Assignment names too narrow: `DB_PASS=`, `pw:`, "my password is X", `mysql -p`, `sshpass -p`, XML password tags, `psk=`, `IDENTIFIED BY` | Medium |
| D6 | Value rule needs >= 8 chars and stops at whitespace/`;`/`,`/quote: passphrases with spaces, 7-char passwords | Medium-Low |
| D7 | Connection strings: empty-user Redis `redis://:PW@host` missed; `@` or `/` inside password | Medium (Redis) |
| D8 | Split variants: tab/double-space, backslash-newline, hyphen-newline, backtick, separator chars, JSON fields, prose | Low-Medium |
| D9 | Other encodings: `\uXXXX`, octal, decimal CSV, binary, quoted-printable, Docker `auth` base64; reversal/ROT13 accepted limits | Low |
| D10 | Case variants (`akia`, `GHP_`, `SK-`), lowercase/spaced PEM header, PEM body without header | Low |
| D11 | `sk-` rule needs a non-alphanumeric left of it (glued prefix missed) | Low |
| D12 | Triple/quadruple nested base64 fail closed (block); possible false blocks, unmeasured | Info |
| D13 | No rule: Stripe `sk_test_`, 19-char OpenAI-style body, bare 32-hex key | Info |
False positives (Low-Medium): URL slugs and words starting `sk-` (e.g. `sk-introduction-to-...`), `token = getAuthenticationTokenFromStorage()`, `token: refreshInterval=30000ms`, `secret: santa_gift_exchange_2024_list`; prose containing a PEM header (arguably correct).

## Good signs
Keys in JSON arrays, URL query strings, YAML/env files, MIME-wrapped and offset base64, hex/base32/percent/HTML entities, 14 invisible characters, Greek/math/circled homoglyphs, Unicode dashes; fail-closed at 16 MB (0.8 s release build).
