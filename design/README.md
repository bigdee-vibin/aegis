# Aegis design board (coded prototype)

No Figma connection was available, so this is a coded, clickable design board rather than a `.fig` file.

Run: `python -m http.server 5188 --directory aegis-design`, then open http://localhost:5188

- `aegis.css`: tokens (dark default, `[data-theme="light"]`) and all components with states.
- `icons.js`: Lucide-style outline icons, status and severity vocabulary.
- `app.js`: shell, Overview, Review Center (two-step consent), Inspector, Profiles, Router, Analytics, Integrations, Settings, dialogs.
- `review.js`: Review Center rendering contract_version 1, with a mock daemon (fake hashes).
- `MASTER.md`: rules and accessibility evidence. `tools/contrast.py`: contrast measurement.
- `extra.js`: Design board, Design system, States (12), Extension popup, Onboarding (10 steps), Flows A-F, Handoff.

Notes
- All data is illustrative sample data.
- The ui-ux-pro-max generator proposed a purple palette; it was overridden with the brief's teal system.
- Platform for v1.0 is unconfirmed; DPAPI wording is a swappable label.
- Not done: native Figma file, Figma prototype links, formal WCAG audit with assistive tech.
