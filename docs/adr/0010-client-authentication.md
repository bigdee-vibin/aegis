# ADR-0010: Hybrid client authentication

**Status:** Approved input (API keys + OS identity), detailed here

**Decision.**
- **API keys** per application: `aegis_<appid>_<secret>`; stored only as HMAC/Argon2id hashes; scoped (routes, modes, profile, rate limit, `on_review` behaviour); revocable; shown once at creation.
- **OS identity** as a second factor where the OS can provide it: loopback TCP peer → PID → executable path/signer (Windows `GetExtendedTcpTable`, Linux `/proc/net/tcp` + inode, macOS `libproc`). A key may be *bound* to an executable identity; mismatch → reject. Unix-socket/named-pipe endpoints (control plane) use true peer credentials.
- Control plane (UI, NM relay) is never reachable via TCP; gateway keys cannot call control endpoints.
- LAN mode: TLS required, key + source-CIDR allow-list, OS identity not available (clearly stated).
- Rate limits (token bucket per key), payload limits, request validation before any inspection work.

**Consequences.** + Misuse by other local processes becomes detectable/containable. − PID→exe mapping on loopback TCP is racy and best-effort; same-user malware can read an app's stored key. We therefore state OS identity as *defence in depth*, not a strong guarantee.

**Alternatives.** mTLS to localhost (heavier setup for SDK users); Unix-socket-only gateway (breaks standard SDK usage on Windows).
