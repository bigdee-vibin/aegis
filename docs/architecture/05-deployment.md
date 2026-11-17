# Aegis — Cross-Platform Deployment Strategy

## 1. Artifacts

| Artifact | Contents | Notes |
|---|---|---|
| Desktop installer | Tauri UI + `aegis-daemon` + `aegis-nmhost` + rule packs | One installer per OS; daemon and UI are separate executables |
| Browser extension | MV3 package | Chrome Web Store / Edge Add-ons; side-load for dev. Fixed extension ID (needed for NM `allowed_origins`) |
| Rule packs | signed `rules/*.pack` | Updated only through the approved-update flow |

Build: GitHub Actions matrix (windows-latest, macos-14 arm64 + macos-13 x64, ubuntu-22.04), reproducible builds as a goal, `cargo-auditable` SBOM, `cargo deny`, `cargo vet` for crypto-adjacent crates. Signing: Authenticode (Windows), Developer ID + notarisation (macOS), detached GPG + AppImage/deb/rpm signatures (Linux).

## 2. Process model

- `aegis-daemon` is a **per-user** background process, not a system service (ADR-0003): DPAPI, Keychain and Secret Service are per-user stores; a system service would need a different trust model and complicates "OS identity" client auth.
- Autostart: Windows — HKCU Run key or Task Scheduler "at logon"; macOS — LaunchAgent plist in `~/Library/LaunchAgents`; Linux — `systemd --user` unit (fallback: XDG autostart `.desktop`).
- Single instance via lock file / named mutex. Watchdog: UI shows daemon health; if the daemon is down, **clients fail closed** (gateway refuses connections; extension blocks submit on protected sites with an explanatory message).
- Control channel (UI ↔ daemon, NM relay ↔ daemon): Windows named pipe `\\.\pipe\aegis-<user-SID>` with an ACL restricted to the user SID; macOS/Linux Unix domain socket `0600` in a user-private directory (`$XDG_RUNTIME_DIR/aegis/` / `~/Library/Application Support/Aegis/run/`). Peer credentials checked (`SO_PEERCRED` / `LOCAL_PEERCRED` / `GetNamedPipeClientProcessId`). The control channel is **never** exposed over TCP.
- Gateway: TCP `127.0.0.1:<port>` (default 8731, configurable). LAN bind requires explicit toggle + TLS + non-default confirmation + per-key allow-list of source CIDRs.

## 3. Secure storage per OS

| OS | Root-key protection | Library | Gotchas |
|---|---|---|---|
| Windows | DPAPI (`CryptProtectData`, user scope), optionally with entropy = master-password KDF output | `windows` crate | Roaming profiles; DPAPI keys are lost on password reset by admin → recovery-key export flow |
| macOS | Keychain item (`kSecAttrAccessibleWhenUnlockedThisDeviceOnly`), ACL to the signed daemon binary | `security-framework` | Code-signing identity changes invalidate ACL → prompt on update; notarised builds |
| Linux | Secret Service (libsecret / D-Bus: GNOME Keyring, KWallet) | `secret-service`/`oo7` | **Headless/minimal desktops lack a Secret Service** → require master password mode there; never silently fall back to plaintext key file |

If OS storage is unavailable: refuse to create the vault until the user picks master-password mode. Master password: Argon2id (params tuned ≥ 250 ms on target hardware), never stored; optional recovery key shown once.

## 4. Browser Native Messaging

Per-browser, per-OS registration, installed/removed by the desktop installer and re-verifiable from Settings → Integrations ("Repair").

Host name: `com.aegis.bridge`. Manifest `allowed_origins`: only the Aegis extension ID(s).

| Browser | Windows | macOS (`~/Library/Application Support/…`) | Linux (`~/.config/…`) |
|---|---|---|---|
| Chrome | `HKCU\Software\Google\Chrome\NativeMessagingHosts\com.aegis.bridge` → manifest path | `Google/Chrome/NativeMessagingHosts/` | `google-chrome/NativeMessagingHosts/` |
| Edge | `HKCU\Software\Microsoft\Edge\NativeMessagingHosts\…` | `Microsoft Edge/NativeMessagingHosts/` | `microsoft-edge/NativeMessagingHosts/` |
| Brave | `HKCU\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\…` | `BraveSoftware/Brave-Browser/NativeMessagingHosts/` | `BraveSoftware/Brave-Browser/NativeMessagingHosts/` |
| Chromium | `HKCU\Software\Chromium\NativeMessagingHosts\…` | `Chromium/NativeMessagingHosts/` | `chromium/NativeMessagingHosts/` |

(Snap/Flatpak browsers on Linux sandbox the NM path and need a different location/portal; listed as a known limitation in v1.)

Lifecycle:
1. Install: write manifest + register; record in `config.db` which browsers were registered.
2. Pairing: first connection performs a one-time pairing (user confirms a short code in the desktop UI) → per-install pairing token stored in the extension's `storage.local`-independent secure place is *not* possible; so pairing authenticates the **NM relay binary + extension ID** (the browser guarantees the origin) and the daemon checks the relay's executable path/signature. The extension never obtains provider credentials or API keys.
3. Service-worker lifetime: MV3 workers sleep; the extension uses `connectNative` ports with reconnect/backoff; while disconnected, adapters **fail closed** (submit blocked with "Aegis unavailable").
4. Protocol limits: host→browser messages ≤ 1 MB — large transformed texts are chunked.
5. Uninstall removes manifests/registry keys and the daemon autostart; vault deletion is a separate explicit user choice.

## 5. Updates and telemetry (ADR-0016)

- No telemetry, no crash reporting, no analytics SDKs. CI fails if a dependency adds a network client outside `aegis-egress`.
- Update checks are **user-initiated or opt-in** (first-run choice, default off — see OPEN-02). A check fetches a static, signed manifest from a pinned host: no identifiers, no user-agent version fingerprinting beyond the version in the request path. Updates are *notified, never auto-installed*; installer signatures verified before launch; rule packs updated through the same flow.
- Extension updates go through the browser store's own mechanism (outside Aegis' control; documented).

## 6. Distribution risks to plan for

- macOS notarisation + Gatekeeper; Windows SmartScreen reputation; Linux packaging matrix (deb/rpm/AppImage) and Secret Service availability.
- Chrome Web Store review for an extension with `nativeMessaging` + broad host permissions: keep host permissions to the curated adapter domains (optional permissions requested per site) — which also supports the "honest coverage" story.
