# ADR-0001: Daemon-centric architecture with a single enforcement point

**Status:** Proposed

**Context.** Aegis has three entry points (extension, API gateway, desktop UI) and must provide consistent guarantees. If each embeds its own inspection logic, policies drift, and a compromised front-end can bypass enforcement.

**Decision.** One daemon (`aegis-daemon`) hosts the gateway, inspection pipeline, policy, approval, router, vault and egress. Extension, UI and third-party apps are clients. Only the daemon holds provider credentials and can reach cloud providers.

**Consequences.**
+ One TCB to audit and test; identical behaviour across integrations; UI crashes don't disable protection.
+ Clients are untrusted by design (aligns with threat model T7, T9).
− Daemon availability is critical → fail-closed clients, watchdog, autostart (see deployment).
− Extra IPC hop for extension flows (acceptable latency).

**Alternatives.** (a) Logic inside the Tauri backend — couples protection to UI lifecycle. (b) Logic inside the extension (WASM) — can't protect API traffic, can't hold credentials, weaker isolation.
