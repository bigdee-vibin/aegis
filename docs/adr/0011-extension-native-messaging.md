# ADR-0011: Extension ↔ daemon via Native Messaging relay

**Status:** Proposed

**Context.** The extension must reach the local daemon without exposing a TCP port to web pages (CORS/DNS-rebinding/CSRF risk) and without holding credentials.

**Decision.** MV3 extension uses `chrome.runtime.connectNative("com.aegis.bridge")`. `aegis-nmhost` is a tiny stdio relay (browser-launched) forwarding framed messages to the daemon control pipe/UDS. Manifest `allowed_origins` pins the extension ID. Daemon validates relay (peer creds, executable path/signature, pairing). The extension is an untrusted client: it submits text for inspection and acts only on daemon decisions (`replace_text` accepted only with matching hash). Approval UI is rendered in the extension popup/side panel or desktop app, never in page DOM. Permissions: per-site optional host permissions for tested adapters. Disconnected → adapters fail closed.

**Consequences.** + No browser-reachable network listener; clean trust boundary. − Per-browser/OS installer complexity; MV3 service-worker lifecycle (reconnect logic); 1 MB host→browser message cap (chunking); Snap/Flatpak browsers need special handling.

**Alternatives.** Localhost HTTP/WebSocket from extension (simpler install, but reachable by any page/process and needs token management); WASM engine in extension (duplicates TCB, can't enforce egress).
