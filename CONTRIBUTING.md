# Contributing

- Read `docs/architecture/01-system-architecture.md` and the ADRs first. Changes that alter an invariant (I1–I10) need a new ADR.
- Sign off every commit (`git commit -s`) per the Developer Certificate of Origin (DCO; default per ADR-0019, CLA not required).
- Never add real personal data, real credentials or captured provider traffic to tests or corpora; use synthetic data.
- Only `aegis-egress` may depend on an outbound HTTP client (enforced by `deny.toml`).
- `aegis-policy` may depend only on `aegis-core`.
- Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` before pushing.
