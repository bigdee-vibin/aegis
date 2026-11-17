# Aegis project site (static)

A static, no-build GitHub Pages site for Aegis. It shows the project vision, an honest status panel, an in-browser Review Center **simulation**, and the full design board.

**Aegis is pre-release. Design prototype and early gateway slice. Not a working firewall yet. No protection is claimed.** Nothing on this site measures or demonstrates real detection.

## Contents

| Path | What it is |
|---|---|
| `index.html` | Landing page: vision, status, flow, scope, coverage tiers, links |
| `demo/` | Review Center simulation (toy regexes on fake sample patterns, all client-side) |
| `board/` | Copy of `design/` (the coded design board). Only change: the two Google Fonts `<link>` lines were removed from `board/index.html` so the site makes no external requests |
| `assets/aegis.css` | Copy of `design/aegis.css` (design tokens, dark default, light theme) |
| `assets/site.css`, `assets/site.js` | Site layer and theme toggle |
| `DEPLOY.md` | How to publish, with a draft Pages workflow |

All links are relative so the site works under `https://bigdee-vibin.github.io/aegis/`. There are no external scripts, fonts, analytics or trackers. The only browser storage used is the light/dark theme choice (`localStorage`, optional).

## Run locally

```
cd site
python -m http.server 8080
```

Then open http://localhost:8080/.

## Enable GitHub Pages

See `DEPLOY.md`. Short version: GitHub's branch deploy only offers `/` or `/docs`, not `/site`, so use a GitHub Actions workflow (draft in `DEPLOY.md`; it is intentionally not installed under `.github/`).

## Keeping it honest

The status text was copied by hand from `docs/status/2026-11-17-initial-audit.md`, the PRD draft (`docs/prd-v1` branch) and commit notes on `m0-m1-scaffold`. A `docs/status/go-live-readiness.md` was not found in any branch. Update the status panel in `index.html` whenever those sources change, and never add effectiveness claims without measured results (PRD NFR-8).

`board/` and `assets/aegis.css` are copies and will drift from `design/`; re-copy when the design changes.
