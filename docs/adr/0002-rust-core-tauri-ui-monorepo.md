# ADR-0002: Rust core, Tauri/React UI, TypeScript extension, monorepo

**Status:** Proposed (Tauri + React + TS is an approved input; Rust core is the proposal)

**Context.** Tauri's backend is Rust. The TCB handles hostile input (prompts, model output, network) and secrets; memory safety, predictable performance, and single-binary distribution matter. Linear-time regex and zeroisation are available in Rust.

**Decision.** Core crates in Rust (tokio, axum, rusqlite+SQLCipher, `regex`/`aho-corasick`, `ort` for ONNX, `zeroize`, `chacha20poly1305`, `argon2`). Desktop UI in React + TypeScript inside Tauri (thin `src-tauri`, no policy logic). Extension in TypeScript (MV3). One monorepo: Cargo workspace + pnpm workspace; TS protocol types generated from Rust.

**Consequences.**
+ Memory safety in the TCB; one language for daemon, NM relay and Tauri.
+ Generated types prevent UI/daemon contract drift.
− Contributors need Rust + TS; ONNX/SQLCipher native build complexity in CI (mitigated by pinned toolchains, vendored builds).

**Alternatives.** Node/TypeScript daemon (faster to iterate, weaker memory/secret handling, heavier runtime); Go daemon (good, but splits languages with Tauri and weaker type-level capability patterns).
