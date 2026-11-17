# Aegis — Internal API Contracts (v0.1)

Rust types in `aegis-core` are the source of truth; TS types and JSON Schema are generated (`schemars` + `ts-rs`). Snippets below are normative in *shape*, not final syntax.

## 1. Canonical request / response

```rust
pub struct AegisRequest {
    pub id: RequestId,                    // ULID
    pub client: ClientIdentity,           // authenticated app/extension (never from payload)
    pub source: Source,                   // Gateway{api: Responses|ChatCompletions} | Extension{site, adapter_ver}
    pub mode: OperatingMode,              // LocalOnly | OnlineOnly | Hybrid | Auto (resolved from profile+client)
    pub profile: ProfileRef,              // id + version
    pub model_hint: Option<String>,       // client-requested model; router may map
    pub messages: Vec<Message>,           // canonical, provider-neutral
    pub tools: Vec<ToolDef>,
    pub params: Params,                   // temperature, max_tokens, response_format, ...
    pub stream: bool,                     // client intent only; upstream always buffered
    pub passthrough: PassthroughParams,   // unknown fields kept for adapters; see §6
}

pub struct Message { pub role: Role, pub parts: Vec<Part>, pub provenance: Provenance }

pub enum Part {
    Text { text: String },
    ToolCall { id: String, name: String, arguments_json: String },
    ToolResult { call_id: String, content: String },      // provenance forced Untrusted
    Unsupported { kind: String, bytes: Option<u64> },      // image/file/audio: never "inspected"
}

pub enum Provenance { UserTyped, UserPasted, SystemConfig, Assistant, ToolOutput, Retrieved, Document }
// Trust: only SystemConfig/UserTyped are Trusted; everything else Untrusted.
```

Response mirror: `AegisResponse { request_id, route: RouteDecision, messages, tool_calls, usage, finish_reason, inspection: InboundInspection }`.

**Never lossy-by-default**: any request parameter an adapter cannot honour yields `aegis_unsupported_parameter` (HTTP 400, upstream-shaped error body with `error.param`) — it is not silently dropped (per brief).

## 2. Gateway surface

| Endpoint | Behaviour |
|---|---|
| `POST /v1/chat/completions` | OpenAI Chat Completions shape in/out; `stream` supported via synthetic SSE; `tools`, `response_format` (json_schema) forwarded when target supports; else error |
| `POST /v1/responses` | Responses shape; `store` forced false (or error), `previous_response_id` rejected, server-side tools rejected |
| `GET /v1/models` | Returns Aegis-visible virtual models (`aegis/auto`, `aegis/local`, plus per-provider mapped names) |
| `GET /healthz` | `{"status":"ok"}` only; no version/config detail |

Auth: `Authorization: Bearer aegis_<app>_<secret>`; key → client record (hashed with Argon2id/HMAC, scope, rate limit, allowed routes, `on_review` behaviour, optional OS-identity binding). Provider credentials are never accepted from or returned to clients; client-supplied upstream `Authorization` headers are rejected.

**Aegis error envelope** (OpenAI-compatible so SDKs surface it):

```json
{ "error": { "message": "Request blocked: credential detected",
             "type": "aegis_blocked", "code": "critical_secret",
             "param": null,
             "aegis": { "request_id": "01J...", "reasons": ["SECRET.OPENAI_KEY"], "review_url": null } } }
```
Note: the error includes finding *types* and counts, never the matched text.

Codes: `critical_secret`, `policy_blocked`, `review_rejected`, `approval_timeout`, `inspection_failed`, `unsupported_content`, `unsupported_parameter`, `unsupported_server_tool`, `local_only_unavailable`, `locked`, `rate_limited`, `payload_too_large`, `provider_error`.

## 3. Detector & classifier interfaces

```rust
pub trait Detector: Send + Sync {
    fn id(&self) -> DetectorId;
    fn version(&self) -> Version;
    fn categories(&self) -> &[Category];
    /// Must be total: never panic across the boundary; errors become DetectorOutcome::Failed.
    fn scan(&self, input: &NormalisedText, ctx: &ScanContext) -> DetectorOutcome;
}
pub enum DetectorOutcome { Ok(Vec<Finding>), Failed { reason: FailureCode } }

pub struct Finding {
    pub category: Category, pub subtype: String,
    pub span: Span /* part_index + byte range, in normalised AND original coordinates */,
    pub severity: Severity, // Info < Low < Medium < High < Critical
    pub confidence: f32, pub detector: DetectorId, pub rule_version: Version,
    pub evidence_hash: [u8;16], // for dedupe; never the text
}

pub trait Classifier: Send + Sync {   // Stage 2; may only add/raise
    fn classify(&self, input: &ContextWindow) -> ClassifierOutcome;
    fn availability(&self) -> Availability;
}
pub struct CoverageReport { pub parts: Vec<PartCoverage> /* Inspected|Uninspected(reason) */, pub stage1: Ran|Failed, pub stage2: Ran|Unavailable|Skipped }
```

## 4. Policy engine

```rust
pub struct PolicyInput<'a> { pub direction: Direction, pub findings: &'a [Finding], pub coverage: &'a CoverageReport,
    pub mode: OperatingMode, pub route_candidates: &'a [RouteCandidateInfo], pub client: &'a ClientPolicyView,
    pub provenance_summary: ProvenanceSummary, pub policy: &'a EffectivePolicy }
pub struct Decision { pub action: Action /* Allow|Notify|Review|Block */, pub reasons: Vec<ReasonCode>,
    pub required_approvals: Vec<ApprovalKind /* Transform | Transmit | ToolCall */>,
    pub transform_plan: Option<TransformPlan>, pub min_route_constraints: RouteConstraints }
pub fn decide(input: &PolicyInput) -> Decision;   // pure, total, deterministic
```

`EffectivePolicy = merge(Mandatory, Profile@version, ClientOverride)` computed by `aegis-profiles`; merge rejects any attempt to relax a non-relaxable rule (returns `PolicyError`, daemon refuses to load).

## 5. Approval engine & control API (UI ↔ daemon)

Transport: length-prefixed JSON (or CBOR) over named pipe/UDS; request/response + server-push events.

```
→ events:   review.requested {review_id, request_id, original_ref, findings[], proposed_transform, route_preview, expires_at}
            request.state_changed {request_id, state}
← commands: review.decide {review_id, decision: Reject | ApproveTransform{edited_text?} | ApproveTransmit{payload_sha256}}
            review.get {review_id}            // content is delivered over the pipe, held in memory only
            policy.get/set, profiles.*, providers.*, clients.*, mode.set, vault.lock/unlock, audit.query (metadata), integrations.*
```

Rules: `ApproveTransmit` must echo the `payload_sha256` of the bytes the UI displayed; mismatch → rejected. Review content is held in memory only, zeroised after decision/timeout; never written to disk (I6). Only authenticated UI/NM peers can issue `review.decide`; the gateway-facing API cannot.

## 6. Provider adapter interface

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn locality(&self) -> Locality;                 // Local | Cloud  (Cloud ⇒ must use aegis-egress)
    fn capabilities(&self) -> Capabilities;         // tools, json_schema, vision(false in v1), max_ctx, models[]
    async fn health(&self) -> Health;
    /// `req` has already been inspected & approved. Returns the FULL buffered response.
    async fn complete(&self, req: ApprovedRequest<'_>) -> Result<RawProviderResponse, ProviderError>;
}
```

- `ApprovedRequest` embeds the `ApprovedOutbound` token for cloud providers; local providers receive a `LocalApproved` marker. The `Cloud` impls only compile against `aegis-egress::send`, which verifies the token.
- Adapters translate canonical ↔ wire format; unsupported features return `ProviderError::UnsupportedParameter{param}` (surfaced to the client verbatim).
- Ollama adapter: `/api/chat`; loopback-only HTTP client type; model digest recorded in audit metadata.
- OpenAI vs compatible: shared wire layer, per-provider quirks table (base URL, auth header, supported params, Responses-API availability).

## 7. Router

```rust
pub struct RouteDecision { pub chosen: ProviderId, pub model: String, pub considered: Vec<Considered{provider, verdict: Chosen|Rejected(ReasonCode)}>, pub explanation: Explanation /* structured; UI renders */ }
pub enum FailureHandling { Fail, ProposeFallback(Vec<ProviderId>) }   // never AutoFallback
```

## 8. Extension ↔ daemon (Native Messaging) protocol

JSON messages, versioned (`v:1`), each with `id`, `type`:

| Message | Direction | Purpose |
|---|---|---|
| `hello` / `hello_ack` | ext→host | version, adapter manifest list; host returns daemon status/mode/locked |
| `submit.inspect` | ext→daemon | `{site, adapter, text, history_window?, attachments_present}` → starts a review/inspection |
| `submit.decision` | daemon→ext | `{allow|block|replace_text{text, sha256}, tier, banner_state}` — extension may only act on `replace_text` with matching hash |
| `response.inspect` | ext→daemon | inbound text (buffered by adapter if Tier B) |
| `response.decision` | daemon→ext | release / redact / block |
| `status` | daemon→ext | mode, profile, locked, protection tier per site |

Extension never receives credentials, mapping tables, or other users' data; reconstruction for extension flows is performed in the daemon and the *reconstructed text* returned only for local display.

## 9. Audit event schema (metadata-only)

```sql
CREATE TABLE audit_event (
  id            INTEGER PRIMARY KEY,
  ts_utc        INTEGER NOT NULL,
  request_id    BLOB    NOT NULL,          -- ULID
  client_id     INTEGER NOT NULL,          -- registry FK, no names
  source_kind   INTEGER NOT NULL,          -- enum
  direction     INTEGER NOT NULL,          -- enum
  profile_id    INTEGER NOT NULL, profile_ver INTEGER NOT NULL,
  mode          INTEGER NOT NULL,
  action        INTEGER NOT NULL,          -- Allow/Notify/Review/Block
  reason_codes  INTEGER NOT NULL,          -- bitset/enums, no text
  category_counts BLOB NOT NULL,           -- fixed-size array of u16 per category
  max_severity  INTEGER NOT NULL,
  approvals     INTEGER NOT NULL,          -- bitset: transform/transmit/edited/rejected
  route_provider INTEGER, route_locality INTEGER,
  payload_sha256_prefix BLOB,              -- 8 bytes of approved-payload hash; no reversibility concern
  input_bytes   INTEGER, output_bytes INTEGER, latency_ms INTEGER
);   -- NO TEXT/BLOB-with-content columns. CI lints for this.
```

Hash note: even short hashes of low-entropy content can be brute-forced; only the *approved, sanitised* payload prefix is stored, and it can be disabled in privacy-max mode.

---

## 10. Review Center data contract (v1)

**Status:** published for the design session; **contract_version = 1**. Any change goes through an ADR **and** a message to the design session (ADR-0021), so neither side invents interfaces. Source requirements: PRD FR-POL-3/4/5/6/9, FR-DET-3, FR-UI-1, FR-BRW-3/5, FR-INB-4.

### 10.1 Principles

1. **The UI renders; the daemon decides.** The UI never computes severity, risk, diffs, hashes, coverage or approval requirements. It renders fields as delivered and sends decisions back.
2. **Snapshots, not patches.** Every update is a full `ReviewRequest` with a monotonically increasing `revision`. The UI replaces its copy; commands carry the `revision` they were made against, and stale commands are rejected (`stale_revision`).
3. **Content is in memory only** (invariant I6). It is delivered over the control channel (desktop) or Native Messaging (extension), never persisted by either client, and zeroised on `review.closed`.
4. **Credentials are never delivered.** Spans of credential-class findings are *pre-masked by the daemon* in every text the UI receives: replaced by `•` (U+2022), **same UTF-16 length**, so offsets stay valid. The UI cannot display, copy or transmit the real value.
5. **No matched text in metadata.** Findings carry locations, never the matched string. Explanations are static catalogue text.
6. **Offsets are UTF-16 code units** into the (masked) text of the referenced part, half-open `[start, end)`, so the UI can slice JS strings directly. The daemon converts from its internal byte offsets.
7. **Honest labelling.** Whatever Aegis could not inspect is delivered as `uninspected`, never omitted (invariant I8).

### 10.2 Types (TypeScript notation; generated from `aegis-core`)

```ts
type Iso8601 = string;                    // RFC 3339 UTC
type Sha256 = string;                     // lowercase hex, 64 chars

type ReviewState =
  | 'inspecting'                // brief; UI may show a spinner
  | 'awaiting_transform_decision'
  | 'reinspecting'              // after an edit or accepted proposal
  | 'awaiting_transmit_decision'
  | 'awaiting_uninspected_override'
  | 'awaiting_tool_call_decision'
  | 'allowed' | 'transmitting' | 'inbound_inspecting'      // progress only
  | 'released'                  // terminal: success
  | 'rejected'                  // terminal: user rejected
  | 'blocked'                   // terminal: mandatory block / fail-closed
  | 'response_blocked'          // terminal: inbound inspection blocked the response
  | 'expired'                   // terminal: approval timeout
  | 'cancelled'                 // terminal: client disconnected / page closed
  | 'failed';                   // terminal: internal/provider error (see errors[])

interface ReviewRequest {
  contract_version: 1;
  contract_minor: 1;                      // additive revisions; see 10.11
  review_id: string;                      // stable for the life of the request
  request_id: string;                     // same id as in audit metadata and API errors
  revision: number;                       // starts at 1, +1 on every change
  state: ReviewState;
  direction: 'outbound' | 'inbound' | 'tool_call';   // inbound/tool_call are P1
  server_time: Iso8601;                   // for clock-skew-safe countdowns
  created_at: Iso8601;
  expires_at: Iso8601 | null;             // null once terminal
  timeout: TimeoutInfo;
  source: SourceInfo;
  profile: { id: string; name: string; version: number };
  mode: 'local_only' | 'online_only' | 'hybrid' | 'auto';
  route: RoutePreview;
  coverage: Coverage;
  risk: RiskSummary;
  content: ContentView;                   // ORIGINAL, credential spans pre-masked
  findings: FindingView[];                // ordered: severity desc, then position
  proposal: Proposal | null;              // null when nothing is proposed (e.g. blocked)
  approvals: ApprovalSteps;
  consequences: Consequences;
  tool_call: ToolCallView | null;         // only when direction === 'tool_call' (P1)
  errors: ReviewError[];                  // non-fatal and fatal; see 10.7
}

interface RiskSummary {
  level: 'low' | 'moderate' | 'high' | 'critical';   // highest finding risk_level
  blocked: boolean;                       // true => nothing approvable
  mandatory: boolean;                     // an immutable rule decided
  reasons: { code: string; title: string }[];
}

interface ToolCallView {                  // P1
  name: string;
  risk_class: 'shell' | 'file_write' | 'file_delete' | 'network' | 'payment' | 'message_send' | 'other';
  arguments_preview: string;              // credential spans masked, truncated
}

interface TimeoutInfo {
  seconds_total: number;
  on_expiry: 'reject';                    // never auto-approve
  held_connection: boolean;               // true for API clients waiting on this decision
}

interface SourceInfo {
  kind: 'gateway' | 'extension';
  application_label: string;              // API client name, or e.g. "ChatGPT"
  site?: { host: string; adapter: string; adapter_version: string };
}

interface RoutePreview {
  locality: 'local' | 'cloud';
  provider_label: string;                 // "Ollama (this device)" / "OpenAI"
  model: string | null;
  explanation: { code: string; text: string };   // plain language, from router (ADR-0018)
  considered: { provider_label: string; verdict: 'chosen' | 'rejected'; reason_text?: string }[];
  cloud_consent: 'not_needed' | 'granted' | 'required';   // 'required' => consent step precedes review
  local_security_processing: true;        // constant: UI must distinguish this from local inference (FR-MOD-4)
}
```

### 10.3 Coverage, tier and UNINSPECTED

```ts
interface Coverage {
  integration: {
    tier: 'A' | 'B' | 'C' | 'D';
    label: 'protected' | 'partially_protected' | 'not_protected';   // FR-BRW-3: render verbatim
    outbound: 'enforced' | 'advisory' | 'none';
    inbound:  'enforced' | 'detect_only' | 'none';
    adapter?: { name: string; version: string; tested_on: Iso8601; canary: 'pass' | 'fail' | 'unknown' };
    limitations: string[];                // plain-language bullets, daemon-authored
  };
  parts: { part_id: string; status: 'inspected' | 'uninspected';
           reason?: 'image' | 'file' | 'audio' | 'unsupported_type' | 'size_limit' | 'decode_limit' }[];
  stage1: 'ran' | 'failed';               // 'failed' => blocked (fail closed)
  stage2: 'ran' | 'unavailable' | 'skipped';
  contextual_check_skipped_notice: boolean;   // true => show the "contextual check skipped" notice (FR-DET-8)
  uninspected_override: {
    available: boolean;                   // false on local routes and wherever policy forbids
    persistent_banner: 'UNINSPECTED';     // when granted, the banner stays on the item
    acknowledgement_key: string;          // copy key for the confirm text
  } | null;                               // null when everything was inspected
}
```

### 10.4 Content, findings, diff

```ts
interface ContentView {
  parts: ContentPart[];
  total_utf16_length: number;
  original_sha256: Sha256;                // hash of the exact ORIGINAL payload; present from the first snapshot (10.11)
}
interface ContentPart {
  part_id: string;
  role: 'system' | 'user' | 'assistant' | 'tool' | 'composer';  // 'composer' = extension text box
  provenance: 'user_typed' | 'user_pasted' | 'system' | 'assistant' | 'tool_output' | 'retrieved' | 'document';
  trusted: boolean;                       // false for anything except typed/system
  text: string;                           // masked where credential-class
  masked_spans: { start: number; end: number }[];
  truncated: boolean;                     // if true, fetch the rest via review.get_content
}

interface FindingView {
  finding_id: string;
  category: 'identity' | 'credential_secret' | 'financial' | 'confidential' | 'technical'
          | 'personal_record' | 'contextual_identity' | 'defence';
  subtype: string;                        // stable code, e.g. "openai_key", "email", "iban"
  severity: 'info' | 'low' | 'medium' | 'high' | 'critical';
  risk_level: 'low' | 'moderate' | 'high' | 'critical';   // mapping in 10.8; 'info' findings are not listed
  confidence: number;                     // 0..1
  confidence_band: 'low' | 'medium' | 'high';
  detector: { id: string; stage: 1 | 2; rule_version: string };
  location: { part_id: string; start: number; end: number } | null;
  via?: 'base64' | 'hex' | 'percent' | 'compat_fold' | 'invisible_chars';  // found inside an encoded/obfuscated
                                          // form; location then points at the ENCODED token in the original
  reason: { code: string; title: string; explanation: string };     // FR-DET-3: static, plain language
  handling: 'blocked' | 'proposed_change' | 'send_as_is' | 'keep_local';
  mandatory: boolean;                     // governed by an immutable rule
  approvable: boolean;                    // false => cannot be sent, whatever the user does
}

interface Proposal {
  proposal_id: string;
  proposal_version: number;               // +1 per engine proposal or user edit
  source: 'engine' | 'user_edited';
  parts: { part_id: string; text: string }[];             // the exact text that WOULD be sent
  changes: ChangeView[];                                  // what actually changed, and why
  diff: DiffOps[];                                        // per part, for side-by-side rendering
  final_payload: { sha256: Sha256; bytes: number };       // hash of the exact bytes to be transmitted
  preservation: { status: 'ok' | 'warning' | 'uncertain';
                  notes: { kind: 'number' | 'date' | 'negation' | 'quantifier' | 'exception' | 'other';
                           part_id: string; start: number; end: number; text: string }[] };   // FR-CTX-2
  reinspection: { state: 'not_needed' | 'pending' | 'clean' | 'new_findings';
                  new_finding_ids: string[] };          // after edits (FR-POL-5)
  reconstruction: { local_reconstruction_available: boolean; placeholders: string[] };  // labels only
}
interface ChangeView {
  change_id: string;
  kind: 'redact' | 'anonymise' | 'pseudonymise' | 'generalise' | 'remove' | 'keep_local';
  finding_ids: string[];
  part_id: string;
  original_span: { start: number; end: number };         // in the masked original
  proposed_span: { start: number; end: number } | null;   // null for removals
  replacement_label: string | null;                        // e.g. "[PERSON_1]"
  reversible_locally: boolean;                             // pseudonym => true; credentials never
  reason: { code: string; explanation: string };
}
interface DiffOps {
  part_id: string;
  ops: { op: 'equal' | 'delete' | 'insert' | 'replace';
         original: { start: number; end: number };         // empty range for insert
         proposed: { start: number; end: number };          // empty range for delete
         change_id?: string }[];
}
```

The things the Review Center must keep visually distinct map to distinct fields: **Original** = `content`; **Detected risks** = `findings`; **Proposed transformation** = `proposal.parts` + `proposal.diff`; **Actual changes** = `proposal.changes` (including after user edits, where `source = 'user_edited'`); **Approval to transform / to transmit** = the two `approvals.steps`.

### 10.5 Approvals (transform is not transmit) and hash binding

```ts
type ApprovalKind = 'transform' | 'transmit' | 'uninspected_override' | 'tool_call';

interface ApprovalSteps {
  steps: ApprovalStep[];                  // in required order
  current: ApprovalKind | null;           // the step awaiting the user
}
interface ApprovalStep {
  kind: ApprovalKind;
  status: 'not_required' | 'pending' | 'granted' | 'declined' | 'denied' | 'expired' | 'invalidated';
  strength: 'notify' | 'one_click' | 'explicit_confirm' | 'not_approvable';   // ADR-0008
  transmit_content: 'original' | 'transformed' | null;   // transmit step only: which text WILL BE SENT (10.11)
  required_acknowledgements: string[];    // finding_ids the user must tick (explicit_confirm, High)
  binds_to: { payload_sha256: Sha256 } | null;   // transmit / override: the exact bytes
  granted_at: Iso8601 | null;
  invalidated_reason?: 'edited' | 'reinspection_new_findings' | 'policy_changed';
}

interface Consequences {                  // FR-UI-1: consequences of approve / reject, as structured data
  if_approved: { transmits_to: string; locality: 'local' | 'cloud'; items_shared: number;
                 items_changed: number; items_kept_local: number; uninspected_parts: number }[];
  if_rejected: { request_sent: false; client_receives: { code: string; text: string } };
  remediation: { code: string; text: string }[];   // e.g. "Remove the key; rotate it if it was shared before"
}
```

Rules the UI can rely on:
- `transform` is `not_required` when no change is proposed or every finding may be sent as-is.
- `transmit` is always a separate, later step and always binds to `proposal.final_payload.sha256` (or the hash of the original when no transform). The UI **must echo that hash** in the decision; mismatch returns `hash_mismatch` and no token is minted.
- Any edit, accepted proposal, or policy change **invalidates** previously granted steps (`status: 'invalidated'`); the state returns to `reinspecting`.
- `not_approvable` means there is no approve action at all (mandatory block). The UI shows the explanation and remediation, not a disabled button.

### 10.6 Commands (UI to daemon) and events (daemon to UI)

Transport: control channel (desktop) / Native Messaging (extension), same payloads.

```ts
// daemon -> UI
'review.requested'  { review: ReviewRequest }
'review.updated'    { review: ReviewRequest }                       // revision strictly increases
'review.closed'     { review_id: string; outcome: 'released'|'rejected'|'blocked'|'response_blocked'|'expired'|'cancelled'|'failed' }

// UI -> daemon
'review.get'          { review_id }                                  -> { review }
'review.get_content'  { review_id, part_id, offset, limit }          -> { text, truncated }   // paging for the NM 1 MB cap
'review.decide'       { review_id, revision, decision: ReviewDecision } -> { ok: true, review } | { ok: false, error: ReviewError }

type ReviewDecision =
  | { type: 'reject' }
  | { type: 'approve_transform'; proposal_id: string; proposal_version: number }
  | { type: 'decline_transform' }                                    // keep original; proceed to transmit step
  | { type: 'edit_proposal'; parts: { part_id: string; text: string }[] }   // => reinspecting
  | { type: 'approve_transmit'; payload_sha256: Sha256; acknowledged_finding_ids: string[] }
  | { type: 'override_uninspected'; payload_sha256: Sha256; acknowledged: true }
  | { type: 'approve_tool_call'; payload_sha256: Sha256 };           // P1
```

Notes: the UI only ever holds masked credential text, so a user edit cannot carry the original secret back; if a user types something that looks like a secret, reinspection blocks it (AT-06). Allowances ("don't ask again", FR-POL-8, P1) are **reserved**: a future `create_allowance` decision will be added through an ADR.

### 10.7 Error and edge states

```ts
interface ReviewError {
  code:
    | 'stale_revision' | 'hash_mismatch' | 'proposal_superseded'
    | 'expired' | 'cancelled_by_client'
    | 'blocked_mandatory' | 'inspection_failed' | 'reinspection_failed'
    | 'vault_locked' | 'consent_required' | 'unsupported_content'
    | 'content_too_large' | 'provider_unavailable' | 'internal';
  fatal: boolean;                         // true => review will close; false => UI may retry/continue
  text: string;                           // plain language, static
  retryable: boolean;
  field?: string;                         // e.g. "decision.payload_sha256"
}
```

UI-local states the daemon cannot send (render without data): **daemon unreachable / Native Messaging disconnected** (show that Aegis is not running and requests are blocked; AT-14), and **malformed channel message** (dropped, AT-15). The UI shows `expires_at` as a countdown using `server_time` and treats reaching zero as `expired` only after the daemon confirms with `review.closed`.

### 10.8 Mappings the UI should not re-derive

| Daemon severity | `risk_level` | Default action (PRD FR-POL-1) |
|---|---|---|
| info | (not listed) | allow |
| low | low | notify only |
| medium | moderate | explain + offer sanitise |
| high | high | review before transmission (explicit confirm) |
| critical, mandatory-block category | critical | block, `not_approvable` |
| critical, other categories | critical | review (explicit confirm) |

### 10.9 Examples

Blocked secret (nothing approvable):

```json
{ "contract_version": 1, "review_id": "rv_01", "request_id": "01J-example", "revision": 1, "state": "blocked",
  "direction": "outbound", "expires_at": null,
  "risk": { "level": "critical", "blocked": true, "mandatory": true, "reasons": [{ "code": "critical_secret", "title": "API key detected" }] },
  "content": { "parts": [{ "part_id": "p0", "role": "user", "provenance": "user_typed", "trusted": true,
      "text": "Debug this, my key is ••••••••••••••••••••••••••••••••••••", "masked_spans": [{ "start": 23, "end": 59 }], "truncated": false }],
      "total_utf16_length": 59 },
  "findings": [{ "finding_id": "f1", "category": "credential_secret", "subtype": "openai_key", "severity": "critical",
      "risk_level": "critical", "confidence": 0.95, "confidence_band": "high",
      "detector": { "id": "secrets", "stage": 1, "rule_version": "secrets-0.1" },
      "location": { "part_id": "p0", "start": 23, "end": 59 }, "handling": "blocked", "mandatory": true, "approvable": false,
      "reason": { "code": "critical_secret", "title": "API key detected", "explanation": "Keys give access to your account. Aegis never sends them to any model." } }],
  "proposal": null,
  "approvals": { "steps": [], "current": null } }
```

Review of a personal-data prompt with an engine proposal: `state = "awaiting_transform_decision"`; `approvals.steps = [{kind:"transform", status:"pending", strength:"one_click"}, {kind:"transmit", status:"pending", strength:"one_click", binds_to:{payload_sha256:"..."}}]`; `proposal.source = "engine"`; `proposal.changes[0] = {kind:"pseudonymise", replacement_label:"[PERSON_1]", reversible_locally:true}`.

### 10.10 Known gaps this contract depends on (engineering, tracked)

- Finding locations in **original** coordinates, including `via` for decoded findings, require mapping decoded/normalised views back to original spans (extends S2.4; the M1 pipeline reports spans in view coordinates only).
- Preservation checker output (`proposal.preservation`) is S2.4.
- `route.considered` and `explanation` need the router (S3.2, now P0).
- Tier and `limitations` strings come from adapter manifests (S5.3).
- Risk reasons and finding explanations need a static catalogue (new story, see backlog S2.9).

### 10.11 Changelog

**v1.1 (additive, ADR-0023; `contract_minor = 1`).** Requested by the design session; existing v1.0 fields are unchanged.
- `ApprovalStep.transmit_content: 'original' | 'transformed' | null` on the `transmit` step states which text will be sent, so the UI no longer infers it. `'original'` when no transform was approved or proposed (or the user declined it); `'transformed'` when the proposal (engine or user-edited) will be sent. `null` on other step kinds.
- `ContentView.original_sha256` is present from the first snapshot (state `inspecting` onward), so the WILL BE SENT banner can show the original-text hash before any proposal exists. It is the SHA-256 of the exact original payload bytes, computed before masking; masked text is a display artefact only. For the transformed case the banner uses `proposal.final_payload.sha256`.
- **Reject-undo is dropped for v1.** Rejection is final: the held API request is released immediately with a `review_rejected` error and the extension is told to discard the send. An undo window would require holding or replaying the request after the user said no. Revisit only through a new ADR.

Design session: renders `transmit_content` verbatim and must treat an absent `contract_minor` as 0 (v1.0 daemon).
