# Labelling spec (corpora v1.0.0)

Data files are JSON Lines (`data/*.jsonl`), UTF-8, LF, one record per line, ASCII-escaped (non-ASCII characters such as
zero-width and full-width forms appear as `\uXXXX` so they are visible in diffs).

## Record

| field | type | meaning |
|---|---|---|
| `id` | string | stable id, `<prefix>-NNNN` (never reused within a version) |
| `corpus_version` | string | semver of the corpora set |
| `corpus` | string | file stem, e.g. `secrets_positive` |
| `text` | string | the content under test (for multi-message records: `parts` joined with `\n`) |
| `labels` | array | ground-truth findings, may be empty (hard negative) |
| `expected_action` | `allow\|notify\|review\|block` | expected policy outcome (see below) |
| `notes` | string | why the record exists / how it was made |
| optional | | `profile`, `tags`, `parts`, `source`, `carrier`, `provenance`, `canonical` (in label), `difficulty`, `pii_type`, `region`, `preservation`, ... |

### Label

`{category, subtype, start, end, severity, char_start, char_end, ...extras}`

* `category` is the `aegis-core` `Category` snake_case name (`identity`, `credential_secret`, `financial`, `confidential`,
  `technical`, `personal_record`, `contextual_identity`, `defence`) plus ONE corpus-local value, `prompt_injection`, which is not
  in taxonomy v1 (inbound detection, S4.2). Map or ignore it as needed.
* `severity` is `aegis-core` `Severity` snake_case (`info|low|medium|high|critical`).
* `start`/`end` are **UTF-8 byte offsets into `text`, end exclusive** (the same unit as `aegis-core` `Span`), `part_index = 0`.
  `char_start`/`char_end` give the same span in Unicode code points. For `parts` records, offsets are into the joined text.
* `subtype` is a detector-style code (`openai_key`, `email`, `iban`, ...). Subtype names for secrets follow
  `aegis-detect-secrets` where a rule exists; new formats use the obvious `<provider>_<kind>` name.
* Span convention: provider keys, JWTs, JWT-like values, card/IBAN/ID/phone/email: the value itself. PEM: the whole block
  (or the whole truncated fragment). Credentialed connection strings: the whole URL. `password_assignment`: the value only.
  Obfuscated values: the obfuscated surface as it appears in `text`; `label.canonical` holds the decoded value
  (compare ignoring whitespace) and `label.obfuscation` names the transform.
* Split values: each fragment is its own label with the same `subtype`, `split_group` and a `fragment` number. Detectors are
  expected to reassemble using conversation/field context; scoring treats the group as found if every fragment is covered.
* Matching rule for scoring (suggested): a predicted finding matches a label when the category and subtype family agree and the
  spans overlap by at least 50% of the shorter span. Severity is scored separately.

### Severity and `expected_action` conventions

`expected_action` is the outcome of the **default** policy (profile `default`) unless `profile` says otherwise. It is the max
over labels using: `critical -> block`, `high -> review`, `medium -> review`, `low -> notify`, `info/none -> allow`
(PRD FR-POL-1). Corpus-assigned severities, advisory only (the policy engine owns the real table):

| type | severity | type | severity |
|---|---|---|---|
| any credential/secret | critical | email, person_name | low |
| national ID, card number, IBAN | high | phone, address, date_of_birth | medium |
| personal_record | high | contextual_identity, confidential | medium |
| technical (internal host/private IP) | low | defence (operational, restricted, infra, personnel, technology) | medium |
| defence borderline | low | prompt_injection | medium or high by payload |

Defence records are **flagged for review, never auto-classified** (FR-DET-5): `needs_human_review=true`, `auto_classify=false`.
Subject matter alone is not severity; general discussion is a negative (`allow`).

### Profiles and OPEN-06

`profile` (secrets_negative only) is `coding` or `default`. Exact well-known documented dummy values (the AWS documentation
example keys) are `allow` under `coding` and `block` under `default` (twin records share text). Near-miss "test key" records
(`tags: dummy_key_not_exact, must_block_in_coding`) must still block under `coding`: there is no entropy, comment or free-text
exemption.

### Tags worth filtering on

`hard_negative`, `unissued_range` (SSN 9xx, UK NINO `QQ`, SIN leading 0: structurally valid, never issued; detectors that
reject unissued ranges should skip these), `reserved_fictional` (phone numbers from ranges reserved for fiction),
`random_number` (random digits that merely look plausible), `documented_test_pan`, `stretch` (known-hard goals, e.g. Cyrillic
homoglyphs), `context_bound` (value only identifiable by its keyword context, e.g. AWS secret key, Azure key, Cloudflare token),
`top30` (provider formats counted toward the S1.4 "top 30" gate; `top30: false` rows are extras).

## Per-corpus notes

* **secrets_positive**: 38 provider formats x 4 contexts (first 30 flagged `top30`), 7 PEM types x 2 + 3 truncated, 6 JWT,
  10 credentialed DB URLs, 10 password assignments, 6 multi-secret messages.
* **secrets_negative**: hashes, UUIDs, SRI/base64 non-secrets, placeholders, public keys/certs, OPEN-06 exemptions and twins.
* **pii_positive / pii_negative**: identity + financial; negatives are checksum-invalid or structure-invalid lookalikes.
* **other_categories**: confidential, technical, personal_record, contextual_identity (small sets plus negatives).
* **obfuscation**: base64, base64url, hex, double base64, percent (full/partial), zero-width (single/mixed), full-width,
  HTML entities, spaced, line-wrapped, Cyrillic homoglyph; PII variants; splits across messages/fields/code.
* **injection**: `source` is `user_upload | tool_result | model_response | user_prompt`. Positives are `review`; they must be
  flagged and must never change policy or tool behaviour (AT-10). `policy_override_attempt=true` marks payloads that try to
  alter Aegis/policy. Hard negatives include benign imperatives and security-article quotations.
* **defence**: five context subtypes (`operational_detail`, `restricted_document`, `infrastructure`, `personnel`, `technology`),
  a borderline set (`notify`), and general-discussion negatives. All fictional.
* **preservation**: `preservation.must_preserve` items `{kind, text, start, end}` (kinds: number, date, negation, exception,
  comparator; offsets in original `text`), `acceptable_rewrite` (PII replaced by placeholders, everything else verbatim) and
  `violating_rewrites` each breaking one item (`violates` = index). A checker passes the acceptable rewrite and fails every
  violating one.

## Versioning

`corpus_version` follows semver. Adding records appended to the end of a file or new files = minor; changing/removing a record
or changing label semantics = major. ids are generated sequentially, so any generator change that alters earlier records is a
major bump. `MANIFEST.json` carries per-file SHA-256, a hash of the generator sources, and counts.
