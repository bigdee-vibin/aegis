# ADR-0016: Updates and zero telemetry

**Status:** Proposed — **tension flagged for PM decision (OPEN-02)**

**Context.** Requirements: zero outbound telemetry by default, *and* "notify and require user approval" for updates. Checking for an update is itself an outbound request that reveals IP and timing to the update host.

**Decision (proposed).**
- No telemetry/crash reporting/analytics in any component; no third-party SDKs with network capability; CI enforces via egress allow-list tests.
- Update checks are **opt-in at first run (default: off / manual "Check now")**. When on: fetch a static signed manifest from a pinned host, no identifiers; show release notes; user approves download; verify signature before install. Rule packs follow the same path.
- Local notification when the manifest indicates a security-critical update (only if checks enabled).

**Consequences.** + Honest zero-telemetry claim. − Users who never enable checks may run outdated rule packs/software; mitigated by periodic in-app reminder (local, time-based) to check manually.

**Alternatives.** Default-on check (violates "zero outbound by default" literally); distribute only through OS package managers/stores (offloads trust, reduces control).
