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
