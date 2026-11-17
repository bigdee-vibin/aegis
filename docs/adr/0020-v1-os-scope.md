# ADR-0020: v1.0 operating-system scope

**Status:** Accepted (2026-11-17), decided by the product owner.

**Decision.** Version 1.0 supports **Windows, macOS and Linux**.

**Context.** The architecture (CI matrix, Native Messaging installers, per-OS secure storage, packaging) already assumed three OSes. The handoff had left this open; the PM recommendation was Windows-only. The owner chose all three.

**Consequences.**
- Release gate: all acceptance tests (AT-01…AT-18) must pass on all three OSes in CI.
- Linux secure storage is the highest platform risk (R-05): master-password mode required when no Secret Service is available; no plaintext fallback.
- Native Messaging install/repair/uninstall must be verified on Chrome, Edge and Brave per OS (S5.1). Snap/Flatpak browsers stay documented as unsupported unless tested.
- M0 spikes S0.5 and S0.6 must produce results on all three OSes before M1 exits.
- Effort and schedule risk increase (R-15). Mitigation: OS-specific code stays behind the storage/identity/IPC abstractions; if a platform slips, the owner may revisit by a new ADR rather than shipping it untested.
- DPAPI is the Windows credential-protection mechanism only; macOS uses Keychain, Linux uses Secret Service or master password.
