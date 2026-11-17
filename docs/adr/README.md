# Architecture Decision Records

Format: Context → Decision → Consequences → Alternatives. Status: Proposed (pending review) | Accepted | Superseded.
Items marked **Approved input** restate decisions already made by the PM; others are architect proposals needing review.

| ADR | Title | Status |
|---|---|---|
| [0001](0001-daemon-centric-enforcement.md) | Daemon-centric architecture with a single enforcement point | Proposed |
| [0002](0002-rust-core-tauri-ui-monorepo.md) | Rust core, Tauri/React UI, TypeScript extension, monorepo | Proposed (UI stack: Approved input) |
| [0003](0003-per-user-background-process.md) | Per-user background process, not a system service | Proposed |
| [0004](0004-canonical-schema-and-adapters.md) | Canonical schema + provider adapters | Approved input, detailed |
| [0005](0005-deterministic-policy-monotonic-ai.md) | Deterministic policy engine; AI classifiers are monotonic | Proposed |
| [0006](0006-capability-token-egress-guard.md) | Capability-token egress guard | Proposed |
| [0007](0007-full-buffer-response-inspection.md) | Full-buffer response inspection with synthetic streaming | Approved input, detailed |
| [0008](0008-approval-state-machine.md) | Approval state machine and re-inspection | Proposed |
| [0009](0009-storage-and-encryption.md) | SQLite storage, key hierarchy, audit-by-construction | Approved input, detailed |
| [0010](0010-client-authentication.md) | Hybrid client authentication | Approved input, detailed |
| [0011](0011-extension-native-messaging.md) | Extension ↔ daemon via Native Messaging relay | Proposed |
| [0012](0012-integration-coverage-tiers.md) | Integration coverage tiers and adapter capability manifests | Proposed |
| [0013](0013-unsupported-content-and-server-side-state.md) | Unsupported content, server-side tools and state | Proposed |
| [0014](0014-pseudonymisation-and-reconstruction.md) | Pseudonymisation and local reconstruction scope | Proposed |
| [0015](0015-local-classifier-strategy.md) | Local classifier strategy | Proposed |
| [0016](0016-updates-and-zero-telemetry.md) | Updates and zero telemetry | Proposed (tension flagged) |
| [0017](0017-layered-policy-and-profiles.md) | Layered policy; profiles cannot override mandatory rules | Approved input, detailed |
| [0018](0018-constraint-first-routing.md) | Constraint-first routing with explanations | Proposed |
| [0019](0019-open-decisions-resolution.md) | Resolution of OPEN-01 … OPEN-10 | Accepted |
| [0020](0020-v1-os-scope.md) | v1.0 OS scope: Windows, macOS, Linux | Accepted |
