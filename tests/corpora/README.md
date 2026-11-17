# Aegis synthetic detection corpora (S0.4)

Versioned, machine-readable, **entirely synthetic** test corpora for the detectors (S1.4 secrets, S2.1 PII, S2.4 preservation,
S4.2 injection) and for the obfuscation/defence-context work. Format and label rules: [LABELLING_SPEC.md](LABELLING_SPEC.md).

```
tests/corpora/
  data/*.jsonl         the corpora (generated; do not hand-edit)
  MANIFEST.json        version, seed, SHA-256 per file, counts per category/subtype/action
  generate.py + cgen_* generator (Python 3 stdlib only, fixed seed)
  LABELLING_SPEC.md    schema, span units, severity/action conventions, tags
```

## Regenerate / verify

```
python -I tests/corpora/generate.py           # rewrite data/ and MANIFEST.json
python -I tests/corpora/generate.py --check   # regenerate in memory, byte-compare with disk, run validation
```

Each corpus has its own `random.Random("aegis-s0.4-corpora-v1:<corpus>")`; no clock, no environment, no hash-order dependence.
Validation asserts: unique ids, known categories/severities/actions, label spans on UTF-8 boundaries and equal in byte and
character coordinates, `parts` join equals `text`, no review/block record without labels, and (preservation) every must-preserve
token is in the acceptable rewrite while every violating rewrite breaks one.

## Counts (v1.0.0, 1003 records)

| file | records | hard negatives | labels by category |
|---|---|---|---|
| secrets_positive | 201 | 0 | credential_secret 213 |
| secrets_negative | 61 | 52 (+9 profile-twin/near-miss blocks) | credential_secret 9 |
| pii_positive | 198 | 0 | identity 184, financial 54 |
| pii_negative | 50 | 50 | none |
| other_categories | 40 | 8 | technical 20, confidential 8, identity 8, personal_record 8, contextual_identity 6 |
| obfuscation | 155 | 8 | credential_secret 151, identity 18, financial 8 |
| injection | 144 | 16 | prompt_injection 128 |
| defence | 90 | 16 | defence 74 |
| preservation | 64 | 0 | identity 88, financial 8 (plus must-preserve items) |

Secrets: 38 provider formats x 4 variants (30 flagged `top30` for the S1.4 gate, 8 extras), PEM (7 types), JWT, DB URLs,
password assignments. Per-subtype and per-action counts are in `MANIFEST.json`.

## How the values are guaranteed synthetic

* **Secrets**: built from the PRNG in the provider's format (prefix, length, alphabet). Nothing is copied from any real key,
  leak, scanner rule fixture or provider example. PEM bodies are base64 of `AEGIS-SYNTHETIC-NOT-A-KEY:` plus random bytes (not a
  valid key structure); JWT header/payload are valid JSON with invented claims and a random signature; DB URLs use hosts under
  `.example.test`. The only non-random credentials are the two AWS **documentation** example access key ids and the AWS
  documentation example secret, used solely for the OPEN-06 exemption cases. No Stripe test keys or other provider-documented
  values are used. Random strings cannot be proven non-existent, but the collision probability with any issued key is negligible.
* **PII**: names are syllable-generated (a collision with a real person can only be accidental); addresses use fictional towns;
  emails use reserved domains (`example.com/.org/.net`, `.test`); phones use ranges reserved for fiction where public ones exist
  (US/CA 555-01xx, UK 07700 900xxx and 020 7946 0xxx, AU 0491 570 xxx, FR 06 39 98 xx xx) and otherwise random digits.
  National IDs are generated with valid checksums but never taken from a source: US SSN uses never-issued area 9xx, UK NINO the
  unallocated prefix `QQ`, Canadian SIN a leading 0; Aadhaar (Verhoeff), DNI (mod 23), BSN (11-test), CPF, NRIC, INSEE (mod 97),
  personnummer (Luhn) are random digits plus the correct check, so a coincidental match with a real number is possible in principle
  (tagged `random_number`). Cards are the publicly documented test PANs (Luhn-valid). IBANs are random BBANs with correct mod-97
  check digits. One preservation template uses the widely published IBAN documentation example.
* **Defence**: fictional states (Valmora, Kesterland, Orrin), units, operations and equipment; no real units, installations or
  documents. **Injection** carriers and payloads are invented; URLs use `.example.test`.
* **No real prompts or conversations** are included; all texts are templated or hand-written for the corpus.

## Caveats for consumers

* Secret-scanning tools (including GitHub push protection) may flag `secrets_*.jsonl` and `obfuscation.jsonl` because the values
  match provider formats by design. Allow-list `tests/corpora/data/` rather than weakening the rules, and never treat these
  files as evidence of a leak.
* Four variants per format is enough to catch rule mistakes but too few to evidence a statistical 99% recall claim; regenerate
  with larger counts (raise the loop sizes in `cgen_secrets.py`) for the gate and bump the minor version.
* `unissued_range` ID records are valid in structure only; skip them if a detector deliberately rejects unissued ranges.
* `expected_action` reflects default-profile conventions in LABELLING_SPEC, not a policy decision; the policy crate owns the table.
