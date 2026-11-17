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

## 2. Review Center data: contract_version 1, contract_minor 1 (real)
Source: `docs/architecture/03-api-contracts.md` section 10 (branch `m0-m1-scaffold`, PR #3). `design/review.js` renders `ReviewRequest` snapshots as delivered and sends `review.decide` commands carrying `revision`, `proposal_version` and `payload_sha256`. The UI does not compute severity, risk, diff, hash, coverage or approval requirements (10.1).
`review.js` contains a **mock daemon** (clearly separated, not product logic) that emits contract-shaped sample snapshots and answers decisions, including `stale_revision`, `hash_mismatch`, `consent_required`, `proposal_superseded`. Its hashes are fake (not SHA-256), and its diff builder is a toy.
Rendered: all `ReviewState` values used by the samples, coverage label verbatim, tier, limitations, uninspected parts, `contextual_check_skipped_notice`, route preview (locality, provider, model, explanation, considered), findings (category/subtype, risk_level, confidence band, handling, mandatory, approvable), proposal parts + diff ops + changes + preservation + reinspection + reconstruction, approval steps with strength and `required_acknowledgements`, uninspected override, consequences and remediation, timeout from `expires_at` (zero is treated as expired only after the daemon confirms), content cleared on terminal states (I6), daemon-unreachable UI-local state.
Not rendered or not covered: `direction: tool_call` (P1), `awaiting_tool_call_decision`, `review.get_content` paging for truncated parts, malformed-message state, `cancelled`/`failed` samples.
**Assumptions needing architecture confirmation:**
1. (Resolved, contract v1.1) The banner renders `ApprovalStep.transmit_content` verbatim. Only a daemon without `contract_minor` (v1.0) is inferred, and the banner then says so. If a v1.1 daemon sends `null`, the UI shows "SEND CONTENT NOT STATED BY AEGIS" and advises rejecting.
2. (Resolved, contract v1.1) The original hash comes from `content.original_sha256`, present from the first snapshot.
3. `server_time` skew handling: the prototype assumes client and mock clocks agree.
4. (Resolved by contract v1.1, ADR-0023) Reject is final; undo removed everywhere.

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

## 7. QA verification 1 follow-ups (design/contract-v1.1)
| ID | Fix | Evidence (scripted, mock daemon) |
|---|---|---|
| D9 | Tooltip bubble is `display:none` until hover/focus; the theme-toggle variant is right-aligned | scrollWidth equals viewport at 1024 px on all 15 screens |
| N1 | `edit_proposal` timer returns if the review is already closed | Edit then expire during re-inspection: stays `expired` |
| N3 | Banner reads NOTHING WILL BE SENT WHILE RE-INSPECTION RUNS / UNTIL THE EDIT IS FIXED when re-inspection is pending or failed | Edit adding an email: banner changes, send locked |
| N4 | Focus moves to the first acknowledgement checkbox (else Approve and send) after approving the transformation; the stepper and spinner stay visible while re-inspecting | document.activeElement checked; stepper present during reinspecting |
| N5 | A warning banner says the unsaved edit was discarded when the review expires while editing | Banner text present |
| N2 | Unknown or out-of-context `rv:` actions are ignored | Injected actions leave state unchanged |

## 8. tool_call (P1) and malformed-message states, against contract_version 1.1
- **tool_call:** `direction: 'tool_call'`, state `awaiting_tool_call_decision`, `tool_call{name, risk_class, arguments_preview}` and one `approvals.steps[kind: 'tool_call']` bound to `payload_sha256`. Decision sent: `approve_tool_call{payload_sha256}`. Banner: THIS TOOL CALL RUNS ONLY IF YOU APPROVE IT, then TOOL CALL WAS APPROVED AND RELEASED, or NOTHING WILL RUN. A confirm dialog with an acknowledgement checkbox precedes the decision (design choice for `explicit_confirm`).
- **Design copy, not contract:** the one-line description per `risk_class` ("Runs a command on your device", ...), "Requested by the model, not typed by you" (derived from `trusted: false`), and "your agent app may run it one time". Needs confirmation by architecture/PM.
- **Malformed messages (UI-local, AT-15):** every incoming snapshot is validated (JSON, `contract_version === 1`, required sections, known state, strictly increasing revision). A failing message is dropped, never rendered, and a notice lists the reason class and count only. A review that becomes unreadable after acceptance shows "This review cannot be shown safely" with a local Dismiss; all decision actions on it are refused.
- **Prototype controls** (not product UI): Inject unreadable message, Inject stale revision, Inject unknown contract version, Corrupt selected review.
- **Assumptions:** the notice's "restart Aegis" advice, and "Aegis rejects the request when it times out" (from `timeout.on_expiry: 'reject'`).
- **Not done:** `risk_class: 'payment'` or other not-approvable tool calls (the layout supports `not_approvable` via "Nothing to approve" but no sample exists); real-time `inbound`/`response_blocked` interplay.

## 9. QA follow-ups N6-N8 (design/contract-v1.1)
| ID | Fix | Evidence (scripted, mock daemon) |
|---|---|---|
| N6 | Approval is locked whenever Aegis did not state what will be sent: `transmit_content` null on a v1.1 snapshot disables Approve transformation, Send original, Approve and send and the acknowledgement checkboxes; the mock daemon also refuses. A v1.0 daemon (no `contract_minor`) is still inferred and labelled | null case: banner SEND CONTENT NOT STATED BY AEGIS, all controls disabled, forced action ignored; v1.0: "Inferred" shown and send enabled after acknowledgements |
| N9 | `transmit_content: 'transformed'` with no proposal is treated the same way (banner + lock, no send) | banner and disabled send observed |
| N7 | Inert Undo button removed from the States page "User rejects request" panel | no button matching /undo/ on #states |
| N8 | A stale banner (e.g. "Edit sent for re-inspection") is not shown on a closed review; only banners marked `keep` (unsaved edit discarded) survive | banner visible while open, gone after expiry; N5 discard banner still shown |

## 10. Keyboard and screen-reader walkthrough: Review Center
**Status: written, NOT yet executed with a screen reader.** What I did observe (real Tab/Enter key presses in a browser pane, plus scripted DOM checks): the Tab order, skip link, focus moves listed below, and the accessible names. Everything marked "TESTER" must be performed and recorded by a person with NVDA (Firefox and Chrome), Narrator (Edge), and, once a macOS build is confirmed, VoiceOver (Safari). The platform for v1.0 is unconfirmed; the script is platform-neutral.
Setup: `python -m http.server 5188 --directory design`, open `http://localhost:5188/#review`. Sample data only; a mock daemon stands in for Aegis (hashes are fake). Never paste real prompts.

### 10.1 Keys
Tab / Shift+Tab move between controls. Enter or Space activates buttons and ticks checkboxes. Esc closes a dialog. There is no arrow-key navigation: the queue items and the operating-mode choices (a `radiogroup` on the Router page) are plain buttons. **Known gap:** the mode radiogroup does not implement arrow keys as WAI-ARIA expects; Tab reaches each option. No shortcuts exist.

### 10.2 Observed Tab order (Review Center, 1440 px, first load)
1 Skip to content (visible at top-left on focus) → 2-15 sidebar links: Overview, Review Center, Security Inspector, Context & Profiles, Model Router, Analytics, Integrations, Settings, then Design board links (7) → theme toggle → queue items (one per review, in queue order) → selected review's controls (Simulate timeout, step buttons, Reject) → prototype controls (last, labelled "not product UI"). Skip link + Enter: focus moves to the main region, the route stays `#review` (the link does not navigate).
Accessible name of a queue item (set with `aria-label`): "High risk, Personal details found, ChatGPT, outbound, Awaiting transform decision".

### 10.3 Announcements
One persistent live region (`role=status`, outside the re-rendered app) announces, on selection and on state change: `{app}, review {id}: {state}. {banner text}` (a transient message title is prefixed). Examples: "ChatGPT, review rv_01: Awaiting transform decision. TRANSFORMED TEXT WILL BE SENT, NOT THE ORIGINAL"; "VS Code assistant, review rv_02: Blocked. NOTHING WILL BE SENT". The review is not wrapped in an `aria-live` region, so a change must not make the reader re-read the whole screen. The countdown is not live (no per-second chatter). TESTER: confirm exactly one announcement per change, in the order above, and that no announcement leaks content other than the banner.

### 10.4 Script (TESTER: record pass/fail, reader + browser + version for each)
| # | Do | Must hear or see | Focus after |
|---|---|---|---|
| 1 | Tab once from page load | "Skip to content", link visible | skip link |
| 2 | Enter | main region; route still `#review` | content region |
| 3 | Tab to rv_01, Enter | live: "...Awaiting transform decision. TRANSFORMED TEXT WILL BE SENT, NOT THE ORIGINAL" | queue item |
| 4 | Tab to "Approve and send" | button announced as unavailable/dimmed with the reason "Locked until the transform step is decided..." (`aria-describedby`); acknowledgement checkboxes announced as dimmed | n/a |
| 5 | Enter on "Approve transformation" | live: "...Awaiting send decision. TRANSFORMED TEXT WILL BE SENT..."; stepper shows step 2 complete | first acknowledgement checkbox |
| 6 | Enter on "Approve and send" with boxes unticked | message "consent_required: Tick each item to confirm before sending." is announced; nothing sent | button |
| 7 | Tick all, Enter on "Approve and send" | live: "...Sent" then "...Sent. Content has been cleared"; toast "Approval bound to..." | content region |
| 8 | Fresh page, rv_01: Enter on "Edit proposed text" | textarea labelled "Edit proposed text" | textarea |
| 9 | Type, Tab to "Save and re-inspect", Enter | live: "Re-inspecting"; status "Re-inspecting. Approvals are on hold." | status row |
| 10 | Wait ~1.5 s | focus moves to "Approve transformation"; stepper "Re-inspection clean" | Approve transformation |
| 11 | Edit again adding an email-like text, save | banner NOTHING WILL BE SENT UNTIL THE EDIT IS FIXED; stepper "Re-inspection failed"; Approve controls dimmed | Edit button or status |
| 12 | "Reject request" | dialog "Reject this request?" announced; "Rejection is final" in text | first button "Keep reviewing" |
| 13 | Tab/Shift+Tab inside dialog | focus wraps, never reaches the page behind (page is `inert`) | within dialog |
| 14 | Esc | dialog closes | the Reject button that opened it |
| 15 | Confirm reject | live: "...Rejected. NOTHING WILL BE SENT"; no send controls remain | content region |
| 16 | rv_03 (uninspected) → "Send uninspected..." | dialog with unticked checkbox; confirm button dimmed until ticked; Esc then reopen: unticked again | checkbox |
| 17 | rv_02 (blocked) | no approve/edit/override controls; remediation text; "Dismiss from queue" | button |
| 18 | rv_07 (tool call) → "Approve this tool call..." | dialog names the tool; confirm dimmed until ticked | checkbox |
| 19 | "Simulate timeout" on an open review | live: "...Expired. NOTHING WILL BE SENT" within ~2 s; editor closed if editing, with "Your unsaved edit was discarded" | content region |
| 20 | "Inject unreadable message" | alert "1 message from Aegis could not be read and was ignored"; reviews unchanged; no raw content read out | n/a |
| 21 | "Simulate daemon unreachable", then try a decision | alert "Aegis is not running... Nothing is sent"; decision refused | n/a |
TESTER also checks: text zoom 200% and 400% (no clipped banner or button), Windows High Contrast / forced colours (banner borders and status icons still distinguish states), `prefers-reduced-motion` (no slide/spin beyond the spinner showing static), and that no status is conveyed by colour alone (every badge has an icon and text).

### 10.5 Decisions on QA notes
- **Acknowledgement checkboxes while send is locked:** they stay visible and are disabled (not removed) whenever the transmit step is `pending` with `explicit_confirm`. Reason: the user can see what they will be asked to confirm and why the step is locked. Evidence: 3 checkboxes present, all disabled, when `transmit_content` is null. My earlier N6 check passed vacuously (an empty list); this corrects it.
- **Edit while send content is not stated:** disabled. Editing a proposal whose destination text Aegis cannot describe has no safe outcome; only Reject (and local Dismiss for closed items) remains.
- **Reject** is always enabled while a review is open.

### 10.6 Not verified
Real screen-reader output; VoiceOver; touch screen readers; the live-region timing (80 ms delay) with a real reader; `role=alert` on the malformed-message notice being announced when the DOM is replaced (I rely on the live region text, which does not include the malformed notice: TESTER must check and report); focus order on the Router, Settings and other screens (only counted structurally).

## 11. Extension popup and side panel on contract v1.1
Rendered from `Coverage` objects (10.3): `integration.label` is shown verbatim (Protected / Partially protected / Not protected), with `tier`, `outbound`, `inbound`, `limitations[]`, and `adapter{name, version, tested_on, canary}`. The UI does not decide coverage. Per-site samples: Tier A protected (chatgpt.com), Tier B partially protected (claude.ai), Tier C not protected with a failed self-test (gemini.example), Tier D not protected with no adapter (unsupported-ai.example).
UI-local states (the daemon cannot send them): **companion/daemon disconnected, fail closed (AT-14)**: badge Disconnected, "Aegis is not running", "requests from supported sites are blocked until Aegis is running again", Retry; **waiting for Aegis** (skeleton); **unreadable answer** (treated as not protected). The 5-state vocabulary also includes "Needs attention", used elsewhere in the app, not in the popup.
Also shown from app state, not from `Coverage`: current mode and profile (`ReviewRequest.mode`/`profile` exist; the popup has no contract message of its own for them), pending review count, and the title of the waiting review.
**Assumptions needing confirmation (not in the contract):** the popup and side panel message set itself (the contract defines review events only); the meaning of tiers A-D (shown as letters, not described); the text "Do not paste sensitive information here" and "use the API gateway"; "Open the Aegis app... Retry" steps; the 440 px side panel and its metadata-only activity list; `canary: 'fail'` shown as "Self-test failed" with the daemon's own limitation text; no Chrome extension API (`chrome.sidePanel`) is used: this is a visual prototype only.
Checks (scripted, browser): 9 frames render; all four labels verbatim; tiers A-D; fail-closed text; self-test badges; Retry gives a toast; no console errors; no overflow at 1440 or 375 px; buttons 44 px or taller at 375 px. Not done: keyboard walkthrough and screen-reader testing of the popup, real extension runtime, light-theme review.
