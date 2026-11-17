# ADR-0012: Integration coverage tiers and adapter capability manifests

**Status:** Proposed

**Context.** Websites are not under Aegis' control; DOM changes, streaming transports and server-side state make guarantees site-specific. The principle "honest integration coverage" and acceptance #15 require transparency. Also note a conflict: "buffer complete response before release" is enforceable at the gateway but not generally in a web page.

**Decision.** Define tiers A (gateway), B (adapter with main-world network interception that can hold responses), C (DOM-only: outbound enforced at composer submit, inbound detect-only), D (unsupported). Each adapter ships a capability manifest (`outbound`, `inbound`, `uploads`, `tested_on`, `selectors_hash`) that the UI renders verbatim. Adapters are tested against recorded fixtures plus a live canary; canary failure auto-downgrades the tier and tells the user. Unsupported sites show a persistent "Not protected" state. The extension requests host permissions only for adapter domains.

**Consequences.** + Honest, testable claims. − Adapter maintenance is an ongoing cost (R-02); Tier B interception of site streams is fragile and may violate site ToS in edge cases (legal review item).

**Alternatives.** Generic "all sites" interception (overpromises, huge permission footprint); browser-level proxy/MITM (requires root CA install — rejected on security grounds).
