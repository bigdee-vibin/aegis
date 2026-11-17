# Tester report 1: M1 gateway (m0-m1-scaffold @ a899794) and design board (design/board-v1 @ b514251)

Independent AI tester, run on Windows 11. Branches tested as pushed. Not run: MSVC, Linux, macOS, CI, cargo-deny, the daemon binary, a real detector panic, screen readers.

## Build/test
| Item | Result | Evidence |
|---|---|---|
| Unpinned `cargo test --workspace` | FAIL on this host | GNU toolchain: `dlltool.exe not found` (windows-sys 0.61.2); MSVC toolchain: no `link.exe` (environment) |
| Pinned scratch copy (axum 0.8.1, tokio 1.43.1, mio 1.0.3, not in repo) | PASS | 51 tests, 0 failed, 0 ignored; clippy -D warnings clean |
| Unpinned clippy and fmt --check | PASS | exit 0 |

## Checklist
| ID | Status (pinned build) | Note |
|---|---|---|
| A1 key blocked, upstream count 0 | PASS | also over a real socket |
| A2 obfuscated secrets (9 forms) | PASS | asserts only HTTP 403, not the reason code |
| A3 detector panic / zero detectors fail closed | PASS | only the injected test panic exercised |
| A4 audit failure blocks | PASS | |
| A5 no prompt text in audit | PASS with limits | JSONL stand-in only (AT-17 not passed); logs/stderr not covered |
| A6 stream/non-text rejected, 401, 413 | PASS | |
| A7 tests are meaningful | PASS with notes | mutation test (Block treated as Allow) broke 6/13 slice tests; `audit_failure_blocks_the_request` unaffected |
| A8 LICENSE, CI file, deny.toml | PASS with limits | YAML validity and cargo-deny not verified |
| A9 secrets/personal data in repo | PASS, 1 note | only synthetic strings; owner email in commit metadata (since rewritten on the branch) |
| B1 15 screens render, no console errors | FAIL | `blockedCount is not defined` (app.js:46) on every load |
| B2 Review Center rules | FAIL | D1-D6 |
| B3 dialogs respond | PASS | |
| B4 keyboard | FAIL | D7 |
| B5 375 px no horizontal scroll | PASS | 7 px overflow at 1024 px (D9) |
| B6 themes readable | PASS with note | small muted text near 3.8-4.4:1 by rough heuristic |
| B7 contrast script | PASS | 50 pass, 0 fail; needs CSS path argument |

## Defects
- D1 medium: dismissing blocked R-2040 crashes (`diffIns`, review.js:37, `proposed` null); UI stale.
- D2 high (security UX): rejected uninspected request R-2037 still offers "Send uninspected" and can send ORIGINAL.
- D3 medium: uninspected acknowledgement persists after Cancel/Esc.
- D4 medium: approval ignores a failed re-inspection after an edit adds an email.
- D5 medium: "Approve and send" enabled while the editor is open on an approved item.
- D6 low: expiry suspended while editing; contradictory banner wording when proposed equals original; no visible re-inspection step.
- D7 medium: dialog focus not moved, not trapped, not restored.
- D8 low: key toast says copy it but no copy control; displayed port 8742 vs daemon default 8731.
- D9 low: 7 px overflow at 1024 px; loads Google Fonts (external request in a zero-telemetry product).
- D10 low: unpinned workspace cannot build on Windows GNU; document pins or require MSVC.
