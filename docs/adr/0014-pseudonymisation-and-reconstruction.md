# ADR-0014: Pseudonymisation and local reconstruction scope

**Status:** Proposed

**Decision.**
- Placeholders are typed and stable within a conversation scope (`[PERSON_1]`, `[ORG_2]`, `[LOCATION_1]`); different conversations get unrelated mappings (no cross-conversation linkability by default).
- Reconstruction happens in the daemon after response inspection, before release to the local client. Unmatched/altered placeholders are shown as placeholders, never guessed.
- **Never** pseudonymised or reconstructed: credentials, passwords, API keys, private keys, tokens, seed phrases (these are blocked or removed outright).
- High-identifiability combos (rare job title + small place + date) are handled by decontextualisation (generalisation) rather than just name replacement; residual re-identification risk is shown in review as a qualitative indicator, not a guarantee.
- Mapping storage per ADR-0009 (AEAD, TTL, crypto-shred on conversation delete). Mapping tables are never sent anywhere or shown to the extension.
- Extension flows: reconstruction result returned only for local display; the page DOM necessarily receives it (page JS could read it) — flagged as a Tier C/B limitation.

**Consequences.** + Cloud sees only pseudonyms; user sees real values. − Quality loss when models transform placeholders; stylistic differences; users need to understand limits (UI copy).

**Alternatives.** Format-preserving fake data (better fluency, but fake values can be mistaken for real and raise collision risk) — optional later.
