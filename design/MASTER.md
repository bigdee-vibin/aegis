# Aegis design: MASTER rules and evidence

Status date: 2026-11-17. Scope: design board prototype in `design/`. Sample data only.

## 1. Rules (apply everywhere)
- **Consent:** approving a transformation and approving transmission are separate controls and separate `approvalKind` values (transform, transmit). Approve-and-send stays locked until the transformation is approved and re-inspection passes.
- **Hash binding:** the transmit approval is bound to the `contentHash` of the exact text shown. Any edit changes the hash, clears approval and triggers re-inspection (`edited-reinspecting`).
- **What leaves the device:** every review shows one unmistakable banner: TRANSFORMED TEXT WILL BE SENT, ORIGINAL TEXT WOULD BE SENT: NOT INSPECTED, or NOTHING WILL BE SENT. Icon + words + colour, never colour alone.
- **Blocked-critical:** no approve, edit or override control exists, only dismiss and remediation steps.
- **Uninspected:** per-request override only, requires an explicit acknowledgement, sends ORIGINAL, applies to one request.
- **Expiry:** open reviews carry a visible countdown (`expiresAt`); on expiry nothing is sent.
- **Coverage vocabulary:** Protected, Partially protected, Not protected, Disconnected, Needs attention. Never imply coverage that is not there.
- **Status:** always icon + text label; colour is reinforcement.
- **Tokens:** `design/tokens/aegis.tokens.json` (DTCG), `design/tokens/shadcn-theme.css`, runtime in `design/aegis.css`. Teal = active protection/primary, amber = review, red = critical/blocked, green = verified, grey = neutral. Control edges use `--edge` (>=3:1), decorative hairlines use `--border`.
- **Type:** system font stack first (SF Pro / Segoe UI Variable), Inter fallback; mono for IDs, hashes, paths. Tighter tracking on large type.
- **Targets:** >=36 px desktop; >=44 px at <=760 px.
- **Motion:** short, springy only for toggles/sheets; `prefers-reduced-motion`, `prefers-reduced-transparency` and `prefers-contrast` handled in CSS.

## 2. Review Center data: contract_version 1 (real)
Source: `docs/architecture/03-api-contracts.md` section 10 (branch `m0-m1-scaffold`, PR #3). `design/review.js` renders `ReviewRequest` snapshots as delivered and sends `review.decide` commands carrying `revision`, `proposal_version` and `payload_sha256`. The UI does not compute severity, risk, diff, hash, coverage or approval requirements (10.1).
`review.js` contains a **mock daemon** (clearly separated, not product logic) that emits contract-shaped sample snapshots and answers decisions, including `stale_revision`, `hash_mismatch`, `consent_required`, `proposal_superseded`. Its hashes are fake (not SHA-256), and its diff builder is a toy.
Rendered: all `ReviewState` values used by the samples, coverage label verbatim, tier, limitations, uninspected parts, `contextual_check_skipped_notice`, route preview (locality, provider, model, explanation, considered), findings (category/subtype, risk_level, confidence band, handling, mandatory, approvable), proposal parts + diff ops + changes + preservation + reinspection + reconstruction, approval steps with strength and `required_acknowledgements`, uninspected override, consequences and remediation, timeout from `expires_at` (zero is treated as expired only after the daemon confirms), content cleared on terminal states (I6), daemon-unreachable UI-local state.
Not rendered or not covered: `direction: tool_call` (P1), `awaiting_tool_call_decision`, `review.get_content` paging for truncated parts, malformed-message state, `cancelled`/`failed` samples.
**Assumptions needing architecture confirmation:**
1. The contract has no explicit "will send original or transformed" field. The UI infers it from the transform step (`declined`/`not_required` or no proposal = original; otherwise transformed). Proposed: a field such as `transmit_content: 'original'|'transformed'`.
2. The contract has no hash of the original text (the transmit step `binds_to` carries it after a decline). The UI only displays what `binds_to` provides.
3. `server_time` skew handling: the prototype assumes client and mock clocks agree.
4. Reject undo (30 s) is NOT in the contract and was removed from the Review Center.

## 3. Measured contrast (WCAG 2.2, computed by `tools/contrast.py`)
Method: relative luminance per WCAG; translucent tints are alpha-blended over the card colour. Text needs 4.5:1; focus ring and control edges need 3:1 (1.4.11).
| theme | foreground | background | ratio | needs | result |
|---|---|---|---|---|---|
| dark | text | bg | 17.31 | 4.5 | PASS |
| dark | text | card | 15.18 | 4.5 | PASS |
| dark | text2 | bg | 7.39 | 4.5 | PASS |
| dark | text2 | card | 6.49 | 4.5 | PASS |
| dark | text2 | bg2 | 6.77 | 4.5 | PASS |
| dark | text3 | card | 4.84 | 4.5 | PASS |
| dark | text3 | bg | 5.51 | 4.5 | PASS |
| dark | text3 | bg2 | 5.05 | 4.5 | PASS |
| dark | accent-text | card | 8.93 | 4.5 | PASS |
| dark | accent-text | accent-soft/card | 7.01 | 4.5 | PASS |
| dark | accent-ink on accent (primary button) |   | 9.18 | 4.5 | PASS |
| dark | warn | card | 9.96 | 4.5 | PASS |
| dark | warn | warn-soft/card | 7.76 | 4.5 | PASS |
| dark | crit | card | 6.01 | 4.5 | PASS |
| dark | crit | crit-soft/card | 5.09 | 4.5 | PASS |
| dark | ok | card | 8.65 | 4.5 | PASS |
| dark | ok | ok-soft/card | 6.83 | 4.5 | PASS |
| dark | info | card | 7.7 | 4.5 | PASS |
| dark | info | info-soft/card | 6.13 | 4.5 | PASS |
| dark | text | add/card (diff ins) | 10.94 | 4.5 | PASS |
| dark | text | del/card (diff del) | 12.04 | 4.5 | PASS |
| dark | focus ring accent | bg (non-text 3:1) | 10.19 | 3.0 | PASS |
| dark | focus ring accent | card (3:1) | 8.93 | 3.0 | PASS |
| dark | edge (control border) | bg2 (3:1) | 4.4 | 3.0 | PASS |
| dark | edge | card (3:1) | 4.22 | 3.0 | PASS |
| light | text | bg | 16.49 | 4.5 | PASS |
| light | text | card | 17.85 | 4.5 | PASS |
| light | text2 | bg | 7.0 | 4.5 | PASS |
| light | text2 | card | 7.58 | 4.5 | PASS |
| light | text2 | bg2 | 6.56 | 4.5 | PASS |
| light | text3 | card | 5.44 | 4.5 | PASS |
| light | text3 | bg | 5.03 | 4.5 | PASS |
| light | text3 | bg2 | 4.71 | 4.5 | PASS |
| light | accent-text | card | 7.32 | 4.5 | PASS |
| light | accent-text | accent-soft/card | 6.46 | 4.5 | PASS |
| light | accent-ink on accent (primary button) |   | 5.47 | 4.5 | PASS |
| light | warn | card | 6.33 | 4.5 | PASS |
| light | warn | warn-soft/card | 5.48 | 4.5 | PASS |
| light | crit | card | 6.57 | 4.5 | PASS |
| light | crit | crit-soft/card | 5.67 | 4.5 | PASS |
| light | ok | card | 5.87 | 4.5 | PASS |
| light | ok | ok-soft/card | 5.1 | 4.5 | PASS |
| light | info | card | 6.27 | 4.5 | PASS |
| light | info | info-soft/card | 5.5 | 4.5 | PASS |
| light | text | add/card (diff ins) | 14.6 | 4.5 | PASS |
| light | text | del/card (diff del) | 14.62 | 4.5 | PASS |
| light | focus ring accent | bg (non-text 3:1) | 5.06 | 3.0 | PASS |
| light | focus ring accent | card (3:1) | 5.47 | 3.0 | PASS |
| light | edge (control border) | bg2 (3:1) | 3.49 | 3.0 | PASS |
| light | edge | card (3:1) | 4.03 | 3.0 | PASS |

FAILS: 0

Fix made during measurement: control borders originally used `--border-strong` (1.8 to 2.1:1, failing 1.4.11); added `--edge` (dark `#6F8299`, light `#708096`), now 3.5 to 4.4:1.

## 4. Per-screen status
| Screen | Built | Browser-checked 1440 | Contrast-measured (tokens) | 375 px checked |
|---|---|---|---|---|
| Review Center (contract v1, mock daemon) | yes | yes + scripted state-machine test (approve, edit, ack, hash, override, expiry) | tokens only | yes, all 6 sample reviews, no overflow, targets >=44 px |
| Overview, Inspector, Profiles, Router | yes | yes | yes | yes (no overflow) |
| Analytics, Integrations, Settings | yes | yes | yes | yes (no overflow) |
| States, Extension popup, Onboarding, Flows, Board, System, Handoff | yes | yes | yes | yes (no overflow) |

"375 px checked" = programmatic: no horizontal overflow outside scroll containers, no target under 43 px, plus screenshots of Review Center and Extension popup. Not every screen was eyeballed at 375.

## 5. Not checked (be honest)
- No screen-reader testing (NVDA, VoiceOver); no formal WCAG audit.
- Keyboard: tab order verified structurally only (all interactive elements named, no positive tabindex, one h1 and one main per screen); focus visibility verified from CSS and the design-system focus state, not by tabbing through every screen. Dialog focus trapping and focus return are not implemented in the prototype.
- Contrast was measured on tokens and tint combinations, not on every rendered pixel pair; light-theme screenshots were not reviewed.
- 1024 and 768 px layouts were not measured numerically.
- Reduced-motion and reduced-transparency rules exist in CSS but were not exercised in a browser.
- Figma: none, this is a coded board.

## 6. Tester report 1 defects (2026-11-17): fixes and regression evidence
Regression checks were scripted in the browser against the fixed build (mock daemon, sample data). Result = observed.
| ID | Fix | Evidence |
|---|---|---|
| D2 high | Terminal states render no send, override or approve control; daemon refuses decisions on closed reviews | Rejected R-2037: state `rejected`, 0 send controls; forced override click leaves state `rejected` |
| D1 | Blocked items get a UI-local "Dismiss from queue"; removed the old diff code path | Dismiss R-2040: no error, selection moves to next item |
| D3 | Acknowledgement reset on every dialog open, close, Esc and selection change | Tick, Esc, reopen: unticked and button disabled |
| D4 | Approve transformation and Approve and send stay locked unless `proposal.reinspection.state` is `clean`/`not_needed`; mock daemon also refuses (`reinspection_failed`); stepper shows "Re-inspection failed" | Edit adding an email: both disabled, daemon refusal observed |
| D5 | All approval controls disabled while the editor is open | Both disabled during edit |
| D6 | Mock daemon confirms expiry even while editing and closes the editor; banner reads "TEXT IDENTICAL TO THE ORIGINAL WILL BE SENT, NO NET CHANGE" when proposed equals original; visible 4-step stepper | Expired while editing; identical-text banner |
| D7 | On open, focus moves into the dialog and background is `inert`; Tab and Shift+Tab wrap; Esc closes; focus returns to the opener | Scripted keydown events; not tested with a screen reader |
| B1 | Removed the premature render at script load | Overview renders, no console error |
| D8 | New-key dialog shows the key once with a Copy button (clipboard); gateway port changed to 8731 (daemon default per tester; confirm) | Copy control present; no 8742 left |
| D9 | Removed Google Fonts links; system font stack; no external requests | `performance` resource list contains no cross-origin entries; no horizontal overflow on any of the 15 screens at 1024 px (I could not identify the original 7 px element, so cannot say which change fixed it) |
| README | States that `contrast.py` needs the CSS path | README updated |
Not addressed: D10 (Rust workspace build on Windows GNU) is outside the design scope. B6 note (small muted text 3.8-4.4:1 by rough heuristic): the script measures `--text3` at 4.5:1 or better on card, bg and bg2; not re-measured on rendered pixels.
