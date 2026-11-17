# QA verification 2: design stack (tool-call-states 89f349e, a11y-walkthrough 72e5f01, extension-states 0fbbbfa)

Independent AI QA agent, Chromium pane, mock daemon only, sample data. Not tested: screen readers, text zoom, forced colors, reduced motion, extension runtime, real daemon, wrong-hash approve_tool_call through the UI (source read only), light-theme contrast numerically. Note: browser cached CSS between branches; skip-link and overflow results are from the final-head re-run.

| Area | Verdict |
|---|---|
| Tool-call review (banner, approve/reject, confirm dialog), malformed x3, corrupt-review fallback | PASS |
| Skip link, accessible queue names, edit/save/re-inspection focus moves, disabled-but-present acks (non-vacuous), Edit disabled when send content not stated | PASS |
| Single persistent live region | PARTIAL (toast container is a second live region) |
| 21-step walkthrough (scripted, no screen reader) | 16 PASS, step 15 FAIL, steps 4/13 partly/not run |
| Keyboard-only full send | PASS |
| Extension popup/side-panel states: coverage labels, tiers A-D, Disconnected fail-closed, waiting, unreadable | PASS (data hard-coded samples) |
| Popup 360 px / side panel 440 px sizes | FAIL (widths vary 192-360 px) |
| Regression, 0 console errors, no overflow at 1024/375 on 15 screens, no cross-origin | PASS |

## New defects
- N10 low-medium (a11y): after confirming Reject, focus ends on BODY, not the content region (walkthrough step 15).
- N11 low: popup/side-panel widths inconsistent (333, 318, 192 px frames).
- N12 low: second live region (toasts) alongside #live.
- N13 low: "Simulate timeout" and the scrollable table wrapper precede the approval buttons in Tab order; Save is two stops past the textarea.
- N14 info: Disconnected popup frame is static, not tied to the Review Center's daemon-unreachable state.
