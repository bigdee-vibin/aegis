# Aegis Integration Coverage Matrix (v0.1 draft)

**Status:** Draft for QA and owner review. **Sources:** PRD v1.0 draft 1, backlog, threat model. Cells marked "TBD" are decisions not yet made in the PRD; they are not facts.

## 1. Status vocabulary

| Status | Meaning | Evidence required to claim it |
|---|---|---|
| Planned | In scope or candidate; no code merged | Entry in PRD/backlog |
| In development | Code under active development; not yet usable | Open branch or PR |
| Implemented | Code merged and functional | Merged code and documentation |
| Tested | Automated tests exist and pass on Windows, macOS, Linux | Green CI run with links in [test-matrix.md](test-matrix.md) |
| Protected | Tested, plus the declared protection claim (what Aegis inspects and blocks for this integration) is verified by the relevant security and privacy tests, and the coverage label is verified by canary or conformance tests | Passing ST-/PT-/IT- rows and sign-off from QA lead |
| Partially protected | As Protected, but only for a documented subset (for example detect-only after render, or some OSes) | Passing tests for the subset; limits stated in UI and docs |
| Unsupported | Explicitly not supported; users are warned accurately (FR-BRW-3) | Documented limitation |
| Deprecated | Previously supported, being retired | Deprecation notice and removal date |

Rules: statuses only advance on evidence. No integration may be labelled Protected without measured results (PRD NFR-8). Status can move backwards if a canary or conformance test fails.

## 2. Summary

| Integration | PRD ref | Priority | Status | Test coverage (rows in test-matrix) |
|---|---|---|---|---|
| Ollama | FR-PRV-1, FR-MOD-1 | P1 (P0 promotion proposed, Q-4) | Planned | IT-OLL-01/02, AT-07/08, PT-LOC-01 |
| OpenAI | FR-PRV-1, FR-GW-1 | P1 | Planned | IT-OAI-01, AT-09 |
| OpenAI-compatible | FR-PRV-1 | P1 | Planned | IT-OAC-01 |
| OpenRouter | FR-PRV-1 (potential) | P1/P2 | Planned | IT-ORT-01 |
| ChatGPT site adapter | FR-BRW-2 | P0 (at least one adapter) / P1 | Planned | IT-ADP-01, AT-18 |
| Claude site adapter | FR-BRW-2 | P1 | Planned | IT-ADP-02 |
| Gemini site adapter | FR-BRW-2 | P1 | Planned | IT-ADP-03 |
| Chrome Native Messaging (Chrome, Edge, Brave) | FR-BRW-1/4 | P0 | Planned | IT-NM-01/02, AT-14/15, ST-AUTH-04 |
| Local gateway (`/v1/chat/completions`, `/v1/responses`) | FR-GW-1/2/3 | P0 (chat, non-stream) / P1 | Planned | FT-GW-01..03, IT-SDK-01, ST-AUTH-01, ST-LAN-01/02 |
| Credential vault | FR-PRV-2 | P0 | Planned | IT-VLT-01, ST-CRED-01/02 |
| SQLite/SQLCipher | FR-DAT-1/2/3 | P0 | Planned | IT-DB-01, AT-17, PT-PER-01, PT-AUD-01 |

No integration is Protected or Partially protected today. Nothing is Implemented or Tested at the time of writing.

## 3. Detail

Field definitions are the same for each entry. "TBD" means not decided in the PRD.

### 3.1 Ollama

| Field | Detail |
|---|---|
| Status | Planned |
| Purpose | Local model inference for LOCAL ONLY and hybrid routing (backlog S3.1) |
| Auth method | None expected on loopback (TBD whether a token is supported) |
| Capabilities | Chat completion via local model; capability detection (FR-PRV-1); model digest recorded in metadata |
| Permissions | Loopback network access only; no internet access by Aegis for this adapter |
| Data exchanged | Aegis sends the inspected, approved prompt; receives model output, which is untrusted and inspected inbound |
| Privacy and security implications | Distinguish local security processing from local inference (FR-MOD-4). Model output is untrusted (prompt injection, secrets). Ollama is semi-trusted (threat model). Model files are user-chosen; digest verification where available (T21) |
| Failure modes | Ollama stopped, slow, malformed or oversized output, model missing. In LOCAL ONLY a failure yields a proposed fallback needing explicit user choice, never automatic cloud fallback (AT-08) |
| Version compatibility | Supported Ollama versions: TBD (to be pinned after S3.1) |
| Test coverage | Planned: IT-OLL-01/02, AT-07, AT-08, PT-CLD-01, PT-LOC-01 (all Not run) |
| Maintenance status | Not started; PRD Q-4 asks whether S3.1/S3.2 are promoted to P0 |

### 3.2 OpenAI

| Field | Detail |
|---|---|
| Status | Planned |
| Purpose | Cloud inference through approved egress (backlog S3.5) |
| Auth method | Provider API key held in the central vault; never visible to clients, browser or logs (FR-PRV-2) |
| Capabilities | Chat Completions and Responses wire formats; streaming (P1); unsupported parameters return explicit errors |
| Permissions | Outbound TLS to the provider through the single egress component only; first-use consent per provider (FR-PRV-3) |
| Data exchanged | Only content with an approval capability, bound to the exact content hash (FR-PRV-4); provider sees approved content plus network metadata (IP, timing) |
| Privacy and security implications | Provider retention and training are outside Aegis control after transmission (T17); consent text shown; `store:false` default; `previous_response_id`, server-side state and tools rejected (T23); images and files blocked by default (T22) |
| Failure modes | Timeouts, 429, 5xx, partial streams, schema drift, credential revoked |
| Version compatibility | API versions and SDK versions: TBD; conformance against recorded fixtures |
| Test coverage | Planned: IT-OAI-01, IT-SDK-01, AT-09, ST-EXF-01 (all Not run) |
| Maintenance status | Not started; fixture refresh process TBD |

### 3.3 OpenAI-compatible

| Field | Detail |
|---|---|
| Status | Planned |
| Purpose | Third-party or self-hosted endpoints speaking the OpenAI wire format (S3.5) |
| Auth method | API key in the vault, or none for loopback endpoints (TBD) |
| Capabilities | Detected per endpoint (FR-PRV-1); features not supported are reported, not silently dropped |
| Permissions | Outbound to user-configured base URL via egress; consent per endpoint |
| Data exchanged | Approved payloads only; response inspected inbound |
| Privacy and security implications | Trust level of the endpoint is user-defined and varies; non-loopback endpoints are cloud routes for LOCAL ONLY purposes (TBD classification of private-LAN endpoints must be defined); user-supplied URL is a misconfiguration and SSRF-style risk |
| Failure modes | Partial compatibility, differing error shapes, streaming divergences, TLS errors |
| Version compatibility | Per endpoint; no guarantee. Conformance suite defines a compatibility floor |
| Test coverage | Planned: IT-OAC-01 (Not run) |
| Maintenance status | Not started |

### 3.4 OpenRouter (potential)

| Field | Detail |
|---|---|
| Status | Planned (potential, not committed in PRD) |
| Purpose | Aggregated access to multiple upstream models |
| Auth method | API key in the vault |
| Capabilities | As OpenAI-compatible, plus model catalogue; capability detection |
| Permissions | Outbound via egress; first-use consent |
| Data exchanged | Approved payloads; routing metadata |
| Privacy and security implications | Data passes through an intermediary and then an upstream model provider; retention differs by upstream; consent text must say so; routing must respect provider data-handling constraints |
| Failure modes | Upstream selection changes, rate limits, model deprecation |
| Version compatibility | TBD |
| Test coverage | Planned: IT-ORT-01 (Not run) |
| Maintenance status | Not started; inclusion decision pending |

### 3.5 ChatGPT, Claude, Gemini site adapters (Chrome extension)

Each adapter is tracked separately. Same attributes unless noted.

| Field | ChatGPT | Claude | Gemini |
|---|---|---|---|
| Status | Planned | Planned | Planned |
| Purpose | Inspect submissions to and responses from the website UI (FR-BRW-2) | Same | Same |
| Auth method | None for Aegis; the user's own site login is untouched. Extension talks to the companion via Native Messaging | Same | Same |
| Capabilities | Capability manifest, fixtures and live canary; tier declared (B holds and inspects, C is detect-only after render) | Same | Same |
| Permissions | Minimal, optional per-site host permissions; no credentials in extension | Same | Same |
| Data exchanged | Composer text and responses sent to the local companion for inspection; decisions returned | Same | Same |
| Privacy and security implications | Page JS is hostile by assumption (T7, T8); approval UI only in popup/side panel (FR-BRW-5); extension treated as untrusted by the daemon (T9); site terms of service and legal review of interception pending (PRD section 12) | Same | Same |
| Failure modes | Site DOM or network change breaks interception; second channel bypass; tier downgrade; companion down (blocked, fail-closed) | Same | Same |
| Version compatibility | Tracks site UI and API revisions, not versioned by the site; browser: Chrome, Edge, Brave (MV3) | Same | Same |
| Test coverage | IT-ADP-01, AT-18, ST-BRW-01/02 | IT-ADP-02 | IT-ADP-03 |
| Maintenance status | Not started; at least one adapter must ship at P0 (first named: ChatGPT); tier decided after spike S0.7 | Not started; P1 | Not started; P1 |

Unsupported websites are labelled Not protected (FR-BRW-3; AT-18). That label is a required behaviour, not an integration to track here.

### 3.6 Chrome Native Messaging (Chrome, Edge, Brave)

| Field | Detail |
|---|---|
| Status | Planned |
| Purpose | Channel between the MV3 extension and the local companion/daemon (FR-BRW-1; S0.6, S5.1) |
| Auth method | Browser-enforced `allowed_origins`; relay path/signature and peer credential checks; pairing token (threat model T10) |
| Capabilities | Message round trip, reconnection, framing with 1 MB message limit (to be verified by spike), install/repair/uninstall per OS and browser |
| Permissions | Registry or manifest entries per OS and browser; relay binary executes as the user |
| Data exchanged | Text submitted for inspection; decisions, hash-bound replace_text; no credentials |
| Privacy and security implications | Same-user malware can launch the relay binary (residual risk, documented); malformed messages must be rejected (AT-15); disconnect fails closed (AT-14) |
| Failure modes | Companion crash, manifest missing after browser update, message too large, malformed framing |
| Version compatibility | Chrome, Edge, Brave on Windows, macOS, Linux; minimum versions TBD |
| Test coverage | Planned: IT-NM-01/02, AT-14, AT-15, ST-AUTH-04, ST-UPL-03 (all Not run) |
| Maintenance status | Not started; spike S0.6 pending |

### 3.7 Local gateway

| Field | Detail |
|---|---|
| Status | Planned |
| Purpose | OpenAI-compatible local endpoint through which applications reach Aegis (FR-GW-1/2/3) |
| Auth method | Per-application API keys (stored hashed, scoped, revocable) plus supported local identity verification; hybrid client authentication |
| Capabilities | `POST /v1/chat/completions` (non-stream P0), `POST /v1/responses`, streaming (P1), OpenAI-style errors, validation, size and rate limits |
| Permissions | Binds loopback by default; LAN only when explicitly enabled with TLS, CIDR allow-list and authentication; `/healthz` unauthenticated and information-free |
| Data exchanged | Client requests (full prompts) in; inspected responses out; buffer-before-forward with synthetic SSE (ADR-0007) |
| Privacy and security implications | Primary TCB boundary. Per-app identity on loopback is best-effort (T5). LAN exposure is a high-risk configuration (T6). Reconstructed output never goes to LAN clients by default (FR-REC-4) |
| Failure modes | Auth failure (401), oversize (413), schema errors (400), unsupported parameters, rate limit, daemon crash (clients fail closed) |
| Version compatibility | OpenAI API shapes as pinned by fixtures; SDK versions TBD; unsupported parameters return explicit errors |
| Test coverage | Planned: FT-GW-01..03, IT-SDK-01, ST-AUTH-01..03, ST-LAN-01..03, AT-01, AT-15 (all Not run) |
| Maintenance status | Not started; wire-format fixtures must be refreshed on provider API changes |

### 3.8 Credential vault

| Field | Detail |
|---|---|
| Status | Planned |
| Purpose | Central encrypted storage of provider credentials and, separately, pseudonym mappings (FR-PRV-2, FR-REC-1) |
| Auth method | OS-protected root key (DPAPI, Keychain, Secret Service) or master password (Argon2id); Linux master-password mode when no Secret Service |
| Capabilities | Store, use without exposure, lock/unlock, session-scoped keys and TTL for mappings, crypto-shred on expiry |
| Permissions | Local files and OS key store entries as the user |
| Data exchanged | Credentials flow only to the egress component; never to clients, browser, logs or models |
| Privacy and security implications | Theft of vault plus root key equals credential loss (root-level malware out of scope); mappings never contain passwords, API keys or private keys (FR-REC-2); memory zeroisation (NFR-1) |
| Failure modes | Locked vault, OS key store unavailable, corrupted file, wrong master password, key loss. A locked vault blocks use (fail-closed matrix) |
| Version compatibility | OS versions per ADR-0020; vault format versioning and migration TBD |
| Test coverage | Planned: IT-VLT-01, ST-CRED-01/02, ST-MAP-01/02, AT-12, AT-13 (all Not run) |
| Maintenance status | Not started; depends on spike S0.5 |

### 3.9 SQLite and SQLCipher (audit store)

| Field | Detail |
|---|---|
| Status | Planned |
| Purpose | Local persistence of metadata-only audit events (FR-DAT-1) |
| Auth method | SQLCipher key derived and protected via the OS key store or master password |
| Capabilities | Typed event writes only; no free-text content fields; local analytics from metadata |
| Permissions | Local file as the user |
| Data exchanged | Timestamp, application, profile, model/provider, risk level, categories, transformation type, approval status, token counts, latency, error category. Never prompts, responses, uploads, plaintext mappings or credentials (FR-DAT-2) |
| Privacy and security implications | Even metadata reveals behaviour; encrypted at rest; integrity matters (A6). Audit write failure blocks cloud transmission (cannot record) |
| Failure modes | Wrong key, corruption, disk full, write failure (block on cloud routes), migration failure |
| Version compatibility | SQLCipher and SQLite versions TBD; schema versioning and migration TBD |
| Test coverage | Planned: IT-DB-01, AT-17, PT-PER-01/02, PT-AUD-01/02, RT-REL-04 (all Not run) |
| Maintenance status | Not started; depends on spike S0.5 |

## 4. Maintenance rules

- Each integration has an owner (TBD) who reviews status monthly and after any upstream change.
- A failing live canary or conformance test moves the integration back from Protected/Tested to the last status supported by passing evidence, and the user-facing coverage label updates.
- Unsupported or Deprecated integrations remain listed with the reason and date.
