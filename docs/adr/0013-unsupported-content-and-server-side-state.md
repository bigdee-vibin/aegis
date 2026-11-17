# ADR-0013: Unsupported content, server-side tools and server-side state

**Status:** Proposed

**Context.** v1 inspects text only, yet APIs carry images, files, audio, server-held history (`previous_response_id`, `store`), and provider-executed tools (web search, code interpreter, file search, remote MCP). Aegis cannot inspect these; labelling them "safe" would violate the honesty principle.

**Decision.**
- Non-text parts → `Part::Unsupported`; `CoverageReport` marks them `Uninspected`. Cloud route: **blocked by default** with `aegis_unsupported_content`; the user may explicitly approve per request with an `UNINSPECTED` warning (final default pending OPEN-05). Local route: allowed with notice if the local model supports it.
- Responses API: `store` forced to `false`; `previous_response_id`/`conversation` rejected; Aegis keeps no server-side dependence. Server-side tools rejected (`aegis_unsupported_server_tool`).
- Client-executed function tools are supported and inspected in both directions (arguments in responses, results in subsequent requests).
- Structured outputs/`response_format` forwarded when the adapter supports them; otherwise explicit error.

**Consequences.** + No false assurance. − Reduced functionality for some Responses-API clients; migration guidance in docs; revisit when multimodal inspection ships (P2).

**Alternatives.** Pass through with a warning (violates fail-closed for cloud), or strip silently (violates "no silent modification").
