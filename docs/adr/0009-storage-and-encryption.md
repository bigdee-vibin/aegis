# ADR-0009: SQLite storage, key hierarchy, audit-by-construction

**Status:** Approved input (SQLite; OS encryption + optional master password), detailed here

**Decision.**
- Three SQLite databases (`config.db`, `vault.db`, `audit.db`), opened with SQLCipher. Separate files = separate keys, backup rules and blast radius.
- Key hierarchy: OS secret store (DPAPI / Keychain / Secret Service) protects a random 256-bit **Root Key**. If a master password is enabled, an Argon2id-derived KEK additionally wraps the Root Key (both factors needed). Per-DB keys and the vault's key-wrapping key derive from the Root Key via HKDF with domain separation.
- Pseudonym mappings: field-level XChaCha20-Poly1305 with per-conversation DEKs (wrapped by the vault key); TTL (default 24 h) + secure sweep; delete-on-conversation-end option.
- Audit: typed `AuditEvent` API with no string payload; schema without free-text columns; CI lint + canary-string scan; log fields allow-listed.
- Locked state: when the master password is not entered, the daemon is `Locked` and blocks everything needing the vault with `aegis_locked`.
- Recovery key (optional, shown once) for DPAPI loss scenarios; documented trade-off.

**Consequences.** + Compromise of one store doesn't expose others; audit leakage is structurally hard. − SQLCipher build/packaging overhead; key rotation tooling needed; master-password UX (lock/unlock, timeouts).

**Alternatives.** Single DB (simpler, larger blast radius); OS-only crypto without app-level AEAD (can't do per-session crypto-shredding).
