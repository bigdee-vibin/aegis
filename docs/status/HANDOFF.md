# Aegis — Project Handoff (source of truth lives on GitHub)

**Current build state for a resuming agent: see [BUILD-STATUS.md](BUILD-STATUS.md) (newer than this file).**

**As of 2026-11-17.** All work sessions and local files may be deleted. This repo is the single source of truth. Everything below is verified from GitHub unless marked otherwise.

## Repo and PRs
- Repo: https://github.com/bigdee-vibin/aegis (public). `main` = baseline docs only (ADR-0001..0020, architecture, threat model, audit, design tokens).
- Open PRs, not merged: #1 `docs/prd-v1` (PRD v1.0 draft, ADR-0022), #2 `design/board-v1` (design board prototype), #3 `m0-m1-scaffold` (Cargo workspace, CI, M1 gateway slice, Review Center contract, ADR-0021). Other branch: `s0-3-testkit` (S0.3 test harness).
- Nothing is on `main` beyond the baseline until PRs are merged by the owner.

## Product (read in order)
1. `docs/product/PRD.md` (on `docs/prd-v1`): requirement IDs, canonical acceptance suite AT-01..AT-19.
2. `docs/architecture/` (system architecture, API contracts incl. Review Center contract §10, backlog M0-M7, deployment, risks).
3. `docs/security/02-threat-model.md`, `docs/adr/` (ADR index in `docs/adr/README.md`).
4. `docs/status/go-live-readiness.md`: 14 go-live gates, all NOT passed.

## Decisions made by the owner
v1.0 OS scope Windows+macOS+Linux (ADR-0020); OPEN-01..10 (ADR-0019): adapters ChatGPT/Claude/Gemini, opt-in updates, per-request uninspected override, Apache-2.0; public repo; guard model separate from the called model and lightweight (FR-DET-9/10, ADR-0022); always-on daemon independent of the browser (FR-DEP).

## Verified vs not verified
- Verified: documents exist on GitHub; branches/PRs above.
- Reported by authors, NOT independently verified: M1 gateway passes 51-58 tests locally on a scratch copy with older pinned dependencies (pins are not in the repo). CI has never completed (runs were cancelled by pushes). No test on macOS/Linux. AT-07/08 partial (connection tripwire only), AT-17 only against a JSONL stand-in.
- Not started: M2-M7 (approval engine, Ollama/LOCAL ONLY, inbound inspection, vault, extension, hardening, release).

## Open items needing the owner
- Merge PRs #1-#3 (nothing merged yet).
- Owner email appears in the repo's first commit `5edf4af` (author) on `main`; removing it needs rewriting `main`. Enable GitHub's "keep my email private" and "block command line pushes that expose my email".
- Token needs Contents, Pull requests, Issues, Workflows (and Pages if the demo site is published).
- Decisions Q-1..Q-6 in PRD §14.

## Delivery process
Cycle: inspect -> compare to PRD -> prioritise -> implement -> independent test -> review -> document -> release. Independent tester and QA verify before any "complete". Rules: no claim without evidence; branches + PRs; never force-push `main`; no real prompts, secrets or personal data in the repo; noreply identity only (`bigdee-vibin@users.noreply.github.com`).

## Pending workstreams at handoff (results not yet seen)
Independent tester (M1 + design board), security review of M1 crates, QA matrices (`docs/qa/`), synthetic corpora (S0.4), GitHub Pages demo site, architecture work (S2.1 PII detectors, approval engine, transform). If their outputs are not on GitHub, they are lost; re-run them from the backlog.
