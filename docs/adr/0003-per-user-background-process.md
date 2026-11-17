# ADR-0003: Per-user background process, not a system service

**Status:** Proposed (approved input: "background service + desktop dashboard"; this ADR fixes the interpretation)

**Context.** DPAPI user scope, macOS Keychain and Linux Secret Service are per-user. OS-identity auth of clients is per-user. A system-wide service would need its own key custody and multi-user isolation.

**Decision.** `aegis-daemon` runs as the logged-in user, auto-started at login via Run key/Task Scheduler (Windows), LaunchAgent (macOS), `systemd --user` (Linux). One daemon per OS user; each has its own vaults, ports and pipes.

**Consequences.**
+ Natural fit for OS secret stores and identity; simple permissions.
− Protection starts at login, not boot; fast-user-switching means one daemon per active user; shared-machine/multi-user port allocation must avoid collisions (per-user port config).
− A user with admin rights can stop it (not in threat model; surfaced via status).

**Alternatives.** System service + per-user brokers — more moving parts and a bigger privileged TCB; revisit for enterprise (out of scope: individual users).
