# QA verification 1: design board fixes (design/board-v1 @ 3644313)

Independent AI QA agent, Chromium pane only, mock daemon only (nothing here validates the real gateway). Mostly JS-dispatched clicks; real key presses for dialogs and one keyboard-only send. Not tested: screen readers, real clipboard, other browsers, visual screenshots, theme contrast, real devices.

| Item | Verdict | Note |
|---|---|---|
| D1 dismiss blocked item | PASS | no console error, queue updates |
| D2 rejected uninspected cannot be sent | PASS | forced override attempt: "already closed", state stays rejected |
| D3 acknowledgement resets on Cancel/Esc | PASS | |
| D4 edit adding email locks approval | PASS | forced approve_transmit returns reinspection_failed |
| D5 approvals disabled while editing | PASS | |
| D6 expiry/wording/re-inspection step | PARTIAL | expiry PASS; identical-text wording no longer contradictory but still says WILL BE SENT while locked; stepper hidden during re-inspection |
| D7 dialog focus (3 dialogs, real keys) | PASS | move in, trap, Esc, return to opener |
| B1 no console errors, 15 screens | PASS | |
| D8 copy control | PASS (control) | clipboard success path not verified |
| D8 port 8731 everywhere | PASS | |
| D9 no cross-origin requests | PASS | |
| D9 overflow at 1024 px | FAIL | 7 px on all 15 screens; cause: always-laid-out ::before/::after tooltip of the theme-toggle `.tip` button |
| D9 overflow at 375 px | PASS | |
| Full flows (two approvals, local route, blocked, expired, uninspected, keyboard-only send) | PASS | |

## New defects
- N1 medium: expiry during re-inspection can resurrect an expired request (edit_proposal timer sets awaiting_transform_decision without checking terminal state; timing dependent, reproduced once).
- N2 low: unguarded EXTRA_ACT handlers throw if injected (not reachable from real UI).
- N3 low: banner says WILL BE SENT while re-inspection failed and send is locked; should say nothing will be sent until fixed.
- N4 low (a11y): after keyboard-activating Approve transformation, focus drops to BODY; stepper hidden while reinspecting.
- N5 info: unsaved edit silently discarded on expiry.
