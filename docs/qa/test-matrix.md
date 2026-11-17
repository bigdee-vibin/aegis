# Aegis Test Matrix (v0.1 draft)

**Status:** Draft for QA and owner review. **Source:** PRD v1.0 draft 1 (`docs/product/PRD.md`), backlog (`docs/architecture/04-backlog.md`), threat model (`docs/security/02-threat-model.md`).

## How to read this matrix

- **No results exist yet.** Every row starts with Actual result = `Not run` and Status = `Not run`. Nothing here is a claim that Aegis works or protects anything (PRD NFR-8).
- **Severity** is the defect severity that applies *if the test fails* (definitions in [test-strategy.md](test-strategy.md) section 8). It is not a result.
- **Evidence** is filled with a link to a CI run, log or artefact when a test is executed. `-` means none yet. **Issue** holds the tracker reference for any failure. `-` means none.
- **Status values:** `Not run`, `Pass`, `Fail`, `Blocked`, `Waived (with owner sign-off)`. Only `Not run` is used at creation.
- Platform scope: unless stated otherwise every row must be run on Windows, macOS and Linux (ADR-0020).
- Requirement refs use PRD IDs. Where the PRD has no requirement for an area (for example UI/UX sub-items), the nearest FR/NFR is cited and the gap is noted in the Scenario.
- Test IDs: `AT-` acceptance (PRD section 8), `FT-` functional, `ST-` security, `PT-` privacy, `IT-` integration, `RT-` reliability/performance, `UX-` UI/UX/accessibility.

## 1. Canonical acceptance suite (AT-01 to AT-18)

Scenario text is copied verbatim from PRD section 8. Expected result for each AT row is the pass condition implied by the scenario text on all three OSes.

| Test ID | Requirement ref | Scenario | Expected result | Actual result | Pass/Fail status | Severity | Evidence | Issue |
|---|---|---|---|---|---|---|---|---|
| AT-01 | FR-DET-2, FR-POL-2 | API key in prompt is blocked/removed before external transmission; counting mock cloud sees 0 requests | Request blocked or key removed; mock cloud request count = 0 | Not run | Not run | S1 Critical | - | - |
| AT-02 | FR-DET-1/3, FR-POL-1 | Personal info detected; correct profile policy applied | Findings carry category, severity, location, confidence, reason; the active profile's policy action is applied | Not run | Not run | S2 High | - | - |
| AT-03 | FR-POL-1, FR-CTX-3 | High-risk context requires review where configured | Request held for review before transmission when configured | Not run | Not run | S2 High | - | - |
| AT-04 | FR-POL-1/2 | Low-risk content follows notify-only | User notified; content unchanged; request proceeds | Not run | Not run | S3 Medium | - | - |
| AT-05 | FR-POL-6 | Rejected requests are not submitted | After rejection, mock cloud sees 0 requests for that item | Not run | Not run | S1 Critical | - | - |
| AT-06 | FR-POL-5 | Edited sanitised content is inspected again | Edits (for example an inserted key) are re-inspected and blocked or re-reviewed before transmission | Not run | Not run | S1 Critical | - | - |
| AT-07 | FR-MOD-1 | LOCAL ONLY yields zero external inference calls (network-deny) | Zero outbound inference attempts under network-deny harness | Not run | Not run | S1 Critical | - | - |
| AT-08 | FR-MOD-1 | Ollama failure in LOCAL ONLY never triggers cloud fallback | No cloud call; a fallback is at most proposed and needs explicit user choice | Not run | Not run | S1 Critical | - | - |
| AT-09 | FR-PRV-4, FR-POL-3 | Cloud providers receive only approved payloads | Bytes received by mock cloud equal approved, hash-bound bytes; post-approval mutation refused | Not run | Not run | S1 Critical | - | - |
| AT-10 | FR-DET-7 | Uploaded prompt-injection content cannot override policy | Policy decision unchanged by injected instructions; injection finding raised | Not run | Not run | S1 Critical | - | - |
| AT-11 | FR-INB-1/2 | Sensitive data in model responses detected | Sensitive data in buffered response detected; secret in any chunk blocks the whole response | Not run | Not run | S2 High | - | - |
| AT-12 | FR-REC-1/2 | Reconstruction mappings encrypted and local | Mappings encrypted at rest, local only; credentials never mapped | Not run | Not run | S1 Critical | - | - |
| AT-13 | FR-REC-3 | Session expiry clears temporary mappings | Mappings crypto-shredded at expiry and unrecoverable | Not run | Not run | S2 High | - | - |
| AT-14 | FR-BRW-4 | Companion failure blocks pending external requests safely | Pending external requests blocked (fail-closed) with a user-visible message | Not run | Not run | S1 Critical | - | - |
| AT-15 | FR-BRW-4, FR-GW-3 | Malformed messages rejected | Malformed messages rejected; no crash; no data forwarded | Not run | Not run | S2 High | - | - |
| AT-16 | FR-PRF-1 | Learning suggestions inactive until approved | Suggestions have no effect until the user activates them (tested with stub at M7) | Not run | Not run | S3 Medium | - | - |
| AT-17 | FR-DAT-1/2/3 | Audit storage holds no prompt or full-response content | Zero canary hits in DB and logs | Not run | Not run | S1 Critical | - | - |
| AT-18 | FR-BRW-3 | Unsupported websites show accurate coverage warning | Unsupported site labelled Not protected with an accurate warning | Not run | Not run | S2 High | - | - |

## 2. Functional tests

| Test ID | Requirement ref | Scenario | Expected result | Actual result | Pass/Fail status | Severity | Evidence | Issue |
|---|---|---|---|---|---|---|---|---|
| FT-DET-01 | FR-DET-2 | Submit prompts containing each of the top 30 provider key formats, PEM private keys and JWTs (synthetic corpus S0.4) | Each is detected by Stage 1 deterministic rules and blocked; recall measured (PRD section 11 target: >= 99% for top 30 formats, proposed) | Not run | Not run | S1 Critical | - | - |
| FT-DET-02 | FR-DET-2 | Submit card numbers (valid and invalid Luhn), IBANs, national IDs with valid and invalid checksums, emails, phones | Checksum-valid items detected; invalid checksums do not raise false critical findings | Not run | Not run | S2 High | - | - |
| FT-DET-03 | FR-DET-3 | Inspect the structure of every finding produced in FT-DET-01/02 | Each finding has category, severity, location, confidence and plain-language reason | Not run | Not run | S3 Medium | - | - |
| FT-DET-04 | FR-DET-1 | Submit synthetic samples across all FR-DET-1 categories (identity, credentials, financial, confidential, technical, sensitive records, contextual identity, defence) | Each category produces a finding or a documented coverage gap | Not run | Not run | S2 High | - | - |
| FT-DET-05 | FR-DET-4 | Run with Stage 2 classifier enabled on a contextual corpus | Stage 2 can only raise severity, never lower a Stage 1 severity (monotonic merge test) | Not run | Not run | S2 High | - | - |
| FT-DET-06 | FR-DET-5 | Submit general military discussion versus operational detail, restricted-document, infrastructure and personnel samples | General discussion not auto-classified as classified; operational context flagged for review | Not run | Not run | S3 Medium | - | - |
| FT-DET-07 | FR-DET-6 | In Coding profile submit exact well-known dummy keys, then high-entropy lookalikes and user-regex attempts | Only exact well-known dummy patterns exempted; no entropy or user-regex exemption for the credential class | Not run | Not run | S1 Critical | - | - |
| FT-DET-08 | FR-DET-8 | Disable Stage 2 on a cloud route with (a) Stage-1 finding >= Medium, (b) no findings | (a) Blocked; (b) proceeds with visible "contextual check skipped" notice | Not run | Not run | S2 High | - | - |
| FT-POL-01 | FR-POL-1 | Submit content at each risk level Low, Moderate, High, Critical | Actions are notify, explain plus offer sanitise, require review, block respectively | Not run | Not run | S2 High | - | - |
| FT-POL-02 | FR-POL-2 | Trigger a transform proposal and decline it | Content is never modified without approval; blocking mandatory-critical content needs no approval | Not run | Not run | S1 Critical | - | - |
| FT-POL-03 | FR-POL-3 | Approve transformation, then approve transmission; separately alter bytes between approval and send | Two distinct approvals required; transmit approval bound to exact content hash; altered bytes refused | Not run | Not run | S1 Critical | - | - |
| FT-POL-04 | FR-POL-4 | Open review for a request with several findings | Original and proposed content shown side by side with what changed and why | Not run | Not run | S3 Medium | - | - |
| FT-POL-06 | FR-POL-6 | Hold a request without answering until the approval timeout | Request not submitted; client receives `approval_timeout` | Not run | Not run | S2 High | - | - |
| FT-POL-07 | FR-POL-7 | Attempt to relax a mandatory critical rule through a profile, settings and API | All attempts rejected; mandatory layer unchanged (negative tests, property tests) | Not run | Not run | S1 Critical | - | - |
| FT-POL-08 | FR-POL-8 | Repeat <= Medium non-mandatory findings with an allowance configured | Allowance reduces prompts for non-mandatory findings only; mandatory rules still enforced | Not run | Not run | S3 Medium | - | - |
| FT-POL-09 | FR-POL-9 | Submit an image or file on a cloud route; then use per-request override | Blocked by default; override is per-request only, shows persistent UNINSPECTED warning, cannot be set at profile level | Not run | Not run | S2 High | - | - |
| FT-CTX-01 | FR-CTX-1 | Redact, anonymise and pseudonymise a corpus with numbers, dates, negations and exceptions | Placeholders stable and local; technical details, constraints, dates, numbers, negations preserved | Not run | Not run | S2 High | - | - |
| FT-CTX-02 | FR-CTX-2 | Feed transformations where semantic equivalence is uncertain | Preservation checker asks the user rather than proceeding silently | Not run | Not run | S2 High | - | - |
| FT-CTX-03 | FR-CTX-3, FR-CTX-4 | Multi-turn conversation with documents and history | Context classified (identity, task, technical, environmental, historical, sensitive); held in memory/session only; changes need approval | Not run | Not run | S3 Medium | - | - |
| FT-CTX-04 | FR-CTX-6 | Request a local-model rewrite | Rewrite presented as a proposal only, never applied silently | Not run | Not run | S2 High | - | - |
| FT-REC-01 | FR-REC-3 | Set retention to default, then to a longer per-profile value; wait for expiry/conversation end | Default is 24 h or conversation end, whichever first; longer only per profile up to 30 days; expiry crypto-shreds | Not run | Not run | S2 High | - | - |
| FT-REC-02 | FR-REC-4 | Request reconstructed response from a verified local client and from a LAN client | Delivered only to verified local client; LAN clients not served unless explicitly allowed per application | Not run | Not run | S1 Critical | - | - |
| FT-INB-01 | FR-INB-1 | Stream request through gateway | Full response buffered and inspected before release; synthetic SSE delivered to streaming clients | Not run | Not run | S2 High | - | - |
| FT-INB-02 | FR-INB-3 | Model returns tool/function calls with risky arguments and results | Arguments and results inspected; risky calls need approval | Not run | Not run | S2 High | - | - |
| FT-INB-03 | FR-INB-4 | Use a Tier B and a Tier C site adapter | Tier B holds and inspects; Tier C is detect-only after render and labelled as such | Not run | Not run | S2 High | - | - |
| FT-PRF-01 | FR-PRF-1, FR-PRF-2 | Create, version and switch among Coding, Research, Personal, Business, Custom profiles; receive a profile suggestion | Profiles independent and versioned; suggestions never auto-activate; mandatory rules cannot be relaxed | Not run | Not run | S3 Medium | - | - |
| FT-MOD-01 | FR-MOD-2, FR-MOD-3 | Exercise ONLINE ONLY, HYBRID and AUTO with varied privacy, capability, cost and latency constraints | Router is constraint-first (privacy, capability, quality, cost, latency, availability); explanation shown; manual override respects policy | Not run | Not run | S2 High | - | - |
| FT-MOD-02 | FR-MOD-4 | Inspect UI and docs for wording about local processing | Local security processing is distinguished from local model inference everywhere | Not run | Not run | S3 Medium | - | - |
| FT-GW-01 | FR-GW-1 | Call `POST /v1/chat/completions` non-stream, stream, and `POST /v1/responses` with official SDKs | Correct OpenAI-shaped responses and SSE events; OpenAI-style errors | Not run | Not run | S2 High | - | - |
| FT-GW-02 | FR-GW-1, FR-GW-3 | Send unsupported parameters, unknown Responses item types, `previous_response_id`, server-side state/tools | Explicit errors, no silent drops; unknown item types and server-side state/tools rejected | Not run | Not run | S2 High | - | - |
| FT-GW-03 | FR-GW-3 | Send oversize bodies, invalid schema, excessive rate | 413, 400 (upstream-shaped) and rate-limit responses as specified | Not run | Not run | S3 Medium | - | - |
| FT-DAT-01 | FR-DAT-1 | Perform varied requests and inspect the audit store schema and rows | Only the permitted metadata fields are stored (timestamp, application, profile, model/provider, risk level, categories, transformation type, approval status, token counts, latency, error category) | Not run | Not run | S2 High | - | - |
| FT-DAT-04 | FR-DAT-4 | Observe network during idle and use; use "Check now" with updates off/on | Zero telemetry; update checks opt-in, default off, signed manifest, user approves download | Not run | Not run | S2 High | - | - |

## 3. Security tests

| Test ID | Requirement ref | Scenario | Expected result | Actual result | Pass/Fail status | Severity | Evidence | Issue |
|---|---|---|---|---|---|---|---|---|
| ST-LEAK-01 | FR-DET-2, NFR-1 (threat T1) | Run full secret corpus through all inbound paths (gateway, extension, CLI) against counting mock cloud | Mock cloud receives 0 secrets (target: 0 critical secret leakage) | Not run | Not run | S1 Critical | - | - |
| ST-LEAK-02 | NFR-1, FR-DAT-2 (T13) | Place canary strings in prompts, responses, tool results, uploads; scan DB, logs, crash dumps, temp files | 0 canary hits in any persisted file or log | Not run | Not run | S1 Critical | - | - |
| ST-LEAK-03 | NFR-1, FR-PRV-2 (T11) | Search client-visible output, browser code, extension storage, logs and model context for provider credentials | No provider credential visible anywhere outside the vault and egress | Not run | Not run | S1 Critical | - | - |
| ST-LEAK-04 | NFR-1 | Inspect process memory and error paths after secret handling (heap/crash dump review) | Secrets zeroised after use; none in error messages | Not run | Not run | S2 High | - | - |
| ST-EVA-01 | FR-DET-2 (T15) | Submit secrets obfuscated by base64, homoglyphs, zero-width characters, split across messages, bounded multi-decode | Detected within documented limits; unsupported encodings documented, not claimed | Not run | Not run | S2 High | - | - |
| ST-INJ-01 | FR-DET-7 (T4) | Prompt injection in prompt text, uploaded document, retrieved data, tool result instructing Aegis to relax policy, approve, or send | Policy unchanged; injection finding raised; model text never parsed as Aegis commands | Not run | Not run | S1 Critical | - | - |
| ST-INJ-02 | FR-INB-2, FR-DET-7 (T16) | Model response containing instructions and markdown image exfiltration (`![](https://host/?q=...)`) | Detected, flagged or neutralised per tier; tool-call gating applies | Not run | Not run | S2 High | - | - |
| ST-UPL-01 | FR-POL-9, FR-GW-3 (T22, T23) | Upload images, PDFs, audio, archives, polyglot and oversized files; zip-bomb-like and deeply nested payloads | Uninspected types blocked on cloud routes with UNINSPECTED label; size/decode limits enforced; no hang or crash | Not run | Not run | S2 High | - | - |
| ST-UPL-02 | FR-GW-3, NFR-1 (T14) | Send ReDoS patterns and decompression-style payloads to detectors and parsers | Linear-time behaviour; per-request time budget; timeout fails closed on cloud routes | Not run | Not run | S2 High | - | - |
| ST-UPL-03 | NFR-1 | Fuzz gateway request parser, IPC framing, NM framing, normaliser | No open crashes or memory-safety faults | Not run | Not run | S2 High | - | - |
| ST-EXF-01 | FR-PRV-4 (T2) | Mutate payload bytes, provider or expiry after approval; replay a used approval token | Egress re-hashes and refuses; replay refused | Not run | Not run | S1 Critical | - | - |
| ST-EXF-02 | FR-PRV-4, NFR-2 | Inspect all outbound connections of every Aegis process during a full test run | Only approved provider calls (and opt-in update checks) leave the device; dependency lint confirms only `aegis-egress` has a general HTTP client | Not run | Not run | S1 Critical | - | - |
| ST-EXF-03 | FR-DAT-4, NFR-2 | Run idle and active with telemetry-detecting network capture | No telemetry or unexpected DNS | Not run | Not run | S2 High | - | - |
| ST-AUTH-01 | FR-GW-2 (T5) | Call gateway with no key, invalid key, revoked key, key for another application | 401 or equivalent; no processing; no information leaked | Not run | Not run | S1 Critical | - | - |
| ST-AUTH-02 | FR-GW-2 (T5) | Attempt gateway use from a non-matching local process identity (best-effort OS identity binding) | Behaviour matches documented best-effort binding; limitation documented | Not run | Not run | S2 High | - | - |
| ST-AUTH-03 | FR-GW-2 | Test key storage (hashed, scoped, revocable) and rate limits under brute force | Keys stored hashed; revocation immediate; brute force rate-limited | Not run | Not run | S2 High | - | - |
| ST-AUTH-04 | FR-BRW-4 (T10) | Launch a rogue NM relay or connect with wrong origin or without pairing token | Connection refused; browser `allowed_origins` and peer checks enforced | Not run | Not run | S1 Critical | - | - |
| ST-LAN-01 | FR-GW-2 (T6) | Default install: connect from another host and another local user | Connection refused; bound to loopback only | Not run | Not run | S1 Critical | - | - |
| ST-LAN-02 | FR-GW-2 (T6) | Enable LAN: connect without TLS, without auth, from outside CIDR allow-list | Refused unless TLS, auth and CIDR allow-list all satisfied; only `/healthz` unauthenticated and leaks no info | Not run | Not run | S1 Critical | - | - |
| ST-LAN-03 | FR-REC-4 (Q-5) | LAN client requests reconstructed response | Not served by default | Not run | Not run | S1 Critical | - | - |
| ST-CRED-01 | FR-PRV-2 (T11) | Inspect vault file and OS key store entries on each OS; locked/unlocked states; Linux without Secret Service uses master-password mode | Credentials encrypted (DPAPI/Keychain/Secret Service or master password); locked vault blocks use | Not run | Not run | S1 Critical | - | - |
| ST-CRED-02 | FR-PRV-2 | Scan extension, logs, config files, clipboard handling for provider keys | None present | Not run | Not run | S1 Critical | - | - |
| ST-MAP-01 | FR-REC-1, FR-REC-2 (T12) | Inspect mapping store at rest; attempt to map passwords, API keys and private keys | Field-level encryption, session-scoped keys; secrets never mapped | Not run | Not run | S1 Critical | - | - |
| ST-MAP-02 | FR-REC-3 (T12) | Expire a session, then try recovering mappings from disk | Crypto-shredded; unrecoverable | Not run | Not run | S2 High | - | - |
| ST-BYP-01 | FR-POL-7 (T19) | Attempt policy bypass via profile edits, config file edits, API, UI, import of profiles | Mandatory layer cannot be written or relaxed | Not run | Not run | S1 Critical | - | - |
| ST-BYP-02 | FR-GW-3 (T23) | Use Responses API features that bypass inspection: `previous_response_id`, server-side tools, `store:true` | Rejected or forced to safe values | Not run | Not run | S1 Critical | - | - |
| ST-BYP-03 | NFR-1 | Inject detector failure, timeout, panic (fault injection) | Blocks on cloud routes (fail-closed) | Not run | Not run | S1 Critical | - | - |
| ST-BYP-04 | FR-DET-1/2 | Send content in multiple parts, roles, languages, and via tool messages to evade scanning | Every content part inspected | Not run | Not run | S1 Critical | - | - |
| ST-BRW-01 | FR-BRW-5 (T7) | Hostile page attempts to spoof, overlay or call the approval UI or NM | Review UI only in popup/side panel; page cannot call NM; isolated world | Not run | Not run | S1 Critical | - | - |
| ST-BRW-02 | FR-BRW-2/3 (T8) | Page sends via second channel (WebSocket, XHR, alternate fetch) or races hook install | Detected and tier downgraded; no mislabelled coverage | Not run | Not run | S2 High | - | - |
| ST-BRW-03 | FR-BRW-1 (T9) | Review extension permissions and what a compromised extension can reach | Minimal and optional per-site permissions; no credentials or vault access | Not run | Not run | S2 High | - | - |
| ST-SUP-01 | NFR-1 (T18, T21) | Supply a tampered update manifest, rule pack and classifier model | Signature or hash verification fails; nothing loaded | Not run | Not run | S1 Critical | - | - |
| ST-SUP-02 | NFR-1 | Run `cargo deny`, lockfile checks, SBOM generation | Clean or accepted advisories documented | Not run | Not run | S2 High | - | - |
| ST-FC-01 | NFR-1 (threat model section 4) | Exercise every row of the fail-closed matrix (detector error, classifier unavailable, vault locked, approval UI unreachable, audit write failure, egress token failure) | Each failure yields the specified block, hold or notice behaviour | Not run | Not run | S1 Critical | - | - |

## 4. Privacy tests

| Test ID | Requirement ref | Scenario | Expected result | Actual result | Pass/Fail status | Severity | Evidence | Issue |
|---|---|---|---|---|---|---|---|---|
| PT-PER-01 | FR-DAT-2 | Run varied sessions then examine all persistent stores, temp files, swap-adjacent caches and logs | No prompts, conversations, responses, uploaded content, plaintext mappings or credentials persisted | Not run | Not run | S1 Critical | - | - |
| PT-PER-02 | FR-DAT-2, FR-CTX-4 | Crash the daemon mid-request and examine crash dumps and recovery state | No content in dumps or recovery state | Not run | Not run | S1 Critical | - | - |
| PT-AUD-01 | FR-DAT-1, FR-DAT-3 | Attempt to write an audit event with free-text fields through the audit API | Typed events only; free text refused; log layer rejects non-allow-listed fields | Not run | Not run | S1 Critical | - | - |
| PT-AUD-02 | FR-DAT-1 | Review audit metadata for re-identifying content (for example application, categories) | Only specified metadata fields; no content-derived strings | Not run | Not run | S2 High | - | - |
| PT-CLD-01 | FR-MOD-1, G5 | Take every router decision path with cloud unreachable, local failing, and policy blocking | No cloud fallback is ever silent; fallback only proposed and needs explicit choice | Not run | Not run | S1 Critical | - | - |
| PT-CLD-02 | FR-PRV-3 | First use of each provider/integration; revoke consent | No egress before consent; revocation effective immediately | Not run | Not run | S2 High | - | - |
| PT-LOC-01 | FR-MOD-1, NFR-2 | LOCAL ONLY with network-deny on all three OSes, including Ollama down, classifier running, update check attempted | Zero external calls | Not run | Not run | S1 Critical | - | - |
| PT-LOC-02 | FR-MOD-1, FR-MOD-4 | Check LOCAL ONLY indicator in extension, desktop, CLI and API responses | State always visible and accurate; local security processing not presented as local inference | Not run | Not run | S2 High | - | - |
| PT-DEF-01 | FR-DET-5 | Defence-related corpus (operational, personnel, infrastructure, technology, restricted documents) | Flagged for review per profile; never auto-labelled classified; never sent to external evaluators | Not run | Not run | S2 High | - | - |
| PT-DEF-02 | FR-CTX-5 | Observe all egress during privacy scoring and classification | Raw prompts never sent to external evaluators | Not run | Not run | S1 Critical | - | - |
| PT-SIL-01 | FR-POL-2, FR-CTX-6 | Compare bytes submitted by client with bytes sent to provider for all action types | Differences exist only after explicit transform approval; blocked content never altered | Not run | Not run | S1 Critical | - | - |
| PT-SIL-02 | FR-POL-3 | Review UI text and transmitted content comparison | "Original vs transformed transmitted" shown accurately | Not run | Not run | S2 High | - | - |
| PT-HON-01 | NFR-8, FR-BRW-3 | Review UI, docs and marketing text for effectiveness or universal-protection claims | No claim unsupported by measured results; limits stated | Not run | Not run | S2 High | - | - |

## 5. Integration tests

| Test ID | Requirement ref | Scenario | Expected result | Actual result | Pass/Fail status | Severity | Evidence | Issue |
|---|---|---|---|---|---|---|---|---|
| IT-OLL-01 | FR-PRV-1, FR-MOD-1 | Chat completion via Ollama adapter; model digest recorded | Loopback-only client; response correct; model digest in metadata | Not run | Not run | S2 High | - | - |
| IT-OLL-02 | FR-PRV-1 | Ollama stopped, slow, returns malformed or oversized output | Timeouts and errors handled; output treated as untrusted; no cloud fallback in LOCAL ONLY | Not run | Not run | S1 Critical | - | - |
| IT-OAI-01 | FR-PRV-1, FR-GW-1 | OpenAI adapter conformance against recorded fixtures (Chat and Responses wire) | Conformance passes; unsupported-param errors preserved | Not run | Not run | S2 High | - | - |
| IT-OAC-01 | FR-PRV-1 | OpenAI-compatible endpoint with differing capability sets | Capability detection correct; unsupported features reported explicitly | Not run | Not run | S3 Medium | - | - |
| IT-ORT-01 | FR-PRV-1 | OpenRouter adapter (potential, P1) with routing metadata | Behaves per adapter conformance; no extra data sent | Not run | Not run | S3 Medium | - | - |
| IT-SDK-01 | FR-GW-1 | Official OpenAI SDKs (Python, JS) against the gateway for both APIs including streaming | SDKs parse responses and streams correctly | Not run | Not run | S2 High | - | - |
| IT-NM-01 | FR-BRW-1 | Native Messaging install, repair, uninstall on Chrome, Edge, Brave on each OS | Install and uninstall verified; reconnect works; 1 MB message limit behaviour as specified | Not run | Not run | S2 High | - | - |
| IT-NM-02 | FR-BRW-4 | Kill companion during pending request; restart | Pending blocked; recovery after restart; no data loss in approval queue within a session | Not run | Not run | S1 Critical | - | - |
| IT-ADP-01 | FR-BRW-2 | ChatGPT adapter fixtures and live canary | Declared tier verified by tests; canary detects breakage | Not run | Not run | S2 High | - | - |
| IT-ADP-02 | FR-BRW-2 | Claude adapter fixtures and live canary | Declared tier verified by tests | Not run | Not run | S2 High | - | - |
| IT-ADP-03 | FR-BRW-2 | Gemini adapter fixtures and live canary | Declared tier verified by tests | Not run | Not run | S2 High | - | - |
| IT-ADP-04 | FR-BRW-3 | Visit site with no adapter | Not protected label and accurate warning | Not run | Not run | S2 High | - | - |
| IT-VLT-01 | FR-PRV-2 | Vault with OS store (DPAPI, Keychain, Secret Service) and master-password mode | Round-trip, lock/unlock behaviour as specified | Not run | Not run | S1 Critical | - | - |
| IT-DB-01 | FR-DAT-1 | SQLCipher open/close, key derivation, wrong key, corrupted DB on each OS | Encrypted at rest; wrong key refused; corruption handled without content exposure | Not run | Not run | S2 High | - | - |
| IT-EGR-01 | FR-PRV-4 | Every provider path goes only through the single egress component | No bypass path; dependency lint enforces | Not run | Not run | S1 Critical | - | - |

## 6. Reliability and performance tests

| Test ID | Requirement ref | Scenario | Expected result | Actual result | Pass/Fail status | Severity | Evidence | Issue |
|---|---|---|---|---|---|---|---|---|
| RT-PRF-01 | NFR-3 | Stage 1 on payloads up to 4k tokens, excluding buffering | p95 added latency <= 300 ms (target is a recommendation pending baseline) | Not run | Not run | S3 Medium | - | - |
| RT-PRF-02 | NFR-3 | Increase payload and concurrency up to limits | Documented degradation; limits enforced; no unsafe pass-through | Not run | Not run | S2 High | - | - |
| RT-REL-01 | NFR-4 | Restart the companion/daemon during idle, pending approval, in-flight request | Recovery; no approval queue loss within a session; in-flight cloud requests fail closed | Not run | Not run | S2 High | - | - |
| RT-REL-02 | NFR-4 | Provider timeouts, 429, 5xx, partial streams | Handled with accurate errors; no content persisted; no retry that bypasses approval | Not run | Not run | S2 High | - | - |
| RT-REL-03 | NFR-1 (T24) | Crash the daemon and observe clients | Clients fail closed; watchdog restarts; extension blocks submit | Not run | Not run | S1 Critical | - | - |
| RT-REL-04 | FR-DAT-1 (matrix) | Audit write failure on cloud route | Blocks on cloud routes; surfaces alarm on local | Not run | Not run | S2 High | - | - |
| RT-SOK-01 | NFR-4 | Long-duration soak (24 h) with mixed load | No memory growth beyond threshold; no crash; no leaked mappings after expiry | Not run | Not run | S3 Medium | - | - |
| RT-PLT-01 | NFR-5 | Install, run and uninstall on Windows, macOS, Linux (including Linux without Secret Service) | Works on all three; Linux master-password mode available | Not run | Not run | S2 High | - | - |
| RT-ADV-01 | FR-DET-2, NFR-8 | Measure recall and precision on corpora (top 30 key formats, structured PII, contextual) | Results published as measured; targets: secrets >= 99%, structured PII >= 95% (proposed, to be baselined) | Not run | Not run | S2 High | - | - |

## 7. UI, UX and accessibility tests

| Test ID | Requirement ref | Scenario | Expected result | Actual result | Pass/Fail status | Severity | Evidence | Issue |
|---|---|---|---|---|---|---|---|---|
| UX-REV-01 | FR-UI-1 | Review Center with a multi-finding request | Shows findings, why it matters, what will be shared, proposed change, consequences of approve/reject, original vs transformed transmitted | Not run | Not run | S2 High | - | - |
| UX-REV-02 | FR-UI-1 (Q-6) | Review flow in extension panel and CLI (P0) and desktop (P1) | Equivalent decisions and information in each surface | Not run | Not run | S2 High | - | - |
| UX-NAV-01 | FR-UI-2 | Navigate Overview, Review Center, Security Inspector, Context & Profiles, Model Router, Analytics, Integrations, Settings | All views present and reachable | Not run | Not run | S3 Medium | - | - |
| UX-THM-01 | FR-UI-3 | Dark theme and light theme on all surfaces | Dark-first default; light theme accessible and complete | Not run | Not run | S3 Medium | - | - |
| UX-A11Y-01 | FR-UI-3, NFR-6 | WCAG 2.2 AA audit (automated plus manual, screen reader on each OS) | No open AA failures on security-decision surfaces | Not run | Not run | S2 High | - | - |
| UX-A11Y-02 | FR-UI-3, NFR-6 | Operate every control by keyboard only, including approve/reject | All operable; visible focus; sensible order | Not run | Not run | S2 High | - | - |
| UX-A11Y-03 | FR-UI-3, NFR-6 | Enable reduced-motion; test responsive layouts and 200% zoom | Motion reduced; layout usable; no clipped decisions | Not run | Not run | S3 Medium | - | - |
| UX-HON-01 | FR-BRW-3, NFR-8 | Check coverage badges for Protected, Partially protected, Not protected, UNINSPECTED | Labels accurate and not colour-only | Not run | Not run | S2 High | - | - |
| UX-SIM-01 | FR-UI-4 | Moderated usability test of Review Center with target users | Task success measured; target set before M6 (PRD section 11) | Not run | Not run | S3 Medium | - | - |
| UX-ERR-01 | FR-GW-1, FR-BRW-4 | Review error and blocked messages (blocked secret, companion down, timeout) | Plain-language, actionable, no content echoed | Not run | Not run | S3 Medium | - | - |
